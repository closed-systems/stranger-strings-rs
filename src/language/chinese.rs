//! Chinese (Simplified) language analysis and validation
//!
//! This module provides Chinese-specific text analysis, including character
//! frequency analysis and validation for Chinese text.

use unicode_script::{Script, UnicodeScript};

/// Chinese character frequency analyzer
pub struct ChineseAnalyzer;

impl ChineseAnalyzer {
    /// Check if a character is a Han (Chinese) character
    pub fn is_han_character(ch: char) -> bool {
        matches!(ch.script(), Script::Han)
    }

    /// Check if text appears to be valid Chinese
    pub fn is_likely_chinese(text: &str) -> bool {
        if text.is_empty() {
            return false;
        }

        let han_chars: Vec<char> = text.chars().filter(|c| Self::is_han_character(*c)).collect();
        
        // Must have at least 60% Han characters (excluding whitespace/punctuation)
        let total_non_space = text.chars().filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation()).count();
        if total_non_space == 0 {
            return false;
        }

        let han_ratio = han_chars.len() as f64 / total_non_space as f64;
        han_ratio >= 0.6 && han_chars.len() >= 1
    }

    /// Get Chinese character statistics
    pub fn get_chinese_stats(text: &str) -> ChineseTextStats {
        let mut han_chars = 0;
        let mut total_chars = 0;
        let mut unique_chars = std::collections::HashSet::new();
        let mut common_chars = 0;

        for ch in text.chars() {
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                continue;
            }

            total_chars += 1;
            unique_chars.insert(ch);

            if Self::is_han_character(ch) {
                han_chars += 1;
                
                // Check against common Chinese characters
                if Self::is_common_chinese_char(ch) {
                    common_chars += 1;
                }
            }
        }

        ChineseTextStats {
            han_characters: han_chars,
            total_characters: total_chars,
            unique_characters: unique_chars.len(),
            common_characters: common_chars,
            han_ratio: if total_chars > 0 { han_chars as f64 / total_chars as f64 } else { 0.0 },
        }
    }

    /// Check if a character is a commonly used Chinese character
    pub fn is_common_chinese_char(ch: char) -> bool {
        // Most frequently used Chinese characters (simplified)
        // This is a subset - in a real implementation, you'd want a larger set
        matches!(ch,
            // Top 50 most common Chinese characters
            '的' | '一' | '是' | '在' | '不' | '了' | '有' | '和' | '人' | '这' |
            '中' | '大' | '为' | '上' | '个' | '国' | '我' | '以' | '要' | '他' |
            '时' | '来' | '用' | '生' | '到' | '作' | '地' | '于' | '出' |
            '就' | '分' | '对' | '成' | '会' | '可' | '主' | '发' | '年' | '动' |
            '同' | '工' | '也' | '能' | '下' | '过' | '子' | '说' | '产' | '种' |
            // Additional common characters
            '好' | '天' | '水' | '火' | '土' | '木' | '金' | '日' | '月' |
            '山' | '川' | '田' | '目' | '口' | '手' | '足' | '心' | '头' | '身' |
            // More common characters
            '你' | '世' | '界' | '家' | '学' | '校' | '老' | '师' | '朋' | '友'
        )
    }

    /// Validate Chinese text authenticity
    pub fn validate_chinese_text(text: &str) -> bool {
        if text.trim().is_empty() {
            return false;
        }

        let stats = Self::get_chinese_stats(text);
        
        // Validation criteria:
        // 1. Must have at least 1 Han character
        // 2. Han characters should make up at least 60% of non-whitespace text
        // 3. Should have some common characters if text is long enough
        // 4. Reasonable character diversity (not just repeated characters)
        
        if stats.han_characters == 0 {
            return false;
        }

        if stats.han_ratio < 0.6 {
            return false;
        }

        // For longer texts, expect some common characters
        if stats.han_characters > 5 && stats.common_characters == 0 {
            return false;
        }

        // Check for reasonable character diversity
        let diversity_ratio = stats.unique_characters as f64 / stats.total_characters as f64;
        if stats.total_characters > 10 && diversity_ratio < 0.3 {
            return false; // Too repetitive
        }

        true
    }

    /// Score Chinese text based on character frequency and patterns
    pub fn score_chinese_text(text: &str) -> f64 {
        if text.trim().is_empty() {
            return -20.0;
        }

        let stats = Self::get_chinese_stats(text);
        
        if stats.han_characters == 0 {
            return -20.0;
        }

        let mut score = 0.0;

        // Base score for having Han characters
        score += stats.han_ratio * 5.0;

        // Bonus for common characters (absolute count + ratio)
        if stats.han_characters > 0 {
            // Give bonus for having common characters at all
            score += (stats.common_characters.min(4) as f64) * 0.5;
            // Additional ratio-based bonus, but capped to avoid penalizing longer text
            let common_ratio = stats.common_characters as f64 / stats.han_characters as f64;
            score += common_ratio.min(0.8) * 2.0;
        }

        // Length bonus (Chinese text can be very concise, reward longer meaningful text)
        if stats.han_characters >= 2 {
            score += 1.0;
        }
        if stats.han_characters >= 4 {
            score += 0.5; // Additional bonus for longer text
        }

        // Diversity bonus
        if stats.total_characters > 0 {
            let diversity = stats.unique_characters as f64 / stats.total_characters as f64;
            score += diversity * 2.0;
        }

        // Penalty for very short text
        if stats.han_characters < 2 {
            score -= 2.0;
        }

        // Convert to negative scale (consistent with existing trigram scores)
        -5.0 + score
    }
}

/// Statistics about Chinese text content
#[derive(Debug, Clone)]
pub struct ChineseTextStats {
    /// Number of Han characters
    pub han_characters: usize,
    /// Total non-whitespace, non-punctuation characters
    pub total_characters: usize,
    /// Number of unique characters
    pub unique_characters: usize,
    /// Number of common Chinese characters
    pub common_characters: usize,
    /// Ratio of Han characters to total
    pub han_ratio: f64,
}

impl ChineseTextStats {
    /// Check if this represents likely valid Chinese text
    pub fn is_likely_valid(&self) -> bool {
        self.han_characters > 0 && 
        self.han_ratio >= 0.6 &&
        (self.han_characters < 5 || self.common_characters > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_han_character_detection() {
        assert!(ChineseAnalyzer::is_han_character('你'));
        assert!(ChineseAnalyzer::is_han_character('好'));
        assert!(ChineseAnalyzer::is_han_character('世'));
        assert!(ChineseAnalyzer::is_han_character('界'));
        assert!(!ChineseAnalyzer::is_han_character('a'));
        assert!(!ChineseAnalyzer::is_han_character('1'));
    }

    #[test]
    fn test_common_chinese_chars() {
        assert!(ChineseAnalyzer::is_common_chinese_char('的'));
        assert!(ChineseAnalyzer::is_common_chinese_char('一'));
        assert!(ChineseAnalyzer::is_common_chinese_char('好'));
        assert!(!ChineseAnalyzer::is_common_chinese_char('龘')); // Rare character
    }

    #[test]
    fn test_chinese_detection() {
        assert!(ChineseAnalyzer::is_likely_chinese("你好"));
        assert!(ChineseAnalyzer::is_likely_chinese("你好世界"));
        assert!(ChineseAnalyzer::is_likely_chinese("这是中文"));
        assert!(!ChineseAnalyzer::is_likely_chinese("hello"));
        assert!(!ChineseAnalyzer::is_likely_chinese("hello 你")); // Too much English
    }

    #[test]
    fn test_chinese_stats() {
        let stats = ChineseAnalyzer::get_chinese_stats("你好世界");
        assert_eq!(stats.han_characters, 4);
        assert_eq!(stats.total_characters, 4);
        assert_eq!(stats.han_ratio, 1.0);
        assert!(stats.common_characters > 0);
    }

    #[test]
    fn test_chinese_validation() {
        assert!(ChineseAnalyzer::validate_chinese_text("你好"));
        assert!(ChineseAnalyzer::validate_chinese_text("你好世界"));
        assert!(ChineseAnalyzer::validate_chinese_text("这是一个测试"));
        assert!(!ChineseAnalyzer::validate_chinese_text("hello"));
        assert!(!ChineseAnalyzer::validate_chinese_text(""));
    }

    #[test]
    fn test_chinese_scoring() {
        let score1 = ChineseAnalyzer::score_chinese_text("你好");
        let score2 = ChineseAnalyzer::score_chinese_text("hello");
        let score3 = ChineseAnalyzer::score_chinese_text("你好世界");
        
        assert!(score1 > score2); // Chinese should score better than English
        assert!(score3 > score1); // Longer Chinese text should score better
        assert!(score2 < -10.0);  // Non-Chinese should get very low score
    }
}