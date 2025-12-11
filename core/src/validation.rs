//! Input Validation Module
//!
//! Provides validation utilities for sanitizing and validating input
//! at API boundaries to prevent security vulnerabilities.

use std::collections::HashSet;

/// Maximum allowed string length for user inputs
pub const MAX_STRING_LENGTH: usize = 10_000;
/// Maximum allowed job name length
pub const MAX_JOB_NAME_LENGTH: usize = 256;
/// Maximum allowed path length
pub const MAX_PATH_LENGTH: usize = 4096;
/// Maximum allowed command length
pub const MAX_COMMAND_LENGTH: usize = 65536;
/// Maximum number of environment variables
pub const MAX_ENV_VARS: usize = 1000;
/// Maximum number of arguments
pub const MAX_ARGUMENTS: usize = 1000;

/// Validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Input is empty when it shouldn't be
    Empty(String),
    /// Input exceeds maximum length
    TooLong { field: String, max: usize, actual: usize },
    /// Input contains invalid characters
    InvalidChars { field: String, invalid: String },
    /// Input contains forbidden patterns
    ForbiddenPattern { field: String, pattern: String },
    /// Input is out of valid range
    OutOfRange { field: String, min: i64, max: i64, actual: i64 },
    /// Generic validation failure
    Invalid(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty(field) => write!(f, "{} cannot be empty", field),
            Self::TooLong { field, max, actual } => {
                write!(f, "{} too long: {} > {} max", field, actual, max)
            }
            Self::InvalidChars { field, invalid } => {
                write!(f, "{} contains invalid characters: {}", field, invalid)
            }
            Self::ForbiddenPattern { field, pattern } => {
                write!(f, "{} contains forbidden pattern: {}", field, pattern)
            }
            Self::OutOfRange { field, min, max, actual } => {
                write!(f, "{} out of range: {} not in [{}, {}]", field, actual, min, max)
            }
            Self::Invalid(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Input validator with configurable rules
pub struct Validator {
    /// Forbidden command patterns (for security)
    forbidden_patterns: HashSet<String>,
}

impl Default for Validator {
    fn default() -> Self {
        let mut forbidden_patterns = HashSet::new();
        // Prevent shell injection
        forbidden_patterns.insert("$((".to_string());
        forbidden_patterns.insert("$(".to_string());
        forbidden_patterns.insert("`".to_string());
        forbidden_patterns.insert("&&".to_string());
        forbidden_patterns.insert("||".to_string());
        forbidden_patterns.insert(";".to_string());
        forbidden_patterns.insert("|".to_string());
        forbidden_patterns.insert(">".to_string());
        forbidden_patterns.insert("<".to_string());
        forbidden_patterns.insert("..".to_string());  // Path traversal
        
        Self { forbidden_patterns }
    }
}

impl Validator {
    /// Create a new validator with default rules
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate a string is not empty
    pub fn require_non_empty(&self, field: &str, value: &str) -> ValidationResult<()> {
        if value.trim().is_empty() {
            Err(ValidationError::Empty(field.to_string()))
        } else {
            Ok(())
        }
    }
    
    /// Validate string length
    pub fn validate_length(&self, field: &str, value: &str, max: usize) -> ValidationResult<()> {
        if value.len() > max {
            Err(ValidationError::TooLong {
                field: field.to_string(),
                max,
                actual: value.len(),
            })
        } else {
            Ok(())
        }
    }
    
    /// Validate a job name (alphanumeric, dashes, underscores)
    pub fn validate_job_name(&self, name: &str) -> ValidationResult<()> {
        self.require_non_empty("job_name", name)?;
        self.validate_length("job_name", name, MAX_JOB_NAME_LENGTH)?;
        
        let invalid: String = name.chars()
            .filter(|c| !c.is_alphanumeric() && *c != '-' && *c != '_')
            .collect();
        
        if !invalid.is_empty() {
            Err(ValidationError::InvalidChars {
                field: "job_name".to_string(),
                invalid,
            })
        } else {
            Ok(())
        }
    }
    
    /// Validate a path (no traversal attacks)
    pub fn validate_path(&self, path: &str) -> ValidationResult<()> {
        self.validate_length("path", path, MAX_PATH_LENGTH)?;
        
        // Check for path traversal
        if path.contains("..") {
            return Err(ValidationError::ForbiddenPattern {
                field: "path".to_string(),
                pattern: "..".to_string(),
            });
        }
        
        // Check for null bytes
        if path.contains('\0') {
            return Err(ValidationError::InvalidChars {
                field: "path".to_string(),
                invalid: "null byte".to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Validate a command (check for injection patterns)
    pub fn validate_command(&self, command: &str) -> ValidationResult<()> {
        self.require_non_empty("command", command)?;
        self.validate_length("command", command, MAX_COMMAND_LENGTH)?;
        
        for pattern in &self.forbidden_patterns {
            if command.contains(pattern) {
                return Err(ValidationError::ForbiddenPattern {
                    field: "command".to_string(),
                    pattern: pattern.clone(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate a numeric value is in range
    pub fn validate_range(&self, field: &str, value: i64, min: i64, max: i64) -> ValidationResult<()> {
        if value < min || value > max {
            Err(ValidationError::OutOfRange {
                field: field.to_string(),
                min,
                max,
                actual: value,
            })
        } else {
            Ok(())
        }
    }
    
    /// Validate GPU count
    pub fn validate_gpu_count(&self, count: u32) -> ValidationResult<()> {
        self.validate_range("gpu_count", count as i64, 0, 1024)
    }
    
    /// Validate priority
    pub fn validate_priority(&self, priority: i32) -> ValidationResult<()> {
        self.validate_range("priority", priority as i64, -1000, 1000)
    }
    
    /// Validate buffer size
    pub fn validate_buffer_size(&self, size: usize) -> ValidationResult<()> {
        self.validate_range("buffer_size", size as i64, 1, 1024 * 1024 * 1024)  // 1GB max
    }
}

/// Sanitize a string by removing control characters
pub fn sanitize_string(input: &str) -> String {
    input.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

/// Sanitize a log message
pub fn sanitize_log_message(message: &str) -> String {
    let sanitized = sanitize_string(message);
    if sanitized.len() > MAX_STRING_LENGTH {
        format!("{}... [truncated]", &sanitized[..MAX_STRING_LENGTH])
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_job_name() {
        let v = Validator::new();
        
        assert!(v.validate_job_name("my-job-123").is_ok());
        assert!(v.validate_job_name("test_job").is_ok());
        assert!(v.validate_job_name("").is_err());
        assert!(v.validate_job_name("my job").is_err());  // space not allowed
        assert!(v.validate_job_name("job;rm -rf").is_err());  // injection attempt
    }
    
    #[test]
    fn test_validate_path() {
        let v = Validator::new();
        
        assert!(v.validate_path("/home/user/data").is_ok());
        assert!(v.validate_path("../../../etc/passwd").is_err());
        assert!(v.validate_path("/path/with\0null").is_err());
    }
    
    #[test]
    fn test_validate_command() {
        let v = Validator::new();
        
        // Valid commands
        assert!(v.validate_command("python train.py").is_ok());
        assert!(v.validate_command("python").is_ok());
        assert!(v.validate_command("python3 -m pytest").is_ok());
        
        // Invalid - shell injection patterns
        assert!(v.validate_command("$(cat /etc/passwd)").is_err());
        assert!(v.validate_command("echo `whoami`").is_err());
        assert!(v.validate_command("cmd1 && cmd2").is_err());
        assert!(v.validate_command("cmd1 || cmd2").is_err());
        assert!(v.validate_command("cmd ; rm -rf /").is_err());
        assert!(v.validate_command("cat file | grep secret").is_err());
        assert!(v.validate_command("echo > /etc/passwd").is_err());
    }
    
    #[test]
    fn test_validate_range() {
        let v = Validator::new();
        
        assert!(v.validate_gpu_count(0).is_ok());
        assert!(v.validate_gpu_count(8).is_ok());
        assert!(v.validate_priority(0).is_ok());
        assert!(v.validate_priority(-100).is_ok());
    }
    
    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_string("hello\x00world"), "helloworld");
        assert_eq!(sanitize_string("line1\nline2"), "line1\nline2");
    }
    
    // ========================================================================
    // MUTATION-KILLING TESTS - Comprehensive coverage for mutation testing
    // ========================================================================
    
    #[test]
    fn test_validate_length_boundary_conditions() {
        let v = Validator::new();
        
        // Test exact boundary: length == max should pass
        let exactly_max = "a".repeat(100);
        assert!(v.validate_length("field", &exactly_max, 100).is_ok());
        
        // Test over boundary: length > max should fail
        let over_max = "a".repeat(101);
        assert!(v.validate_length("field", &over_max, 100).is_err());
        
        // Test under boundary: length < max should pass
        let under_max = "a".repeat(99);
        assert!(v.validate_length("field", &under_max, 100).is_ok());
        
        // This catches mutation: > replaced with ==
        // If > becomes ==, then 101 > 100 would be false (fail to catch)
        // but 100 == 100 would be true (incorrectly fail)
        let exactly_100 = "a".repeat(100);
        let exactly_101 = "a".repeat(101);
        assert!(v.validate_length("field", &exactly_100, 100).is_ok(), 
            "Exactly max should pass - catches > to == mutation");
        assert!(v.validate_length("field", &exactly_101, 100).is_err(),
            "Over max should fail - catches > to == mutation");
    }
    
    #[test]
    fn test_validate_range_boundary_conditions() {
        let v = Validator::new();
        
        // Test exactly at min boundary
        assert!(v.validate_range("test", 0, 0, 100).is_ok());
        
        // Test exactly at max boundary
        assert!(v.validate_range("test", 100, 0, 100).is_ok());
        
        // Test below min (should fail)
        assert!(v.validate_range("test", -1, 0, 100).is_err());
        
        // Test above max (should fail)
        assert!(v.validate_range("test", 101, 0, 100).is_err());
        
        // This catches mutation: || replaced with &&
        // If || becomes &&, then (value < min) && (value > max) is never true
        // So values both below min and above max would incorrectly pass
        assert!(v.validate_range("test", -10, 0, 100).is_err(),
            "Below min should fail - catches || to && mutation");
        assert!(v.validate_range("test", 200, 0, 100).is_err(),
            "Above max should fail - catches || to && mutation");
        
        // This catches mutation: > replaced with ==
        // value > max becomes value == max, so only exactly max fails
        assert!(v.validate_range("test", 101, 0, 100).is_err(),
            "Just above max should fail - catches > to == mutation");
        assert!(v.validate_range("test", 100, 0, 100).is_ok(),
            "Exactly max should pass - catches > to == mutation");
    }
    
    #[test]
    fn test_validate_buffer_size_arithmetic() {
        let v = Validator::new();
        
        // Valid buffer sizes
        assert!(v.validate_buffer_size(1).is_ok());
        assert!(v.validate_buffer_size(1024).is_ok());
        assert!(v.validate_buffer_size(1024 * 1024).is_ok());  // 1MB
        
        // Max valid: 1GB = 1024 * 1024 * 1024
        assert!(v.validate_buffer_size(1024 * 1024 * 1024).is_ok());
        
        // Too large (over 1GB)
        assert!(v.validate_buffer_size(1024 * 1024 * 1024 + 1).is_err());
        
        // Zero is invalid (below min of 1)
        assert!(v.validate_buffer_size(0).is_err());
        
        // These catch arithmetic mutations (* -> +, * -> /)
        // If 1024 * 1024 * 1024 becomes 1024 + 1024 + 1024 = 3072
        // then 1MB (1048576) would incorrectly fail
        // If 1024 * 1024 * 1024 becomes 1024 / 1024 / 1024 = 0
        // then almost everything would fail
        let one_mb = 1024 * 1024;
        assert!(v.validate_buffer_size(one_mb).is_ok(),
            "1MB should be valid - catches * to + or / mutation");
        
        let half_gb = 512 * 1024 * 1024;
        assert!(v.validate_buffer_size(half_gb).is_ok(),
            "512MB should be valid - catches arithmetic mutations");
    }
    
    #[test]
    fn test_sanitize_log_message_truncation() {
        // Test that truncation happens at correct length
        let short_msg = "short message";
        assert_eq!(sanitize_log_message(short_msg), short_msg);
        
        // Exactly at max length
        let exactly_max = "a".repeat(MAX_STRING_LENGTH);
        assert_eq!(sanitize_log_message(&exactly_max), exactly_max);
        
        // Over max length - should truncate
        let over_max = "a".repeat(MAX_STRING_LENGTH + 100);
        let truncated = sanitize_log_message(&over_max);
        assert!(truncated.ends_with("... [truncated]"));
        assert!(truncated.len() < over_max.len());
        
        // This catches mutation: > replaced with < or ==
        // If > becomes <, short messages would be truncated
        // If > becomes ==, only exactly MAX_STRING_LENGTH would be truncated
        let just_over = "a".repeat(MAX_STRING_LENGTH + 1);
        let result = sanitize_log_message(&just_over);
        assert!(result.ends_with("... [truncated]"),
            "Just over max should truncate - catches > to < or == mutation");
        
        // Verify non-truncated doesn't have suffix
        let at_max = "b".repeat(MAX_STRING_LENGTH);
        let result_at_max = sanitize_log_message(&at_max);
        assert!(!result_at_max.ends_with("... [truncated]"),
            "At max should not truncate - catches > to < mutation");
    }
    
    #[test]
    fn test_sanitize_log_message_returns_string() {
        // Catches mutation: replace with String::new() or "xyzzy".into()
        let input = "hello world";
        let result = sanitize_log_message(input);
        
        // Result should contain the input content
        assert!(result.contains("hello"),
            "Result should contain input - catches return value mutations");
        assert!(result.contains("world"),
            "Result should contain input - catches return value mutations");
        
        // Specific check for "xyzzy" mutation
        assert!(!result.contains("xyzzy"),
            "Result should not be 'xyzzy' - catches specific mutation");
        
        // Check it's not empty
        assert!(!result.is_empty(),
            "Result should not be empty - catches String::new() mutation");
    }
    
    #[test]
    fn test_validation_error_display() {
        // Test Display trait to catch fmt mutation
        let empty_err = ValidationError::Empty("field".to_string());
        let display = format!("{}", empty_err);
        assert!(display.contains("field"));
        assert!(display.contains("empty"));
        
        let too_long = ValidationError::TooLong { 
            field: "name".to_string(), 
            max: 10, 
            actual: 20 
        };
        let display = format!("{}", too_long);
        assert!(display.contains("name"));
        assert!(display.contains("10"));
        assert!(display.contains("20"));
        
        let invalid_chars = ValidationError::InvalidChars {
            field: "test".to_string(),
            invalid: "!@#".to_string(),
        };
        let display = format!("{}", invalid_chars);
        assert!(display.contains("test"));
        assert!(display.contains("!@#"));
        
        let forbidden = ValidationError::ForbiddenPattern {
            field: "cmd".to_string(),
            pattern: "&&".to_string(),
        };
        let display = format!("{}", forbidden);
        assert!(display.contains("cmd"));
        assert!(display.contains("&&"));
        
        let out_of_range = ValidationError::OutOfRange {
            field: "value".to_string(),
            min: 0,
            max: 100,
            actual: 200,
        };
        let display = format!("{}", out_of_range);
        assert!(display.contains("value"));
        assert!(display.contains("0"));
        assert!(display.contains("100"));
        assert!(display.contains("200"));
        
        let invalid = ValidationError::Invalid("custom error".to_string());
        let display = format!("{}", invalid);
        assert!(display.contains("custom error"));
    }
    
    #[test]
    fn test_require_non_empty() {
        let v = Validator::new();
        
        // Non-empty should pass
        assert!(v.require_non_empty("field", "value").is_ok());
        
        // Empty should fail
        assert!(v.require_non_empty("field", "").is_err());
        
        // Whitespace-only should fail (trimmed)
        assert!(v.require_non_empty("field", "   ").is_err());
        assert!(v.require_non_empty("field", "\t\n").is_err());
    }
    
    #[test]
    fn test_validate_gpu_count_boundaries() {
        let v = Validator::new();
        
        // Valid range: 0 to 1024
        assert!(v.validate_gpu_count(0).is_ok());
        assert!(v.validate_gpu_count(1024).is_ok());
        assert!(v.validate_gpu_count(512).is_ok());
        
        // Invalid: over 1024
        // Note: u32 can't be negative, so we only test upper bound
        assert!(v.validate_gpu_count(1025).is_err());
    }
    
    #[test]
    fn test_validate_priority_boundaries() {
        let v = Validator::new();
        
        // Valid range: -1000 to 1000
        assert!(v.validate_priority(-1000).is_ok());
        assert!(v.validate_priority(1000).is_ok());
        assert!(v.validate_priority(0).is_ok());
        
        // Invalid
        assert!(v.validate_priority(-1001).is_err());
        assert!(v.validate_priority(1001).is_err());
    }
    
    #[test]
    fn test_sanitize_string_control_chars() {
        // Test various control characters are removed
        assert_eq!(sanitize_string("a\x00b"), "ab");  // null
        assert_eq!(sanitize_string("a\x01b"), "ab");  // SOH
        assert_eq!(sanitize_string("a\x07b"), "ab");  // bell
        assert_eq!(sanitize_string("a\x1Bb"), "ab");  // escape
        
        // Tab and newline should be preserved
        assert_eq!(sanitize_string("a\tb"), "a\tb");
        assert_eq!(sanitize_string("a\nb"), "a\nb");
        assert_eq!(sanitize_string("a\n\tb"), "a\n\tb");
        
        // Normal text unchanged
        assert_eq!(sanitize_string("hello world"), "hello world");
        assert_eq!(sanitize_string(""), "");
    }
}
