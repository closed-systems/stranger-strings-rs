use crate::constants::ASCII_CHAR_COUNT;

/// Counts for trigrams during model loading
#[derive(Debug, Clone)]
pub struct TrigramCounts {
    pub trigram_counts: Vec<Vec<Vec<u32>>>,
    pub begin_trigram_counts: Vec<Vec<u32>>,
    pub end_trigram_counts: Vec<Vec<u32>>,
    pub total_trigrams: u32,
}

impl TrigramCounts {
    pub fn new() -> Self {
        Self {
            trigram_counts: create_3d_array(),
            begin_trigram_counts: create_2d_array(),
            end_trigram_counts: create_2d_array(),
            total_trigrams: 0,
        }
    }
}

/// Trigram model storing log probabilities for string scoring
#[derive(Debug, Clone)]
pub struct TrigramModel {
    trigram_probs: Vec<Vec<Vec<f64>>>,
    begin_trigram_probs: Vec<Vec<f64>>,
    end_trigram_probs: Vec<Vec<f64>>,
    model_type: String,
    is_lowercase: bool,
}

impl TrigramModel {
    pub fn new() -> Self {
        Self {
            trigram_probs: create_3d_array_f64(),
            begin_trigram_probs: create_2d_array_f64(),
            end_trigram_probs: create_2d_array_f64(),
            model_type: String::new(),
            is_lowercase: false,
        }
    }

    /// Load model from trigram counts and apply smoothing
    pub fn load_from_counts(&mut self, counts: TrigramCounts, model_type: String) {
        self.model_type = model_type.clone();
        self.is_lowercase = model_type == "lowercase";

        // Create copies for smoothing
        let mut trigram_counts = counts.trigram_counts;
        let mut begin_trigram_counts = counts.begin_trigram_counts;
        let mut end_trigram_counts = counts.end_trigram_counts;
        let mut total_trigrams = counts.total_trigrams;

        // Apply Laplace smoothing (add 1 to all zero counts)
        for i in 0..ASCII_CHAR_COUNT {
            for j in 0..ASCII_CHAR_COUNT {
                if begin_trigram_counts[i][j] == 0 {
                    begin_trigram_counts[i][j] = 1;
                    total_trigrams += 1;
                }

                if end_trigram_counts[i][j] == 0 {
                    end_trigram_counts[i][j] = 1;
                    total_trigrams += 1;
                }

                for k in 0..ASCII_CHAR_COUNT {
                    if trigram_counts[i][j][k] == 0 {
                        trigram_counts[i][j][k] = 1;
                        total_trigrams += 1;
                    }
                }
            }
        }

        // Calculate log probabilities (base 10)
        let total_f64 = total_trigrams as f64;
        for i in 0..ASCII_CHAR_COUNT {
            for j in 0..ASCII_CHAR_COUNT {
                self.begin_trigram_probs[i][j] = (begin_trigram_counts[i][j] as f64 / total_f64).log10();
                self.end_trigram_probs[i][j] = (end_trigram_counts[i][j] as f64 / total_f64).log10();

                for k in 0..ASCII_CHAR_COUNT {
                    self.trigram_probs[i][j][k] = (trigram_counts[i][j][k] as f64 / total_f64).log10();
                }
            }
        }
    }

    /// Get trigram probability for three characters
    pub fn get_trigram_prob(&self, char1: u8, char2: u8, char3: u8) -> f64 {
        self.trigram_probs[char1 as usize][char2 as usize][char3 as usize]
    }

    /// Get beginning trigram probability for two characters
    pub fn get_begin_trigram_prob(&self, char1: u8, char2: u8) -> f64 {
        self.begin_trigram_probs[char1 as usize][char2 as usize]
    }

    /// Get ending trigram probability for two characters
    pub fn get_end_trigram_prob(&self, char1: u8, char2: u8) -> f64 {
        self.end_trigram_probs[char1 as usize][char2 as usize]
    }

    /// Get the model type
    pub fn get_model_type(&self) -> &str {
        &self.model_type
    }

    /// Check if this is a lowercase model
    pub fn is_lowercase_model(&self) -> bool {
        self.is_lowercase
    }
}

impl Default for TrigramModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a 3D array for trigram counts
fn create_3d_array() -> Vec<Vec<Vec<u32>>> {
    let mut arr = Vec::with_capacity(ASCII_CHAR_COUNT);
    for _ in 0..ASCII_CHAR_COUNT {
        let mut inner = Vec::with_capacity(ASCII_CHAR_COUNT);
        for _ in 0..ASCII_CHAR_COUNT {
            inner.push(vec![0u32; ASCII_CHAR_COUNT]);
        }
        arr.push(inner);
    }
    arr
}

/// Create a 2D array for bigram counts
fn create_2d_array() -> Vec<Vec<u32>> {
    let mut arr = Vec::with_capacity(ASCII_CHAR_COUNT);
    for _ in 0..ASCII_CHAR_COUNT {
        arr.push(vec![0u32; ASCII_CHAR_COUNT]);
    }
    arr
}

/// Create a 3D array for trigram probabilities
fn create_3d_array_f64() -> Vec<Vec<Vec<f64>>> {
    let mut arr = Vec::with_capacity(ASCII_CHAR_COUNT);
    for _ in 0..ASCII_CHAR_COUNT {
        let mut inner = Vec::with_capacity(ASCII_CHAR_COUNT);
        for _ in 0..ASCII_CHAR_COUNT {
            inner.push(vec![0.0f64; ASCII_CHAR_COUNT]);
        }
        arr.push(inner);
    }
    arr
}

/// Create a 2D array for bigram probabilities
fn create_2d_array_f64() -> Vec<Vec<f64>> {
    let mut arr = Vec::with_capacity(ASCII_CHAR_COUNT);
    for _ in 0..ASCII_CHAR_COUNT {
        arr.push(vec![0.0f64; ASCII_CHAR_COUNT]);
    }
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigram_model_creation() {
        let model = TrigramModel::new();
        assert_eq!(model.get_model_type(), "");
        assert!(!model.is_lowercase_model());
    }

    #[test]
    fn test_trigram_counts_creation() {
        let counts = TrigramCounts::new();
        assert_eq!(counts.total_trigrams, 0);
        assert_eq!(counts.trigram_counts.len(), ASCII_CHAR_COUNT);
        assert_eq!(counts.begin_trigram_counts.len(), ASCII_CHAR_COUNT);
        assert_eq!(counts.end_trigram_counts.len(), ASCII_CHAR_COUNT);
    }

    #[test]
    fn test_model_loading_with_smoothing() {
        let mut model = TrigramModel::new();
        let mut counts = TrigramCounts::new();
        
        // Add some test data
        counts.trigram_counts[65][66][67] = 10; // 'A', 'B', 'C'
        counts.begin_trigram_counts[65][66] = 5; // 'A', 'B'
        counts.end_trigram_counts[66][67] = 3; // 'B', 'C'
        counts.total_trigrams = 18;

        model.load_from_counts(counts, "lowercase".to_string());

        assert_eq!(model.get_model_type(), "lowercase");
        assert!(model.is_lowercase_model());

        // Test that probabilities are negative (log10 of values < 1)
        assert!(model.get_trigram_prob(65, 66, 67) < 0.0);
        assert!(model.get_begin_trigram_prob(65, 66) < 0.0);
        assert!(model.get_end_trigram_prob(66, 67) < 0.0);

        // Test smoothing: previously zero entries should have small negative values
        assert!(model.get_trigram_prob(0, 0, 0) < 0.0);
        assert!(model.get_begin_trigram_prob(0, 0) < 0.0);
        assert!(model.get_end_trigram_prob(0, 0) < 0.0);
    }

    #[test]
    fn test_model_type_detection() {
        let mut model = TrigramModel::new();
        let counts = TrigramCounts::new();
        
        model.load_from_counts(counts.clone(), "lowercase".to_string());
        assert!(model.is_lowercase_model());
        
        model.load_from_counts(counts.clone(), "mixed".to_string());
        assert!(!model.is_lowercase_model());
        
        model.load_from_counts(counts, "Lowercase".to_string());
        assert!(!model.is_lowercase_model()); // Only exact "lowercase" match
    }
}