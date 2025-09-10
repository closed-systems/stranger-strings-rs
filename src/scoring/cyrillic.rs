//! Cyrillic script scoring adapted from trigram approach
//!
//! This module provides scoring for Cyrillic (Russian) text using
//! adapted trigram patterns and character frequency analysis.

use super::{StringScorer, ScoringResult};
use crate::language::{ScriptType, cyrillic::CyrillicAnalyzer};

/// Cyrillic character and pattern-based string scorer
pub struct CyrillicStringScorer;

impl CyrillicStringScorer {
    /// Create a new Cyrillic scorer
    pub fn new() -> Self {
        Self
    }
}

impl Default for CyrillicStringScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl StringScorer for CyrillicStringScorer {
    fn score_string(&self, text: &str) -> ScoringResult {
        let score = CyrillicAnalyzer::score_cyrillic_text(text);
        
        // Threshold based on text length and content
        let threshold = if CyrillicAnalyzer::is_likely_cyrillic(text) {
            -3.0 // More lenient threshold for likely Cyrillic text
        } else {
            10.0 // High threshold for non-Cyrillic text
        };
        
        ScoringResult::new(
            score,
            threshold,
            ScriptType::Cyrillic,
            "Cyrillic".to_string(),
        )
    }

    fn script_type(&self) -> ScriptType {
        ScriptType::Cyrillic
    }

    fn name(&self) -> &'static str {
        "Cyrillic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyrillic_scorer() {
        let scorer = CyrillicStringScorer::new();
        
        assert_eq!(scorer.script_type(), ScriptType::Cyrillic);
        assert_eq!(scorer.name(), "Cyrillic");
        
        let result_cyrillic = scorer.score_string("привет");
        let result_english = scorer.score_string("hello");
        
        assert!(result_cyrillic.score > result_english.score);
        assert_eq!(result_cyrillic.script_type, ScriptType::Cyrillic);
        assert_eq!(result_cyrillic.scorer_name, "Cyrillic");
        
        // Cyrillic text should have lower threshold
        assert!(result_cyrillic.threshold < result_english.threshold);
    }

    #[test]
    fn test_cyrillic_scorer_validation() {
        let scorer = CyrillicStringScorer::new();
        
        let result = scorer.score_string("привет");
        assert!(result.is_valid);
        
        let result2 = scorer.score_string("hello");
        assert!(!result2.is_valid);
    }
}