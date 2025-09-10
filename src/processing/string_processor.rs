
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
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(trimmed.len());
        let mut chars = trimmed.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                // Add one space and skip any consecutive spaces
                result.push(' ');
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            } else if ch == '\t' {
                // Add one tab and skip any consecutive tabs
                result.push('\t');
                while chars.peek() == Some(&'\t') {
                    chars.next();
                }
            } else {
                result.push(ch);
            }
        }
        
        result
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
    fn test_ascii_normalization_for_garbage_prevention() {
        // Test the specific case identified: non-ASCII chars get replaced with spaces
        // This is what allows garbage strings to pass trigram scoring
        
        // Simulate a garbage string like "UNCeñÉ¹ð" that would be extracted from Latin-1
        let garbage_input = "UNCeñÉ¹ð";
        let result = StringProcessor::process_string(garbage_input, false);
        
        // After processing, non-ASCII chars should be replaced with spaces
        // and then normalized, leaving just "UNCe" which looks like valid English
        assert_eq!(result.original_string, "UNCeñÉ¹ð");
        assert_eq!(result.scored_string, "UNCe"); // Non-ASCII replaced with spaces, then normalized
        
        // This demonstrates why we need garbage filtering BEFORE this processing step!
        // The trigram model will see "UNCe" and think it's valid English
    }
    
    #[test]
    fn test_legitimate_non_ascii_normalization() {
        // Test that legitimate text with some non-ASCII chars gets processed reasonably
        let legitimate = "café menu";
        let result = StringProcessor::process_string(legitimate, false);
        
        assert_eq!(result.original_string, "café menu");
        assert_eq!(result.scored_string, "caf menu"); // é becomes space, then normalized
        
        // After normalization: "caf menu" - still recognizable
    }

    #[test]
    fn test_special_characters() {
        let result = StringProcessor::process_string("hello@world.com", false);
        assert_eq!(result.scored_string, "hello@world.com");
        // @ = 64, . = 46
        assert_eq!(result.ascii_codes, vec![104u8, 101, 108, 108, 111, 64, 119, 111, 114, 108, 100, 46, 99, 111, 109]);
    }
}