//! Arabic language analysis and validation
//!
//! This module provides Arabic-specific text analysis, including character
//! frequency analysis and right-to-left text handling.

use unicode_script::{Script, UnicodeScript};

/// Arabic text analyzer
pub struct ArabicAnalyzer;

impl ArabicAnalyzer {
    /// Check if a character is Arabic
    pub fn is_arabic_character(ch: char) -> bool {
        matches!(ch.script(), Script::Arabic)
    }

    /// Check if text appears to be valid Arabic
    pub fn is_likely_arabic(text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        let arabic_chars: Vec<char> = text.chars().filter(|c| Self::is_arabic_character(*c)).collect();
        
        // Must have at least 60% Arabic characters (excluding whitespace/punctuation)
        let total_non_space = text.chars().filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation()).count();
        if total_non_space == 0 {
            return false;
        }

        let arabic_ratio = arabic_chars.len() as f64 / total_non_space as f64;
        arabic_ratio >= 0.6 && arabic_chars.len() >= 2
    }

    /// Get Arabic text statistics
    pub fn get_arabic_stats(text: &str) -> ArabicTextStats {
        let mut arabic_chars = 0;
        let mut total_chars = 0;
        let mut unique_chars = std::collections::HashSet::new();
        let mut common_chars = 0;
        let mut connecting_chars = 0;

        for ch in text.chars() {
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                continue;
            }

            total_chars += 1;
            unique_chars.insert(ch);

            if Self::is_arabic_character(ch) {
                arabic_chars += 1;
                
                if Self::is_common_arabic_char(ch) {
                    common_chars += 1;
                }

                if Self::is_connecting_arabic_char(ch) {
                    connecting_chars += 1;
                }
            }
        }

        ArabicTextStats {
            arabic_characters: arabic_chars,
            total_characters: total_chars,
            unique_characters: unique_chars.len(),
            common_characters: common_chars,
            connecting_characters: connecting_chars,
            arabic_ratio: if total_chars > 0 { arabic_chars as f64 / total_chars as f64 } else { 0.0 },
        }
    }

    /// Check if a character is a commonly used Arabic character
    pub fn is_common_arabic_char(ch: char) -> bool {
        // Most frequently used Arabic characters
        matches!(ch,
            // Basic Arabic letters (most common)
            'ا' | 'ب' | 'ت' | 'ث' | 'ج' | 'ح' | 'خ' | 'د' | 'ذ' | 'ر' |
            'ز' | 'س' | 'ش' | 'ص' | 'ض' | 'ط' | 'ظ' | 'ع' | 'غ' | 'ف' |
            'ق' | 'ك' | 'ل' | 'م' | 'ن' | 'ه' | 'و' | 'ي' |
            // Common combinations and forms
            'أ' | 'إ' | 'آ' | 'ة' | 'ى' | 'ؤ' | 'ئ' |
            // Common diacritics (though often omitted in modern text)
            'َ' | 'ُ' | 'ِ' | 'ْ' | 'ً' | 'ٌ' | 'ٍ'
        )
    }

    /// Check if an Arabic character connects to adjacent letters
    pub fn is_connecting_arabic_char(ch: char) -> bool {
        // Most Arabic letters connect to adjacent letters
        // Non-connecting letters: ا د ذ ر ز و
        !matches!(ch, 'ا' | 'د' | 'ذ' | 'ر' | 'ز' | 'و') && Self::is_arabic_character(ch)
    }

    /// Validate Arabic text authenticity
    pub fn validate_arabic_text(text: &str) -> bool {
        if text.trim().is_empty() {
            return false;
        }

        let stats = Self::get_arabic_stats(text);
        
        // Validation criteria:
        // 1. Must have at least 2 Arabic characters (Arabic words are typically 2+ chars)
        // 2. Arabic characters should make up at least 60% of non-whitespace text
        // 3. Should have some common characters
        // 4. Should have reasonable mix of connecting/non-connecting characters
        
        if stats.arabic_characters < 2 {
            return false;
        }

        if stats.arabic_ratio < 0.6 {
            return false;
        }

        // Should have some common Arabic characters
        if stats.arabic_characters > 3 && stats.common_characters == 0 {
            return false;
        }

        // Check for reasonable character diversity
        let diversity_ratio = stats.unique_characters as f64 / stats.total_characters as f64;
        if stats.total_characters > 10 && diversity_ratio < 0.2 {
            return false; // Too repetitive
        }

        true
    }

    /// Score Arabic text based on character frequency and patterns
    pub fn score_arabic_text(text: &str) -> f64 {
        if text.trim().is_empty() {
            return -20.0;
        }

        let stats = Self::get_arabic_stats(text);
        
        if stats.arabic_characters == 0 {
            return -20.0;
        }

        let mut score = 0.0;

        // Base score for having Arabic characters
        score += stats.arabic_ratio * 5.0;

        // Bonus for common characters (absolute count + ratio)
        if stats.arabic_characters > 0 {
            // Give bonus for having common characters at all
            score += (stats.common_characters.min(5) as f64) * 0.4;
            // Additional ratio-based bonus, but capped to avoid penalizing longer text
            let common_ratio = stats.common_characters as f64 / stats.arabic_characters as f64;
            score += common_ratio.min(0.8) * 2.5;
        }

        // Length bonus (reward longer meaningful text)
        if stats.arabic_characters >= 3 {
            score += 1.0;
        }
        if stats.arabic_characters >= 6 {
            score += 0.5; // Additional bonus for longer text
        }

        // Connecting character bonus (Arabic words typically have connecting letters)
        if stats.arabic_characters > 1 {
            let connecting_ratio = stats.connecting_characters as f64 / stats.arabic_characters as f64;
            score += connecting_ratio * 1.5;
        }

        // Diversity bonus
        if stats.total_characters > 0 {
            let diversity = stats.unique_characters as f64 / stats.total_characters as f64;
            score += diversity * 2.0;
        }

        // Penalty for very short text
        if stats.arabic_characters < 3 {
            score -= 1.0;
        }

        // Convert to negative scale (consistent with existing trigram scores)
        -5.0 + score
    }

    /// Check if text might be right-to-left oriented
    pub fn is_likely_rtl(text: &str) -> bool {
        let stats = Self::get_arabic_stats(text);
        stats.arabic_ratio > 0.5
    }

    /// Normalize Arabic text for analysis (remove diacritics, normalize forms)
    pub fn normalize_arabic_text(text: &str) -> String {
        text.chars()
            .map(|ch| {
                // Remove common diacritics
                match ch {
                    'َ' | 'ُ' | 'ِ' | 'ْ' | 'ً' | 'ٌ' | 'ٍ' | 'ّ' => None,
                    // Normalize some letter forms
                    'أ' | 'إ' | 'آ' => Some('ا'),
                    'ة' => Some('ه'),
                    'ى' => Some('ي'),
                    'ؤ' => Some('و'),
                    'ئ' => Some('ي'),
                    _ => Some(ch),
                }
            })
            .filter_map(|ch| ch)
            .collect()
    }
}

/// Statistics about Arabic text content
#[derive(Debug, Clone)]
pub struct ArabicTextStats {
    /// Number of Arabic characters
    pub arabic_characters: usize,
    /// Total non-whitespace, non-punctuation characters
    pub total_characters: usize,
    /// Number of unique characters
    pub unique_characters: usize,
    /// Number of common Arabic characters
    pub common_characters: usize,
    /// Number of connecting Arabic characters
    pub connecting_characters: usize,
    /// Ratio of Arabic characters to total
    pub arabic_ratio: f64,
}

impl ArabicTextStats {
    /// Check if this represents likely valid Arabic text
    pub fn is_likely_valid(&self) -> bool {
        self.arabic_characters >= 2 && 
        self.arabic_ratio >= 0.6 &&
        (self.arabic_characters < 4 || self.common_characters > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_character_detection() {
        assert!(ArabicAnalyzer::is_arabic_character('ا'));
        assert!(ArabicAnalyzer::is_arabic_character('ب'));
        assert!(ArabicAnalyzer::is_arabic_character('ع'));
        assert!(ArabicAnalyzer::is_arabic_character('م'));
        assert!(!ArabicAnalyzer::is_arabic_character('a'));
        assert!(!ArabicAnalyzer::is_arabic_character('1'));
    }

    #[test]
    fn test_common_arabic_chars() {
        assert!(ArabicAnalyzer::is_common_arabic_char('ا'));
        assert!(ArabicAnalyzer::is_common_arabic_char('ب'));
        assert!(ArabicAnalyzer::is_common_arabic_char('م'));
        assert!(ArabicAnalyzer::is_common_arabic_char('ل'));
    }

    #[test]
    fn test_connecting_chars() {
        assert!(ArabicAnalyzer::is_connecting_arabic_char('ب'));
        assert!(ArabicAnalyzer::is_connecting_arabic_char('ت'));
        assert!(ArabicAnalyzer::is_connecting_arabic_char('م'));
        assert!(!ArabicAnalyzer::is_connecting_arabic_char('ا')); // Non-connecting
        assert!(!ArabicAnalyzer::is_connecting_arabic_char('د')); // Non-connecting
        assert!(!ArabicAnalyzer::is_connecting_arabic_char('ر')); // Non-connecting
    }

    #[test]
    fn test_arabic_detection() {
        assert!(ArabicAnalyzer::is_likely_arabic("السلام"));
        assert!(ArabicAnalyzer::is_likely_arabic("مرحبا"));
        assert!(ArabicAnalyzer::is_likely_arabic("العربية"));
        assert!(!ArabicAnalyzer::is_likely_arabic("hello"));
        assert!(!ArabicAnalyzer::is_likely_arabic("hello مرحبا")); // Too much English
    }

    #[test]
    fn test_arabic_stats() {
        let stats = ArabicAnalyzer::get_arabic_stats("مرحبا");
        assert!(stats.arabic_characters > 0);
        assert!(stats.arabic_ratio > 0.8);
        assert!(stats.common_characters > 0);
    }

    #[test]
    fn test_arabic_validation() {
        assert!(ArabicAnalyzer::validate_arabic_text("السلام"));
        assert!(ArabicAnalyzer::validate_arabic_text("مرحبا"));
        assert!(!ArabicAnalyzer::validate_arabic_text("hello"));
        assert!(!ArabicAnalyzer::validate_arabic_text(""));
        assert!(!ArabicAnalyzer::validate_arabic_text("ا")); // Too short
    }

    #[test]
    fn test_arabic_scoring() {
        let score1 = ArabicAnalyzer::score_arabic_text("مرحبا");
        let score2 = ArabicAnalyzer::score_arabic_text("hello");
        let score3 = ArabicAnalyzer::score_arabic_text("السلام عليكم");
        
        assert!(score1 > score2); // Arabic should score better than English
        assert!(score3 > score1); // Longer Arabic text should score better
        assert!(score2 < -10.0);  // Non-Arabic should get very low score
    }

    #[test]
    fn test_rtl_detection() {
        assert!(ArabicAnalyzer::is_likely_rtl("مرحبا"));
        assert!(!ArabicAnalyzer::is_likely_rtl("hello"));
        assert!(!ArabicAnalyzer::is_likely_rtl("hello مرحبا")); // Mixed, but Arabic not dominant
    }

    #[test]
    fn test_arabic_normalization() {
        let normalized = ArabicAnalyzer::normalize_arabic_text("مَرْحَبًا");
        assert!(!normalized.contains('َ')); // Diacritics removed
        assert!(!normalized.contains('ْ'));
        assert!(!normalized.contains('ً'));
        
        let normalized2 = ArabicAnalyzer::normalize_arabic_text("أهلاً");
        assert!(normalized2.starts_with('ا')); // أ normalized to ا
    }
}