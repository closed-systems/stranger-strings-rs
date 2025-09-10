//! Arabic script scoring with RTL support
//!
//! This module provides scoring for Arabic text using character frequency
//! analysis and RTL text patterns.

use super::{StringScorer, ScoringResult};
use crate::language::{ScriptType, arabic::ArabicAnalyzer};

/// Arabic character frequency-based string scorer
pub struct ArabicStringScorer;

impl ArabicStringScorer {
    /// Create a new Arabic scorer
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArabicStringScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl StringScorer for ArabicStringScorer {
    fn score_string(&self, text: &str) -> ScoringResult {
        let score = ArabicAnalyzer::score_arabic_text(text);
        
        // Threshold based on text length and content
        let threshold = if ArabicAnalyzer::is_likely_arabic(text) {
            -3.0 // More lenient threshold for likely Arabic text
        } else {
            10.0 // High threshold for non-Arabic text
        };
        
        ScoringResult::new(
            score,
            threshold,
            ScriptType::Arabic,
            "Arabic".to_string(),
        )
    }

    fn script_type(&self) -> ScriptType {
        ScriptType::Arabic
    }

    fn name(&self) -> &'static str {
        "Arabic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_scorer() {
        let scorer = ArabicStringScorer::new();
        
        assert_eq!(scorer.script_type(), ScriptType::Arabic);
        assert_eq!(scorer.name(), "Arabic");
        
        let result_arabic = scorer.score_string("مرحبا");
        let result_english = scorer.score_string("hello");
        
        assert!(result_arabic.score > result_english.score);
        assert_eq!(result_arabic.script_type, ScriptType::Arabic);
        assert_eq!(result_arabic.scorer_name, "Arabic");
        
        // Arabic text should have lower threshold
        assert!(result_arabic.threshold < result_english.threshold);
    }

    #[test]
    fn test_arabic_scorer_validation() {
        let scorer = ArabicStringScorer::new();
        
        let result = scorer.score_string("السلام");
        assert!(result.is_valid);
        
        let result2 = scorer.score_string("hello");
        assert!(!result2.is_valid);
    }
}