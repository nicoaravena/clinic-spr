use std::error::Error;
use std::fmt;

const MAX_LENGTH: usize = 254;

/// A normalized email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Result<Self, EmailError> {
        let address = input.trim().to_lowercase();

        if address.is_empty() {
            return Err(EmailError::Empty);
        }
        if address.len() > MAX_LENGTH {
            return Err(EmailError::TooLong);
        }
        if !is_well_formed(&address) {
            return Err(EmailError::InvalidFormat);
        }

        Ok(Self(address))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A basic shape check, not full RFC 5322.
fn is_well_formed(address: &str) -> bool {
    if address.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = address.split_once('@') else {
        return false;
    };

    !local.is_empty()
        && domain.contains('.')
        && !domain.contains('@')
        && domain.split('.').all(|label| !label.is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailError {
    Empty,
    TooLong,
    InvalidFormat,
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "email address is empty",
            Self::TooLong => "email address is too long",
            Self::InvalidFormat => "email address has an invalid format",
        };
        f.write_str(message)
    }
}

impl Error for EmailError {}
