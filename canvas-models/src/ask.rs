use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskPrompt(String);

impl AskPrompt {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AskPromptParseError {
    #[error("prompt must not be empty")]
    Empty,
}

impl FromStr for AskPrompt {
    type Err = AskPromptParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AskPromptParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskRationale(String);

impl AskRationale {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AskRationaleError {
    #[error("rationale must not be empty")]
    Empty,
}

impl TryFrom<&str> for AskRationale {
    type Error = AskRationaleError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AskRationaleError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskRisk {
    None,
    ReadOnly,
    Destructive,
}

impl AskRisk {
    pub fn as_str(self) -> &'static str {
        match self {
            AskRisk::None => "none",
            AskRisk::ReadOnly => "read_only",
            AskRisk::Destructive => "destructive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskCommandKind {
    AuthCheck,
    CourseList,
}

impl AskCommandKind {
    pub fn id(self) -> &'static str {
        match self {
            AskCommandKind::AuthCheck => "auth-check",
            AskCommandKind::CourseList => "course-list",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            AskCommandKind::AuthCheck => "auth check",
            AskCommandKind::CourseList => "course list",
        }
    }

    pub fn risk(self) -> AskRisk {
        match self {
            AskCommandKind::AuthCheck | AskCommandKind::CourseList => {
                AskRisk::None
            }
        }
    }

    pub fn destructive(self) -> bool {
        matches!(self.risk(), AskRisk::Destructive)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskParamKey(String);

impl AskParamKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AskParamKeyParseError {
    #[error("param key must not be empty")]
    Empty,
}

impl FromStr for AskParamKey {
    type Err = AskParamKeyParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AskParamKeyParseError::Empty);
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskParam {
    key: AskParamKey,
    value: String,
}

impl AskParam {
    pub fn new(key: AskParamKey, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }

    pub fn key(&self) -> &AskParamKey {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskCommand {
    kind: AskCommandKind,
    params: Vec<AskParam>,
}

impl AskCommand {
    pub fn new(kind: AskCommandKind, params: Vec<AskParam>) -> Self {
        Self { kind, params }
    }

    pub fn kind(&self) -> AskCommandKind {
        self.kind
    }

    pub fn params(&self) -> &[AskParam] {
        &self.params
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskPlan {
    prompt: AskPrompt,
    rationale: AskRationale,
    commands: Vec<AskCommand>,
}

impl AskPlan {
    pub fn new(
        prompt: AskPrompt,
        rationale: AskRationale,
        commands: Vec<AskCommand>,
    ) -> Self {
        Self {
            prompt,
            rationale,
            commands,
        }
    }

    pub fn prompt(&self) -> &AskPrompt {
        &self.prompt
    }

    pub fn rationale(&self) -> &AskRationale {
        &self.rationale
    }

    pub fn commands(&self) -> &[AskCommand] {
        &self.commands
    }
}
