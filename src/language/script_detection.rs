//! Script detection utilities for identifying writing systems
//!
//! This module provides detailed Unicode script analysis capabilities.

use super::{ScriptType, LanguageDetectionResult};
use std::collections::HashMap;
use unicode_script::{Script, UnicodeScript};

/// Extended script detection with detailed analysis
pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    /// Analyze a string and provide detailed script information
    pub fn analyze_detailed(text: &str) -> DetailedScriptAnalysis {
        let mut char_analysis = Vec::new();
        let mut script_counts: HashMap<Script, usize> = HashMap::new();
        let mut total_chars = 0;

        for ch in text.chars() {
            let script = ch.script();
            let char_info = CharacterInfo {
                character: ch,
                script,
                is_letter: ch.is_alphabetic(),
                is_digit: ch.is_numeric(),
                is_punctuation: ch.is_ascii_punctuation(),
                is_whitespace: ch.is_whitespace(),
            };

            char_analysis.push(char_info);
            
            // Only count non-whitespace, non-punctuation characters
            if !ch.is_whitespace() && !ch.is_ascii_punctuation() {
                *script_counts.entry(script).or_insert(0) += 1;
                total_chars += 1;
            }
        }

        DetailedScriptAnalysis {
            text: text.to_string(),
            char_analysis,
            script_counts,
            total_chars,
        }
    }

    /// Check if a character belongs to a specific script family
    pub fn is_script_family(ch: char, script_type: ScriptType) -> bool {
        let script = ch.script();
        match script_type {
            ScriptType::Latin => matches!(script, Script::Latin),
            ScriptType::Han => matches!(script, Script::Han | Script::Hiragana | Script::Katakana),
            ScriptType::Arabic => matches!(script, Script::Arabic),
            ScriptType::Cyrillic => matches!(script, Script::Cyrillic),
            ScriptType::Mixed | ScriptType::Unknown => false,
        }
    }

    /// Get common script families for mixed text analysis
    pub fn get_script_families(text: &str) -> HashMap<ScriptType, usize> {
        let mut family_counts = HashMap::new();

        for ch in text.chars() {
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                continue;
            }

            for script_type in &[ScriptType::Latin, ScriptType::Han, ScriptType::Arabic, ScriptType::Cyrillic] {
                if Self::is_script_family(ch, *script_type) {
                    *family_counts.entry(*script_type).or_insert(0) += 1;
                    break;
                }
            }
        }

        family_counts
    }

    /// Check if text contains significant amounts of a specific script
    pub fn has_significant_script(text: &str, script_type: ScriptType, min_threshold: f64) -> bool {
        let family_counts = Self::get_script_families(text);
        let total: usize = family_counts.values().sum();
        
        if total == 0 {
            return false;
        }

        let script_count = family_counts.get(&script_type).unwrap_or(&0);
        (*script_count as f64 / total as f64) >= min_threshold
    }
}

/// Information about a single character
#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub character: char,
    pub script: Script,
    pub is_letter: bool,
    pub is_digit: bool,
    pub is_punctuation: bool,
    pub is_whitespace: bool,
}

/// Detailed analysis of script composition in text
#[derive(Debug)]
pub struct DetailedScriptAnalysis {
    pub text: String,
    pub char_analysis: Vec<CharacterInfo>,
    pub script_counts: HashMap<Script, usize>,
    pub total_chars: usize,
}

impl DetailedScriptAnalysis {
    /// Get the dominant script
    pub fn dominant_script(&self) -> Option<Script> {
        self.script_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(script, _)| *script)
    }

    /// Get confidence level for the dominant script
    pub fn dominant_confidence(&self) -> f64 {
        if self.total_chars == 0 {
            return 0.0;
        }

        let max_count = self.script_counts
            .values()
            .max()
            .unwrap_or(&0);

        *max_count as f64 / self.total_chars as f64
    }

    /// Check if the text is script-homogeneous
    pub fn is_homogeneous(&self, threshold: f64) -> bool {
        self.dominant_confidence() >= threshold
    }

    /// Get script distribution as percentages
    pub fn script_percentages(&self) -> HashMap<Script, f64> {
        if self.total_chars == 0 {
            return HashMap::new();
        }

        self.script_counts
            .iter()
            .map(|(script, count)| (*script, *count as f64 / self.total_chars as f64))
            .collect()
    }

    /// Convert to simplified LanguageDetectionResult
    pub fn to_language_result(&self) -> LanguageDetectionResult {
        let script_type_counts: HashMap<ScriptType, usize> = self.script_counts
            .iter()
            .map(|(script, count)| (ScriptType::from(*script), *count))
            .collect();

        let dominant_script_type = self.dominant_script()
            .map(ScriptType::from)
            .unwrap_or(ScriptType::Unknown);

        let confidence = self.dominant_confidence();

        LanguageDetectionResult::new(
            dominant_script_type,
            confidence,
            script_type_counts,
            self.total_chars,
        )
    }
}

/// Script-specific validation rules
pub struct ScriptValidator;

impl ScriptValidator {
    /// Validate if text looks like authentic language content
    pub fn validate_script_authenticity(text: &str, script_type: ScriptType) -> bool {
        match script_type {
            ScriptType::Latin => Self::validate_latin(text),
            ScriptType::Han => Self::validate_han(text),
            ScriptType::Arabic => Self::validate_arabic(text),
            ScriptType::Cyrillic => Self::validate_cyrillic(text),
            ScriptType::Mixed => Self::validate_mixed(text),
            ScriptType::Unknown => false,
        }
    }

    fn validate_latin(text: &str) -> bool {
        // Basic Latin validation: should have vowels and consonants
        let vowels = ['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];
        let has_vowel = text.chars().any(|c| vowels.contains(&c));
        let has_consonant = text.chars().any(|c| c.is_ascii_alphabetic() && !vowels.contains(&c));
        
        has_vowel && has_consonant && text.len() >= 2
    }

    fn validate_han(text: &str) -> bool {
        // For Chinese: check if we have actual Han characters
        text.chars().any(|c| matches!(c.script(), Script::Han)) && text.chars().count() >= 1
    }

    fn validate_arabic(text: &str) -> bool {
        // For Arabic: check for Arabic script characters
        text.chars().any(|c| matches!(c.script(), Script::Arabic)) && text.len() >= 2
    }

    fn validate_cyrillic(text: &str) -> bool {
        // For Cyrillic: similar to Latin but with Cyrillic characters
        let cyrillic_vowels = ['а', 'е', 'и', 'о', 'у', 'ы', 'э', 'ю', 'я', 'А', 'Е', 'И', 'О', 'У', 'Ы', 'Э', 'Ю', 'Я'];
        let has_vowel = text.chars().any(|c| cyrillic_vowels.contains(&c));
        let has_consonant = text.chars().any(|c| matches!(c.script(), Script::Cyrillic) && !cyrillic_vowels.contains(&c));
        
        has_vowel && has_consonant && text.len() >= 2
    }

    fn validate_mixed(text: &str) -> bool {
        // For mixed text: should have at least 2 different script families
        let families = ScriptAnalyzer::get_script_families(text);
        families.len() >= 2 && text.len() >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_analysis() {
        let analysis = ScriptAnalyzer::analyze_detailed("Hello 你好");
        
        assert!(analysis.script_counts.len() >= 2);
        assert!(analysis.total_chars > 0);
        assert!(!analysis.is_homogeneous(0.8));
    }

    #[test]
    fn test_script_families() {
        let families = ScriptAnalyzer::get_script_families("Hello 你好 مرحبا Привет");
        
        assert!(families.contains_key(&ScriptType::Latin));
        assert!(families.contains_key(&ScriptType::Han));
        assert!(families.len() >= 2);
    }

    #[test]
    fn test_significant_script_detection() {
        assert!(ScriptAnalyzer::has_significant_script("Hello World", ScriptType::Latin, 0.8));
        assert!(!ScriptAnalyzer::has_significant_script("Hello 你好世界", ScriptType::Latin, 0.8));
    }

    #[test]
    fn test_latin_validation() {
        assert!(ScriptValidator::validate_script_authenticity("hello", ScriptType::Latin));
        assert!(ScriptValidator::validate_script_authenticity("world", ScriptType::Latin));
        assert!(!ScriptValidator::validate_script_authenticity("bcdfg", ScriptType::Latin)); // No vowels
    }

    #[test]
    fn test_han_validation() {
        assert!(ScriptValidator::validate_script_authenticity("你好", ScriptType::Han));
        assert!(!ScriptValidator::validate_script_authenticity("hello", ScriptType::Han));
    }
}