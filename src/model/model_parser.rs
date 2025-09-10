use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::constants::{create_description_to_ascii_map, BEGIN_MARKER, END_MARKER};
use crate::error::{StrangerError, Result};
use super::trigram_model::{TrigramCounts, TrigramModel};

pub struct ModelParser;

impl ModelParser {
    const MODEL_TYPE_PREFIX: &'static str = "Model Type: ";

    /// Parse a .sng model file from disk
    pub fn parse_model_file(file_path: &Path) -> Result<TrigramModel> {
        let file = fs::File::open(file_path)?;
        let reader = BufReader::new(file);
        Self::parse_model_reader(reader)
    }

    /// Parse a .sng model from string content
    pub fn parse_model_string(content: &str) -> Result<TrigramModel> {
        let reader = std::io::Cursor::new(content);
        Self::parse_model_reader(BufReader::new(reader))
    }

    /// Parse model from any BufRead source
    fn parse_model_reader<R: BufRead>(reader: R) -> Result<TrigramModel> {
        let mut counts = TrigramCounts::new();
        let mut model_type = String::new();
        let description_map = create_description_to_ascii_map();

        for line in reader.lines() {
            let line = line?;
            
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse comment lines for model type
            if line.starts_with('#') {
                if line.contains(Self::MODEL_TYPE_PREFIX) {
                    if let Some(prefix_index) = line.find(Self::MODEL_TYPE_PREFIX) {
                        model_type = line[prefix_index + Self::MODEL_TYPE_PREFIX.len()..].to_string();
                    }
                }
                continue;
            }

            // Parse data lines (tab-delimited)
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 4 {
                return Err(StrangerError::ModelParsing(
                    format!("Invalid line format: {}", line)
                ));
            }

            let count = parts[3].parse::<u32>()
                .map_err(|_| StrangerError::ModelParsing(
                    format!("Invalid count in line: {}", line)
                ))?;

            let chars = Self::convert_to_ascii_codes(&parts[0..3], &description_map)?;

            // Process based on trigram type
            if chars[0] == BEGIN_MARKER {
                // Beginning of string trigram: [^] + char1 + char2
                if chars[2] != END_MARKER {
                    let char1 = Self::marker_to_ascii(&chars[1], &description_map)?;
                    let char2 = Self::marker_to_ascii(&chars[2], &description_map)?;
                    counts.begin_trigram_counts[char1 as usize][char2 as usize] += count;
                }
            } else if chars[2] == END_MARKER {
                // End of string trigram: char1 + char2 + [$]
                let char1 = Self::marker_to_ascii(&chars[0], &description_map)?;
                let char2 = Self::marker_to_ascii(&chars[1], &description_map)?;
                counts.end_trigram_counts[char1 as usize][char2 as usize] += count;
            } else {
                // Regular trigram: char1 + char2 + char3
                let char1 = Self::marker_to_ascii(&chars[0], &description_map)?;
                let char2 = Self::marker_to_ascii(&chars[1], &description_map)?;
                let char3 = Self::marker_to_ascii(&chars[2], &description_map)?;
                counts.trigram_counts[char1 as usize][char2 as usize][char3 as usize] += count;
            }

            counts.total_trigrams += count;
        }

        if model_type.is_empty() {
            return Err(StrangerError::ModelParsing(
                "Model file does not contain model type".to_string()
            ));
        }

        let mut model = TrigramModel::new();
        model.load_from_counts(counts, model_type);
        Ok(model)
    }

    /// Convert parts of a line to character representations
    fn convert_to_ascii_codes(parts: &[&str], description_map: &HashMap<String, u8>) -> Result<Vec<String>> {
        let mut result = Vec::new();
        
        for part in parts {
            if *part == BEGIN_MARKER || *part == END_MARKER {
                result.push(part.to_string());
            } else if part.len() > 1 {
                // Special character representation
                if description_map.contains_key(*part) {
                    result.push(part.to_string());
                } else {
                    return Err(StrangerError::ModelParsing(
                        format!("Unknown character representation: {}", part)
                    ));
                }
            } else {
                // Regular character
                result.push(part.to_string());
            }
        }
        
        Ok(result)
    }

    /// Convert a marker string to ASCII code
    fn marker_to_ascii(marker: &str, description_map: &HashMap<String, u8>) -> Result<u8> {
        if marker == BEGIN_MARKER || marker == END_MARKER {
            return Err(StrangerError::ModelParsing(
                format!("Unexpected marker in ASCII position: {}", marker)
            ));
        }

        if marker.len() == 1 {
            // Regular character
            Ok(marker.chars().next().unwrap() as u8)
        } else {
            // Special character description
            description_map.get(marker)
                .copied()
                .ok_or_else(|| StrangerError::ModelParsing(
                    format!("Unknown character description: {}", marker)
                ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_simple_model() {
        let model_content = r#"# Model Type: lowercase
# Test model
[^]	h	e	10
h	e	l	20
l	l	o	15
l	o	[$]	5
"#;

        let model = ModelParser::parse_model_string(model_content).unwrap();
        assert_eq!(model.get_model_type(), "lowercase");
        assert!(model.is_lowercase_model());

        // Check that probabilities are computed (should be negative log values)
        assert!(model.get_begin_trigram_prob(b'h', b'e') < 0.0);
        assert!(model.get_trigram_prob(b'h', b'e', b'l') < 0.0);
        assert!(model.get_end_trigram_prob(b'l', b'o') < 0.0);
    }

    #[test]
    fn test_parse_model_with_special_characters() {
        let model_content = r#"# Model Type: mixed
[^]	[SP]	a	5
[HT]	[SP]	[SP]	3
a	b	[$]	7
"#;

        let model = ModelParser::parse_model_string(model_content).unwrap();
        assert_eq!(model.get_model_type(), "mixed");
        assert!(!model.is_lowercase_model());

        // Check that special characters are handled
        assert!(model.get_begin_trigram_prob(32, b'a') < 0.0); // [SP] = ASCII 32
        assert!(model.get_trigram_prob(9, 32, 32) < 0.0); // [HT] = ASCII 9, [SP] = ASCII 32
    }

    #[test]
    fn test_parse_model_file() {
        let model_content = r#"# Model Type: test
[^]	t	e	1
t	e	s	2
s	t	[$]	1
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", model_content).unwrap();

        let model = ModelParser::parse_model_file(temp_file.path()).unwrap();
        assert_eq!(model.get_model_type(), "test");
    }

    #[test]
    fn test_invalid_model_format() {
        let model_content = r#"# Model Type: test
invalid	line
"#;

        let result = ModelParser::parse_model_string(model_content);
        assert!(result.is_err());
        if let Err(StrangerError::ModelParsing(msg)) = result {
            assert!(msg.contains("Invalid line format"));
        } else {
            panic!("Expected ModelParsing error");
        }
    }

    #[test]
    fn test_missing_model_type() {
        let model_content = r#"# No model type specified
[^]	h	e	1
"#;

        let result = ModelParser::parse_model_string(model_content);
        assert!(result.is_err());
        if let Err(StrangerError::ModelParsing(msg)) = result {
            assert!(msg.contains("does not contain model type"));
        } else {
            panic!("Expected ModelParsing error");
        }
    }

    #[test]
    fn test_invalid_count() {
        let model_content = r#"# Model Type: test
[^]	h	e	not_a_number
"#;

        let result = ModelParser::parse_model_string(model_content);
        assert!(result.is_err());
        if let Err(StrangerError::ModelParsing(msg)) = result {
            assert!(msg.contains("Invalid count"));
        } else {
            panic!("Expected ModelParsing error");
        }
    }

    #[test]
    fn test_unknown_character_representation() {
        let model_content = r#"# Model Type: test
[^]	[UNKNOWN]	e	1
"#;

        let result = ModelParser::parse_model_string(model_content);
        assert!(result.is_err());
        if let Err(StrangerError::ModelParsing(msg)) = result {
            assert!(msg.contains("Unknown character representation"));
        } else {
            panic!("Expected ModelParsing error");
        }
    }
}