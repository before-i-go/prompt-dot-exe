//! Hexadecimal token generation
//!
//! Generates sequential hexadecimal tokens (A0, A1, A2... AA, AB...) for pattern replacement.
//! Uses idiomatic Rust patterns including Iterator trait, Option/Result types, and compile-time guarantees.

use crate::compression::{CompressionError, TokenGeneration};
use std::fmt;

/// Newtype for hex tokens with compile-time guarantees
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexToken(String);

impl HexToken {
    /// Create a new hex token with validation
    pub fn new(token: String) -> Option<Self> {
        if Self::is_valid_hex_token(&token) {
            Some(Self(token))
        } else {
            None
        }
    }

    /// Get the token string
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate hex token format
    fn is_valid_hex_token(token: &str) -> bool {
        // New format: T0000, T0001, etc. (5 characters: T + 4 hex digits)
        token.len() == 5
            && token.starts_with('T')
            && token
                .chars()
                .skip(1)
                .all(|c| c.is_ascii_hexdigit() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
    }
}

impl fmt::Display for HexToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for HexToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Generator for sequential hexadecimal tokens
///
/// Generates tokens in sequence: A0, A1, A2... A9, AA, AB... AZ, B0, B1...
/// Uses Iterator trait and Option/Result types for idiomatic Rust patterns.
#[derive(Debug, Clone)]
pub struct HexTokenGenerator {
    current_token: u32,
    max_tokens: u32,
}

impl HexTokenGenerator {
    /// Maximum number of tokens that can be generated (16^4 - 1)
    const MAX_TOKENS: u32 = 65535; // 0xFFFF

    /// Create a new token generator
    pub fn new() -> Self {
        Self {
            current_token: 0,
            max_tokens: Self::MAX_TOKENS,
        }
    }

    /// Create a token generator with custom maximum
    #[allow(dead_code)]
    pub fn with_max_tokens(max_tokens: u32) -> Result<Self, CompressionError> {
        if max_tokens == 0 {
            return Err(CompressionError::config_validation(
                "Max tokens must be greater than 0",
            ));
        }
        if max_tokens > Self::MAX_TOKENS {
            return Err(CompressionError::config_validation(format!(
                "Max tokens cannot exceed {}",
                Self::MAX_TOKENS
            )));
        }

        Ok(Self {
            current_token: 0,
            max_tokens,
        })
    }

    /// Format a token value as hexadecimal string
    ///
    /// Uses collision-free sequential hex tokens: T0000, T0001, T0002... TFFFF
    /// The 'T' prefix ensures no collision with actual code patterns.
    fn format_token(value: u32) -> String {
        format!("T{:04X}", value)
    }

    /// Check if more tokens are available
    pub fn has_next(&self) -> bool {
        self.current_token < self.max_tokens
    }

    /// Get the current token count
    #[allow(dead_code)]
    pub fn token_count(&self) -> u32 {
        self.current_token
    }

    /// Get remaining token capacity
    pub fn remaining_capacity(&self) -> u32 {
        self.max_tokens.saturating_sub(self.current_token)
    }

    /// Reset the token generator to start from the beginning
    pub fn reset(&mut self) {
        self.current_token = 0;
    }
}

impl Default for HexTokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenGeneration for HexTokenGenerator {
    fn next_token(&mut self) -> Result<String, CompressionError> {
        if self.current_token >= self.max_tokens {
            return Err(CompressionError::TokenOverflow);
        }

        let token_str = Self::format_token(self.current_token);
        self.current_token += 1;

        Ok(token_str)
    }

    fn reset(&mut self) {
        self.current_token = 0;
    }
}

impl Iterator for HexTokenGenerator {
    type Item = Result<HexToken, CompressionError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next() {
            match self.next_token() {
                Ok(token_str) => {
                    // This should always succeed since format_token produces valid tokens
                    HexToken::new(token_str)
                        .map(Ok)
                        .or_else(|| Some(Err(CompressionError::TokenOverflow)))
                }
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining_capacity() as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for HexTokenGenerator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_sequence() {
        let mut generator = HexTokenGenerator::new();

        // Test first few tokens follow expected collision-free sequence
        assert_eq!(generator.next_token().unwrap(), "T0000");
        assert_eq!(generator.next_token().unwrap(), "T0001");
        assert_eq!(generator.next_token().unwrap(), "T0002");
        assert_eq!(generator.next_token().unwrap(), "T0003");
        assert_eq!(generator.next_token().unwrap(), "T0004");
        assert_eq!(generator.next_token().unwrap(), "T0005");
        assert_eq!(generator.next_token().unwrap(), "T0006");
        assert_eq!(generator.next_token().unwrap(), "T0007");
        assert_eq!(generator.next_token().unwrap(), "T0008");
        assert_eq!(generator.next_token().unwrap(), "T0009");
        assert_eq!(generator.next_token().unwrap(), "T000A");
        assert_eq!(generator.next_token().unwrap(), "T000B");
        assert_eq!(generator.next_token().unwrap(), "T000C");
        assert_eq!(generator.next_token().unwrap(), "T000D");
        assert_eq!(generator.next_token().unwrap(), "T000E");
        assert_eq!(generator.next_token().unwrap(), "T000F");
        assert_eq!(generator.next_token().unwrap(), "T0010");
    }

    #[test]
    fn test_token_format_validation() {
        // Test format_token produces collision-free tokens with T prefix
        assert_eq!(HexTokenGenerator::format_token(0), "T0000");
        assert_eq!(HexTokenGenerator::format_token(1), "T0001");
        assert_eq!(HexTokenGenerator::format_token(15), "T000F");
        assert_eq!(HexTokenGenerator::format_token(16), "T0010");
        assert_eq!(HexTokenGenerator::format_token(255), "T00FF");
        assert_eq!(HexTokenGenerator::format_token(256), "T0100");
        assert_eq!(HexTokenGenerator::format_token(4095), "T0FFF");
        assert_eq!(HexTokenGenerator::format_token(65535), "TFFFF");
    }

    #[test]
    fn test_token_overflow_handling() {
        let mut generator = HexTokenGenerator::with_max_tokens(2).unwrap();

        // Should generate 2 tokens successfully
        assert!(generator.next_token().is_ok());
        assert!(generator.next_token().is_ok());

        // Third token should overflow
        assert!(matches!(
            generator.next_token(),
            Err(CompressionError::TokenOverflow)
        ));
    }

    #[test]
    fn test_generator_state_tracking() {
        let mut generator = HexTokenGenerator::with_max_tokens(5).unwrap();

        assert_eq!(generator.token_count(), 0);
        assert_eq!(generator.remaining_capacity(), 5);
        assert!(generator.has_next());

        generator.next_token().unwrap();
        assert_eq!(generator.token_count(), 1);
        assert_eq!(generator.remaining_capacity(), 4);
        assert!(generator.has_next());

        // Generate remaining tokens
        for _ in 0..4 {
            generator.next_token().unwrap();
        }

        assert_eq!(generator.token_count(), 5);
        assert_eq!(generator.remaining_capacity(), 0);
        assert!(!generator.has_next());
    }

    #[test]
    fn test_generator_reset() {
        let mut generator = HexTokenGenerator::new();

        // Generate some tokens
        generator.next_token().unwrap();
        generator.next_token().unwrap();
        assert_eq!(generator.token_count(), 2);

        // Reset should restore initial state
        generator.reset();
        assert_eq!(generator.token_count(), 0);
        assert_eq!(generator.next_token().unwrap(), "T0000");
    }

    #[test]
    fn test_iterator_implementation() {
        let generator = HexTokenGenerator::with_max_tokens(3).unwrap();

        let tokens: Result<Vec<_>, _> = generator.collect();
        let tokens = tokens.unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].as_str(), "T0000");
        assert_eq!(tokens[1].as_str(), "T0001");
        assert_eq!(tokens[2].as_str(), "T0002");
    }

    #[test]
    fn test_iterator_size_hint() {
        let generator = HexTokenGenerator::with_max_tokens(10).unwrap();
        let (lower, upper) = generator.size_hint();

        assert_eq!(lower, 10);
        assert_eq!(upper, Some(10));
    }

    #[test]
    fn test_hex_token_newtype() {
        // Valid tokens (new format: T + 4 hex digits)
        assert!(HexToken::new("T0000".to_string()).is_some());
        assert!(HexToken::new("T00FF".to_string()).is_some());
        assert!(HexToken::new("TABCD".to_string()).is_some());
        assert!(HexToken::new("TFFFF".to_string()).is_some());

        // Invalid tokens
        assert!(HexToken::new("".to_string()).is_none()); // Empty
        assert!(HexToken::new("A0".to_string()).is_none()); // Old format
        assert!(HexToken::new("t0000".to_string()).is_none()); // Lowercase T
        assert!(HexToken::new("T000G".to_string()).is_none()); // Invalid hex
        assert!(HexToken::new("T000".to_string()).is_none()); // Too short
        assert!(HexToken::new("T00000".to_string()).is_none()); // Too long
    }

    #[test]
    fn test_hex_token_display() {
        let token = HexToken::new("T0000".to_string()).unwrap();
        assert_eq!(format!("{}", token), "T0000");
        assert_eq!(token.as_str(), "T0000");
        assert_eq!(token.as_ref(), "T0000");
    }

    #[test]
    fn test_generator_configuration_validation() {
        // Valid configurations
        assert!(HexTokenGenerator::with_max_tokens(1).is_ok());
        assert!(HexTokenGenerator::with_max_tokens(65535).is_ok());

        // Invalid configurations
        assert!(HexTokenGenerator::with_max_tokens(0).is_err());
        assert!(HexTokenGenerator::with_max_tokens(65536).is_err());
    }

    #[test]
    fn test_edge_case_boundary_values() {
        let mut generator = HexTokenGenerator::with_max_tokens(1).unwrap();

        // Should generate exactly one token
        assert_eq!(generator.next_token().unwrap(), "T0000");
        assert!(generator.next_token().is_err());
    }
}
