// Input Validation Module
// Provides security-focused validation for all user inputs

use crate::error::{EnterpriseError, Result};
use regex::Regex;

/// Maximum lengths for various input fields
pub mod limits {
    pub const MAX_EMAIL_LENGTH: usize = 254;
    pub const MAX_NAME_LENGTH: usize = 100;
    pub const MIN_NAME_LENGTH: usize = 1;
    pub const MAX_ORG_NAME_LENGTH: usize = 200;
    pub const MAX_DESCRIPTION_LENGTH: usize = 2000;
    pub const MAX_PHONE_LENGTH: usize = 20;
    pub const MIN_PASSWORD_LENGTH: usize = 12;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const MAX_URL_LENGTH: usize = 2048;
    pub const MAX_ADDRESS_LENGTH: usize = 500;
}

/// Email validation
/// Validates format and length according to RFC 5321/5322
pub fn validate_email(email: &str) -> Result<()> {
    // Check length
    if email.is_empty() {
        return Err(EnterpriseError::InvalidInput("Email cannot be empty".into()));
    }

    if email.len() > limits::MAX_EMAIL_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("Email exceeds maximum length of {}", limits::MAX_EMAIL_LENGTH)
        ));
    }

    // RFC 5322 compliant email regex (simplified but secure)
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    if !email_regex.is_match(email) {
        return Err(EnterpriseError::InvalidInput("Invalid email format".into()));
    }

    // Additional security: check for suspicious patterns
    if email.contains("..") || email.starts_with('.') || email.ends_with('.') {
        return Err(EnterpriseError::InvalidInput("Invalid email format".into()));
    }

    Ok(())
}

/// Name validation (legal names, user names, etc.)
/// Allows letters, spaces, hyphens, and apostrophes
pub fn validate_name(name: &str, field_name: &str) -> Result<()> {
    // Check length
    if name.is_empty() {
        return Err(EnterpriseError::InvalidInput(
            format!("{} cannot be empty", field_name)
        ));
    }

    if name.len() < limits::MIN_NAME_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("{} is too short", field_name)
        ));
    }

    if name.len() > limits::MAX_NAME_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("{} exceeds maximum length of {}", field_name, limits::MAX_NAME_LENGTH)
        ));
    }

    // Allow letters (including Unicode), spaces, hyphens, apostrophes, and periods
    let name_regex = Regex::new(r"^[\p{L}\p{M}\s'\-\.]+$").unwrap();

    if !name_regex.is_match(name) {
        return Err(EnterpriseError::InvalidInput(
            format!("{} contains invalid characters. Only letters, spaces, hyphens, apostrophes, and periods are allowed", field_name)
        ));
    }

    // Security: prevent excessive whitespace
    if name.trim() != name {
        return Err(EnterpriseError::InvalidInput(
            format!("{} cannot start or end with whitespace", field_name)
        ));
    }

    // Security: prevent multiple consecutive spaces
    if name.contains("  ") {
        return Err(EnterpriseError::InvalidInput(
            format!("{} cannot contain multiple consecutive spaces", field_name)
        ));
    }

    Ok(())
}

/// Organization name validation
pub fn validate_organization_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(EnterpriseError::InvalidInput("Organization name cannot be empty".into()));
    }

    if name.len() > limits::MAX_ORG_NAME_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("Organization name exceeds maximum length of {}", limits::MAX_ORG_NAME_LENGTH)
        ));
    }

    // Allow alphanumeric, spaces, and common business name characters
    let org_regex = Regex::new(r"^[\p{L}\p{N}\s&'.,\-()]+$").unwrap();

    if !org_regex.is_match(name) {
        return Err(EnterpriseError::InvalidInput(
            "Organization name contains invalid characters".into()
        ));
    }

    if name.trim() != name {
        return Err(EnterpriseError::InvalidInput(
            "Organization name cannot start or end with whitespace".into()
        ));
    }

    Ok(())
}

/// Password validation
/// Enforces strong password requirements
pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < limits::MIN_PASSWORD_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("Password must be at least {} characters long", limits::MIN_PASSWORD_LENGTH)
        ));
    }

    if password.len() > limits::MAX_PASSWORD_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("Password exceeds maximum length of {}", limits::MAX_PASSWORD_LENGTH)
        ));
    }

    // Check for required character types
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_lowercase || !has_uppercase || !has_digit || !has_special {
        return Err(EnterpriseError::InvalidInput(
            "Password must contain uppercase, lowercase, digit, and special character".into()
        ));
    }

    Ok(())
}

/// Phone number validation
pub fn validate_phone(phone: &str) -> Result<()> {
    if phone.is_empty() {
        return Err(EnterpriseError::InvalidInput("Phone number cannot be empty".into()));
    }

    if phone.len() > limits::MAX_PHONE_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("Phone number exceeds maximum length of {}", limits::MAX_PHONE_LENGTH)
        ));
    }

    // Allow digits, spaces, dashes, parentheses, and + for international
    let phone_regex = Regex::new(r"^[\d\s\-+()]+$").unwrap();

    if !phone_regex.is_match(phone) {
        return Err(EnterpriseError::InvalidInput(
            "Phone number contains invalid characters".into()
        ));
    }

    // Extract digits only for minimum length check
    let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();
    if digits.len() < 10 {
        return Err(EnterpriseError::InvalidInput(
            "Phone number must contain at least 10 digits".into()
        ));
    }

    Ok(())
}

/// Description/text field validation
pub fn validate_description(description: &str, field_name: &str) -> Result<()> {
    if description.len() > limits::MAX_DESCRIPTION_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("{} exceeds maximum length of {}", field_name, limits::MAX_DESCRIPTION_LENGTH)
        ));
    }

    // Security: prevent control characters (except newline and tab)
    for ch in description.chars() {
        if ch.is_control() && ch != '\n' && ch != '\t' && ch != '\r' {
            return Err(EnterpriseError::InvalidInput(
                format!("{} contains invalid control characters", field_name)
            ));
        }
    }

    Ok(())
}

/// URL validation
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(EnterpriseError::InvalidInput("URL cannot be empty".into()));
    }

    if url.len() > limits::MAX_URL_LENGTH {
        return Err(EnterpriseError::InvalidInput(
            format!("URL exceeds maximum length of {}", limits::MAX_URL_LENGTH)
        ));
    }

    // Basic URL validation
    let url_regex = Regex::new(r"^https?://[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=]+$").unwrap();

    if !url_regex.is_match(url) {
        return Err(EnterpriseError::InvalidInput("Invalid URL format".into()));
    }

    // Security: only allow http and https schemes
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(EnterpriseError::InvalidInput(
            "URL must use http:// or https:// scheme".into()
        ));
    }

    Ok(())
}

/// UUID validation
pub fn validate_uuid(uuid_str: &str, field_name: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(uuid_str).map_err(|_| {
        EnterpriseError::InvalidInput(format!("Invalid UUID format for {}", field_name))
    })
}

/// Amount validation (for financial transactions)
pub fn validate_amount(amount: u64) -> Result<()> {
    if amount == 0 {
        return Err(EnterpriseError::InvalidInput("Amount must be greater than zero".into()));
    }

    // Prevent overflow issues with reasonable maximum
    const MAX_AMOUNT: u64 = u64::MAX / 2;
    if amount > MAX_AMOUNT {
        return Err(EnterpriseError::InvalidInput("Amount exceeds maximum allowed value".into()));
    }

    Ok(())
}

/// Sanitize string by removing potentially dangerous characters
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

/// HIGH PRIORITY FIX: Metadata size validation
/// Prevents DoS via large metadata payloads
pub fn validate_metadata_size(metadata: &serde_json::Value) -> Result<()> {
    // Maximum metadata size: 256 bytes (as recommended in audit)
    const MAX_METADATA_SIZE: usize = 256;

    let serialized = serde_json::to_string(metadata)
        .map_err(|e| EnterpriseError::InvalidInput(format!("Invalid metadata format: {}", e)))?;

    if serialized.len() > MAX_METADATA_SIZE {
        return Err(EnterpriseError::InvalidInput(
            format!("Metadata exceeds maximum size of {} bytes (actual: {} bytes)",
                MAX_METADATA_SIZE, serialized.len())
        ));
    }

    Ok(())
}

/// HIGH PRIORITY FIX: Sanitize HTML/JavaScript to prevent XSS
/// Removes potentially dangerous HTML/JS characters from user input
pub fn sanitize_for_display(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name+tag@example.co.uk").is_ok());

        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user..name@example.com").is_err());
    }

    #[test]
    fn test_validate_name() {
        assert!(validate_name("John Doe", "Name").is_ok());
        assert!(validate_name("Mary-Jane O'Brien", "Name").is_ok());

        assert!(validate_name("", "Name").is_err());
        assert!(validate_name("Name123", "Name").is_err());
        assert!(validate_name("  Name", "Name").is_err());
        assert!(validate_name("Name  With  Spaces", "Name").is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("SecurePass123!").is_ok());
        assert!(validate_password("Tr0ng#P@ssw0rd").is_ok());

        assert!(validate_password("weak").is_err());
        assert!(validate_password("NoSpecialChar1").is_err());
        assert!(validate_password("nouppercas3!").is_err());
        assert!(validate_password("NOLOWERCASE1!").is_err());
    }

    #[test]
    fn test_validate_organization_name() {
        assert!(validate_organization_name("Acme Corp.").is_ok());
        assert!(validate_organization_name("Smith & Sons LLC").is_ok());

        assert!(validate_organization_name("").is_err());
        assert!(validate_organization_name("  Acme").is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(100).is_ok());
        assert!(validate_amount(1000000).is_ok());

        assert!(validate_amount(0).is_err());
    }
}
