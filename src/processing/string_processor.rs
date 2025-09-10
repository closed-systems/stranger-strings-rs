use regex::Regex;

/// Processed string data for scoring
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedString {
    pub original_string: String,
    pub scored_string: String,
    pub ascii_codes: Vec<u8>,
}

pub struct StringProcessor;

impl StringProcessor {
    /// Process a string for trigram analysis
    /// Applies case conversion, ASCII validation, and space normalization
    pub fn process_string(input: &str, is_lowercase_model: bool) -> ProcessedString {
        let original_string = input.to_string();
        
        // Apply case conversion if needed
        let mut scored_string = if is_lowercase_model {
            input.to_lowercase()
        } else {
            input.to_string()
        };

        // Check if all characters are ASCII and replace non-ASCII with spaces
        if !Self::is_ascii(&scored_string) {
            scored_string = Self::replace_invalid_ascii(&scored_string);
        }

        // Normalize spaces and tabs
        scored_string = Self::normalize_spaces(&scored_string);

        // Convert to ASCII codes
        let ascii_codes = Self::convert_to_ascii_codes(&scored_string);

        ProcessedString {
            original_string,
            scored_string,
            ascii_codes,
        }
    }

    /// Check if string contains only ASCII characters
    fn is_ascii(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii())
    }

    /// Replace non-ASCII characters with spaces
    fn replace_invalid_ascii(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii() { c } else { ' ' })
            .collect()
    }

    /// Normalize spaces and tabs according to the algorithm
    fn normalize_spaces(s: &str) -> String {
        let mut normalized = s.trim().to_string();

        // Collapse consecutive spaces into 1 space
        let space_regex = Regex::new(r" {2,}").unwrap();
        normalized = space_regex.replace_all(&normalized, " ").to_string();

        // Collapse consecutive tabs into 1 tab
        let tab_regex = Regex::new(r"\t{2,}").unwrap();
        normalized = tab_regex.replace_all(&normalized, "\t").to_string();

        normalized
    }

    /// Convert string to ASCII codes
    fn convert_to_ascii_codes(s: &str) -> Vec<u8> {
        s.chars().map(|c| c as u8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_processing() {
        let result = StringProcessor::process_string("hello", false);
        assert_eq!(result.original_string, "hello");
        assert_eq!(result.scored_string, "hello");
        assert_eq!(result.ascii_codes, vec![104u8, 101, 108, 108, 111]); // h, e, l, l, o
    }

    #[test]
    fn test_lowercase_conversion() {
        let result = StringProcessor::process_string("HELLO", true);
        assert_eq!(result.original_string, "HELLO");
        assert_eq!(result.scored_string, "hello");
        assert_eq!(result.ascii_codes, vec![104u8, 101, 108, 108, 111]);
    }

    #[test]
    fn test_no_lowercase_conversion() {
        let result = StringProcessor::process_string("HELLO", false);
        assert_eq!(result.original_string, "HELLO");
        assert_eq!(result.scored_string, "HELLO");
        assert_eq!(result.ascii_codes, vec![72u8, 69, 76, 76, 79]); // H, E, L, L, O
    }

    #[test]
    fn test_non_ascii_replacement() {
        let result = StringProcessor::process_string("héllo", true);
        assert_eq!(result.original_string, "héllo");
        assert_eq!(result.scored_string, "h llo"); // é replaced with space
        assert_eq!(result.ascii_codes, vec![104u8, 32, 108, 108, 111]); // h, space, l, l, o
    }

    #[test]
    fn test_space_normalization() {
        let result = StringProcessor::process_string("  hello    world  ", false);
        assert_eq!(result.scored_string, "hello world");
        
        let result = StringProcessor::process_string("hello  world", false);
        assert_eq!(result.scored_string, "hello world");
        
        let result = StringProcessor::process_string("hello\t\t\tworld", false);
        assert_eq!(result.scored_string, "hello\tworld");
    }

    #[test]
    fn test_mixed_normalization() {
        let result = StringProcessor::process_string("  HÉLLO   WÖRLD  ", true);
        assert_eq!(result.original_string, "  HÉLLO   WÖRLD  ");
        assert_eq!(result.scored_string, "h llo w rld"); // lowercase + non-ASCII replaced + spaces normalized
    }

    #[test]
    fn test_empty_string() {
        let result = StringProcessor::process_string("", false);
        assert_eq!(result.original_string, "");
        assert_eq!(result.scored_string, "");
        assert_eq!(result.ascii_codes, vec![] as Vec<u8>);
    }

    #[test]
    fn test_whitespace_only() {
        let result = StringProcessor::process_string("   \t\t   ", false);
        assert_eq!(result.scored_string, ""); // All whitespace trimmed
        assert_eq!(result.ascii_codes, vec![] as Vec<u8>);
    }

    #[test]
    fn test_tabs_and_spaces_mixed() {
        let result = StringProcessor::process_string("\t  hello \t world \t ", false);
        assert_eq!(result.scored_string, "hello \t world"); // Leading/trailing removed, internal normalized
    }

    #[test]
    fn test_special_characters() {
        let result = StringProcessor::process_string("hello@world.com", false);
        assert_eq!(result.scored_string, "hello@world.com");
        // @ = 64, . = 46
        assert_eq!(result.ascii_codes, vec![104u8, 101, 108, 108, 111, 64, 119, 111, 114, 108, 100, 46, 99, 111, 109]);
    }
}