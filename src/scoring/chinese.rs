//! Chinese character frequency-based scoring
//!
//! This module provides scoring for Chinese (Han script) text using
//! character frequency analysis instead of trigrams.

use super::{StringScorer, ScoringResult};
use crate::language::{ScriptType, chinese::ChineseAnalyzer};

/// Chinese character frequency-based string scorer
pub struct ChineseStringScorer;

impl ChineseStringScorer {
    /// Create a new Chinese scorer
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChineseStringScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl StringScorer for ChineseStringScorer {
    fn score_string(&self, text: &str) -> ScoringResult {
        let score = ChineseAnalyzer::score_chinese_text(text);
        
        // Threshold based on text length and content
        let threshold = if ChineseAnalyzer::is_likely_chinese(text) {
            -3.0 // More lenient threshold for likely Chinese text
        } else {
            10.0 // High threshold for non-Chinese text
        };
        
        ScoringResult::new(
            score,
            threshold,
            ScriptType::Han,
            "Chinese".to_string(),
        )
    }

    fn script_type(&self) -> ScriptType {
        ScriptType::Han
    }

    fn name(&self) -> &'static str {
        "Chinese"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_scorer() {
        let scorer = ChineseStringScorer::new();
        
        assert_eq!(scorer.script_type(), ScriptType::Han);
        assert_eq!(scorer.name(), "Chinese");
        
        let result_chinese = scorer.score_string("你好");
        let result_english = scorer.score_string("hello");
        
        assert!(result_chinese.score > result_english.score);
        assert_eq!(result_chinese.script_type, ScriptType::Han);
        assert_eq!(result_chinese.scorer_name, "Chinese");
        
        // Chinese text should have lower threshold
        assert!(result_chinese.threshold < result_english.threshold);
    }

    #[test]
    fn test_chinese_scorer_validation() {
        let scorer = ChineseStringScorer::new();
        
        let result = scorer.score_string("你好世界");
        assert!(result.is_valid);
        
        let result2 = scorer.score_string("hello");
        assert!(!result2.is_valid);
    }
}