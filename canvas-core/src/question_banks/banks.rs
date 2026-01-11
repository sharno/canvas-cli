use canvas_models::{CourseId, QuestionBankId, QuestionBankTitle};
use serde::{Deserialize, Serialize};

use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Deserialize)]
struct ApiQuestionBank {
    id: u64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    question_count: Option<u64>,
    #[serde(default)]
    context_type: Option<String>,
    #[serde(default)]
    context_id: Option<u64>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

impl ApiQuestionBank {
    fn into_summary(self) -> QuestionBankSummary {
        QuestionBankSummary {
            id: self.id,
            title: self.title,
            question_count: self.question_count,
            context_type: self.context_type,
            context_id: self.context_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuestionBankSummary {
    pub id: u64,
    pub title: Option<String>,
    pub question_count: Option<u64>,
    pub context_type: Option<String>,
    pub context_id: Option<u64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuestionBankCreateInput {
    pub title: QuestionBankTitle,
}

impl QuestionBankCreateInput {
    pub fn new(title: QuestionBankTitle) -> Self {
        Self { title }
    }

    pub fn from_json(raw: &str) -> Result<Self, CanvasError> {
        let parsed: QuestionBankJsonInput =
            serde_json::from_str(raw).map_err(|err| {
                CanvasError::InvalidQuestionBankJson(err.to_string())
            })?;
        let title_raw = parsed
            .title
            .ok_or_else(|| CanvasError::InvalidQuestionBankTitle("missing".to_string()))?;
        let title = title_raw
            .parse::<QuestionBankTitle>()
            .map_err(|_| CanvasError::InvalidQuestionBankTitle(title_raw))?;
        Ok(Self { title })
    }
}

#[derive(Debug, Clone)]
pub struct QuestionBankUpdateInput {
    pub title: Option<QuestionBankTitle>,
}

impl QuestionBankUpdateInput {
    pub fn new(title: Option<QuestionBankTitle>) -> Result<Self, CanvasError> {
        if title.is_none() {
            return Err(CanvasError::InvalidQuestionBankUpdate("no_fields".to_string()));
        }
        Ok(Self { title })
    }

    pub fn from_json(raw: &str) -> Result<Self, CanvasError> {
        let parsed: QuestionBankJsonInput =
            serde_json::from_str(raw).map_err(|err| {
                CanvasError::InvalidQuestionBankJson(err.to_string())
            })?;
        let title = match parsed.title {
            Some(value) => Some(
                value
                    .parse::<QuestionBankTitle>()
                    .map_err(|_| CanvasError::InvalidQuestionBankTitle(value))?,
            ),
            None => None,
        };
        Self::new(title)
    }
}

#[derive(Debug, Deserialize)]
struct QuestionBankJsonInput {
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct QuestionBankCreateForm {
    title: String,
}

#[derive(Debug, Serialize)]
struct QuestionBankUpdateForm {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

pub fn list_question_banks(
    config: &CanvasConfig,
    course_id: CourseId,
) -> Result<Vec<QuestionBankSummary>, CanvasError> {
    let client = CanvasClient::new(config)?;
    let banks: Vec<ApiQuestionBank> = client
        .get_paginated(&format!("/courses/{}/question_banks", course_id.get()))?;
    Ok(banks.into_iter().map(ApiQuestionBank::into_summary).collect())
}

pub fn create_question_bank(
    config: &CanvasConfig,
    course_id: CourseId,
    input: &QuestionBankCreateInput,
) -> Result<QuestionBankSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let form = QuestionBankCreateForm {
        title: input.title.as_str().to_string(),
    };
    let bank: ApiQuestionBank = client
        .post_json(&format!("/courses/{}/question_banks", course_id.get()), &form)?;
    Ok(bank.into_summary())
}

pub fn update_question_bank(
    config: &CanvasConfig,
    course_id: CourseId,
    bank_id: QuestionBankId,
    input: &QuestionBankUpdateInput,
) -> Result<QuestionBankSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let form = QuestionBankUpdateForm {
        title: input.title.as_ref().map(|value| value.as_str().to_string()),
    };
    let bank: ApiQuestionBank = client.put_json(
        &format!(
            "/courses/{}/question_banks/{}",
            course_id.get(),
            bank_id.get()
        ),
        &form,
    )?;
    Ok(bank.into_summary())
}

pub fn delete_question_bank(
    config: &CanvasConfig,
    course_id: CourseId,
    bank_id: QuestionBankId,
) -> Result<QuestionBankSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let bank: ApiQuestionBank = client.delete_json(&format!(
        "/courses/{}/question_banks/{}",
        course_id.get(),
        bank_id.get()
    ))?;
    Ok(bank.into_summary())
}
