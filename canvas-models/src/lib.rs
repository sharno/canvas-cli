use std::str::FromStr;
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CourseId(u64);

impl CourseId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CourseIdParseError {
    #[error("course id must be a positive integer")]
    Invalid,
}

impl FromStr for CourseId {
    type Err = CourseIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed.parse::<u64>().map_err(|_| CourseIdParseError::Invalid)?;
        if value == 0 {
            return Err(CourseIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanvasHost(String);

impl CanvasHost {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CanvasHostParseError {
    #[error("host must not be empty")]
    Empty,
    #[error("host must be a valid URL")]
    InvalidUrl,
    #[error("host must use http or https")]
    InvalidScheme,
    #[error("host must include a domain")]
    MissingHost,
}

impl FromStr for CanvasHost {
    type Err = CanvasHostParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CanvasHostParseError::Empty);
        }

        let parsed = Url::parse(trimmed).map_err(|_| CanvasHostParseError::InvalidUrl)?;
        match parsed.scheme() {
            "http" | "https" => {}
            _ => return Err(CanvasHostParseError::InvalidScheme),
        }
        if parsed.host_str().is_none() {
            return Err(CanvasHostParseError::MissingHost);
        }

        let normalized = trimmed.trim_end_matches('/');
        Ok(Self(normalized.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanvasToken(String);

impl CanvasToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CanvasTokenParseError {
    #[error("token must not be empty")]
    Empty,
    #[error("token must not contain whitespace")]
    ContainsWhitespace,
}

impl FromStr for CanvasToken {
    type Err = CanvasTokenParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CanvasTokenParseError::Empty);
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(CanvasTokenParseError::ContainsWhitespace);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{CanvasHost, CanvasToken, CourseId};

    #[test]
    fn course_id_parses_positive_integers() {
        let parsed: CourseId = "42".parse().expect("expected course id");
        assert_eq!(parsed.get(), 42);
    }

    #[test]
    fn course_id_rejects_zero() {
        let parsed: Result<CourseId, _> = "0".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn course_id_rejects_non_numbers() {
        let parsed: Result<CourseId, _> = "abc".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn canvas_host_accepts_http_urls() {
        let host: CanvasHost = "https://example.instructure.com".parse().expect("host");
        assert_eq!(host.as_str(), "https://example.instructure.com");
    }

    #[test]
    fn canvas_host_rejects_missing_scheme() {
        let parsed: Result<CanvasHost, _> = "example.instructure.com".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn canvas_token_rejects_whitespace() {
        let parsed: Result<CanvasToken, _> = "tok en".parse();
        assert!(parsed.is_err());
    }

    #[test]
    fn canvas_token_accepts_non_empty() {
        let token: CanvasToken = "token123".parse().expect("token");
        assert_eq!(token.as_str(), "token123");
    }
}
