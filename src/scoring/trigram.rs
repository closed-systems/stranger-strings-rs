//! Trigram-based scoring for Latin script text
//!
//! This module wraps the existing trigram scoring system in the new
//! trait-based architecture.

use super::{ScoringResult, StringScorer};
use crate::language::ScriptType;
use crate::model::TrigramModel;
use crate::processing::{StringProcessor, StringScorer as LegacyStringScorer};
use std::sync::Arc;

/// Trigram-based string scorer for Latin script
pub struct TrigramStringScorer {
    model: Arc<TrigramModel>,
    legacy_scorer: LegacyStringScorer,
}

impl TrigramStringScorer {
    /// Create a new trigram scorer with the given model
    pub fn new(model: Arc<TrigramModel>) -> Self {
        let legacy_scorer = LegacyStringScorer::new(&model);
        Self {
            model,
            legacy_scorer,
        }
    }

    /// Get the underlying trigram model
    pub fn model(&self) -> &TrigramModel {
        &self.model
    }
}

impl StringScorer for TrigramStringScorer {
    fn score_string(&self, text: &str) -> ScoringResult {
        // Use the existing processing and scoring logic
        let processed = StringProcessor::process_string(text, self.model.is_lowercase_model());
        let (score, threshold) = self
            .legacy_scorer
            .score_string_with_model(&processed, &self.model);

        ScoringResult::new(score, threshold, ScriptType::Latin, "Trigram".to_string())
    }

    fn script_type(&self) -> ScriptType {
        ScriptType::Latin
    }

    fn name(&self) -> &'static str {
        "Trigram"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TrigramCounts;

    fn create_test_model() -> TrigramModel {
        let counts = TrigramCounts::new();
        let mut model = TrigramModel::new();
        model.load_from_counts(counts, "lowercase".to_string());
        model
    }

    #[test]
    fn test_trigram_scorer() {
        let model = create_test_model();
        let scorer = TrigramStringScorer::new(Arc::new(model));

        assert_eq!(scorer.script_type(), ScriptType::Latin);
        assert_eq!(scorer.name(), "Trigram");

        let result = scorer.score_string("the");
        assert_eq!(result.script_type, ScriptType::Latin);
        assert_eq!(result.scorer_name, "Trigram");
        // Length 3 string should have threshold from NG_THRESHOLDS[3] = 10.0 (invalid)
        assert!(result.threshold > 0.0); // Short strings get high thresholds (impossible to pass)
    }

    #[test]
    fn test_trigram_scorer_with_real_text() {
        let model = create_test_model();
        let scorer = TrigramStringScorer::new(Arc::new(model));

        let result1 = scorer.score_string("testing");
        let result2 = scorer.score_string("xyzabc");

        // Results should be reasonable (though not necessarily valid without a proper model)
        assert!(result1.score < 0.0);
        assert!(result2.score < 0.0);
        assert_eq!(result1.script_type, ScriptType::Latin);
        assert_eq!(result2.script_type, ScriptType::Latin);
    }
}
