use std::str::FromStr;
use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::CourseId;

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
}
