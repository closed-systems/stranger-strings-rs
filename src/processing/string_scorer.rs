use crate::constants::{DEFAULT_LOG_VALUE, MINIMUM_STRING_LENGTH, get_threshold_for_length};
use crate::model::TrigramModel;
use super::string_processor::ProcessedString;

pub struct StringScorer {
    // We don't store the model directly to avoid lifetime issues
    // Instead, the caller passes the model when needed
}

impl StringScorer {
    pub fn new(_model: &TrigramModel) -> Self {
        Self {}
    }

    /// Score a processed string and return (score, threshold)
    pub fn score_string(&self, processed: &ProcessedString) -> (f64, f64) {
        let ascii_codes = &processed.ascii_codes;
        let length = ascii_codes.len();
        
        let score = if length < MINIMUM_STRING_LENGTH {
            DEFAULT_LOG_VALUE
        } else {
            // This will be calculated properly with the model
            DEFAULT_LOG_VALUE
        };

        let threshold = get_threshold_for_length(length);
        (score, threshold)
    }

    /// Score a string using the provided model
    pub fn score_string_with_model(&self, processed: &ProcessedString, model: &TrigramModel) -> (f64, f64) {
        let ascii_codes = &processed.ascii_codes;
        let length = ascii_codes.len();
        
        let score = if length < MINIMUM_STRING_LENGTH {
            DEFAULT_LOG_VALUE
        } else {
            self.calculate_trigrams(ascii_codes, model)
        };

        let threshold = get_threshold_for_length(length);
        (score, threshold)
    }

    /// Calculate trigram score for a sequence of ASCII codes
    fn calculate_trigrams(&self, ascii_codes: &[u8], model: &TrigramModel) -> f64 {
        let string_length = ascii_codes.len();
        
        // We can't calculate a score for strings less than minimum length
        if string_length < MINIMUM_STRING_LENGTH {
            return DEFAULT_LOG_VALUE;
        }

        let max_ind_ngram = string_length - 3;
        let mut local_likelihood = 0.0;

        // Add beginning of string trigram probability: [^] + first two chars
        local_likelihood += model.get_begin_trigram_prob(ascii_codes[0], ascii_codes[1]);

        // Add all middle trigram probabilities
        let mut char_index = 1;
        while char_index <= max_ind_ngram {
            local_likelihood += model.get_trigram_prob(
                ascii_codes[char_index],
                ascii_codes[char_index + 1],
                ascii_codes[char_index + 2],
            );
            char_index += 1;
        }

        // Add end of string trigram probability: last two chars + [$]
        local_likelihood += model.get_end_trigram_prob(
            ascii_codes[char_index],
            ascii_codes[char_index + 1],
        );

        // Return average log probability per character
        local_likelihood / string_length as f64
    }

    /// Get the minimum string length for scoring
    pub fn get_minimum_string_length() -> usize {
        MINIMUM_STRING_LENGTH
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{TrigramModel, TrigramCounts};
    use crate::processing::StringProcessor;

    fn create_test_model() -> TrigramModel {
        let mut model = TrigramModel::new();
        let mut counts = TrigramCounts::new();

        // Add some test trigram data for "hello"
        // [^] h e
        counts.begin_trigram_counts[b'h' as usize][b'e' as usize] = 10;
        // h e l
        counts.trigram_counts[b'h' as usize][b'e' as usize][b'l' as usize] = 15;
        // e l l
        counts.trigram_counts[b'e' as usize][b'l' as usize][b'l' as usize] = 20;
        // l l o
        counts.trigram_counts[b'l' as usize][b'l' as usize][b'o' as usize] = 25;
        // l o [$]
        counts.end_trigram_counts[b'l' as usize][b'o' as usize] = 8;

        counts.total_trigrams = 78;
        model.load_from_counts(counts, "test".to_string());
        model
    }

    #[test]
    fn test_string_scorer_creation() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        assert_eq!(StringScorer::get_minimum_string_length(), MINIMUM_STRING_LENGTH);
    }

    #[test]
    fn test_short_string_scoring() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        let processed = StringProcessor::process_string("hi", false);
        
        let (score, threshold) = scorer.score_string_with_model(&processed, &model);
        assert_eq!(score, DEFAULT_LOG_VALUE); // Too short to score
        assert_eq!(threshold, get_threshold_for_length(2));
    }

    #[test]
    fn test_valid_string_scoring() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        let processed = StringProcessor::process_string("hello", false);
        
        let (score, threshold) = scorer.score_string_with_model(&processed, &model);
        
        // Score should be negative (log10 of probabilities < 1)
        assert!(score < 0.0);
        assert!(score > DEFAULT_LOG_VALUE); // Should be better than default
        assert_eq!(threshold, get_threshold_for_length(5));
    }

    #[test]
    fn test_threshold_calculation() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        
        // Test various string lengths
        let test_cases = vec![
            ("abc", 3, get_threshold_for_length(3)),
            ("abcd", 4, get_threshold_for_length(4)),
            ("hello", 5, get_threshold_for_length(5)),
            ("verylongstring", 14, get_threshold_for_length(14)),
        ];

        for (text, expected_len, expected_threshold) in test_cases {
            let processed = StringProcessor::process_string(text, false);
            let (_, threshold) = scorer.score_string_with_model(&processed, &model);
            
            assert_eq!(processed.ascii_codes.len(), expected_len);
            assert_eq!(threshold, expected_threshold);
        }
    }

    #[test]
    fn test_trigram_calculation() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        
        // Test with a string that has known trigrams in our test model
        let processed = StringProcessor::process_string("hello", false);
        let score = scorer.calculate_trigrams(&processed.ascii_codes, &model);
        
        // The score should be the average of:
        // - begin trigram: [^] h e
        // - trigram: h e l
        // - trigram: e l l  
        // - trigram: l l o
        // - end trigram: l o [$]
        // All divided by string length (5)
        
        assert!(score < 0.0); // Log probabilities are negative
        assert!(score > DEFAULT_LOG_VALUE); // Should be better than default
    }

    #[test]
    fn test_unknown_string_scoring() {
        let model = create_test_model();
        let scorer = StringScorer::new(&model);
        
        // Test with a string that has no trigrams in our model
        let processed = StringProcessor::process_string("xyz", false);
        let (score, _) = scorer.score_string_with_model(&processed, &model);
        
        // Should still get a score due to Laplace smoothing
        assert!(score < 0.0);
        assert!(score != DEFAULT_LOG_VALUE);
    }
}