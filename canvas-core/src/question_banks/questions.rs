use canvas_models::{
    PointsPossible, QuestionBankId, QuestionId, QuestionName, QuestionText, QuestionType,
};
use serde::{Deserialize, Serialize};

use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Deserialize)]
struct ApiQuestion {
    id: u64,
    #[serde(default)]
    question_name: Option<String>,
    #[serde(default)]
    question_text: Option<String>,
    #[serde(default)]
    question_type: Option<String>,
    #[serde(default)]
    points_possible: Option<f64>,
    #[serde(default)]
    position: Option<u32>,
    #[serde(default)]
    correct_comments: Option<String>,
    #[serde(default)]
    incorrect_comments: Option<String>,
    #[serde(default)]
    neutral_comments: Option<String>,
    #[serde(default)]
    answers: Vec<ApiQuestionAnswer>,
}

#[derive(Debug, Deserialize)]
struct ApiQuestionAnswer {
    #[serde(default)]
    id: Option<u64>,
    #[serde(default, rename = "answer_text")]
    text: Option<String>,
    #[serde(default, rename = "answer_weight")]
    weight: Option<f64>,
    #[serde(default, rename = "answer_comments")]
    comments: Option<String>,
    #[serde(default, rename = "answer_html")]
    html: Option<String>,
}

impl ApiQuestion {
    fn into_summary(self) -> QuestionSummary {
        let answers = self
            .answers
            .into_iter()
            .map(|answer| QuestionAnswerSummary {
                id: answer.id,
                text: answer.text,
                weight: answer.weight,
                comments: answer.comments,
                html: answer.html,
            })
            .collect();
        QuestionSummary {
            id: self.id,
            question_name: self.question_name,
            question_text: self.question_text,
            question_type: self.question_type,
            points_possible: self.points_possible,
            position: self.position,
            correct_comments: self.correct_comments,
            incorrect_comments: self.incorrect_comments,
            neutral_comments: self.neutral_comments,
            answers,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuestionAnswerSummary {
    pub id: Option<u64>,
    pub text: Option<String>,
    pub weight: Option<f64>,
    pub comments: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuestionSummary {
    pub id: u64,
    pub question_name: Option<String>,
    pub question_text: Option<String>,
    pub question_type: Option<String>,
    pub points_possible: Option<f64>,
    pub position: Option<u32>,
    pub correct_comments: Option<String>,
    pub incorrect_comments: Option<String>,
    pub neutral_comments: Option<String>,
    pub answers: Vec<QuestionAnswerSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswerInput {
    #[serde(rename = "answer_text")]
    text: String,
    #[serde(rename = "answer_weight", default, skip_serializing_if = "Option::is_none")]
    weight: Option<f64>,
    #[serde(rename = "answer_comments", default, skip_serializing_if = "Option::is_none")]
    comments: Option<String>,
    #[serde(rename = "answer_html", default, skip_serializing_if = "Option::is_none")]
    html: Option<String>,
}

impl QuestionAnswerInput {
    pub fn new(text: String) -> Result<Self, CanvasError> {
        if text.trim().is_empty() {
            return Err(CanvasError::InvalidQuestionText(text));
        }
        Ok(Self {
            text,
            weight: None,
            comments: None,
            html: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct QuestionCreateInput {
    pub question_name: Option<QuestionName>,
    pub question_text: QuestionText,
    pub question_type: QuestionType,
    pub points_possible: Option<PointsPossible>,
    pub correct_comments: Option<String>,
    pub incorrect_comments: Option<String>,
    pub neutral_comments: Option<String>,
    pub answers: Option<Vec<QuestionAnswerInput>>,
}

impl QuestionCreateInput {
    pub fn new(
        question_name: Option<QuestionName>,
        question_text: QuestionText,
        question_type: QuestionType,
        points_possible: Option<PointsPossible>,
        correct_comments: Option<String>,
        incorrect_comments: Option<String>,
        neutral_comments: Option<String>,
        answers: Option<Vec<QuestionAnswerInput>>,
    ) -> Self {
        Self {
            question_name,
            question_text,
            question_type,
            points_possible,
            correct_comments,
            incorrect_comments,
            neutral_comments,
            answers,
        }
    }

    pub fn from_json(raw: &str) -> Result<Self, CanvasError> {
        let parsed: QuestionJsonInput = serde_json::from_str(raw)
            .map_err(|err| CanvasError::InvalidQuestionJson(err.to_string()))?;
        parsed.into_create()
    }
}

#[derive(Debug, Clone)]
pub struct QuestionUpdateInput {
    pub question_name: Option<QuestionName>,
    pub question_text: Option<QuestionText>,
    pub question_type: Option<QuestionType>,
    pub points_possible: Option<PointsPossible>,
    pub correct_comments: Option<String>,
    pub incorrect_comments: Option<String>,
    pub neutral_comments: Option<String>,
    pub answers: Option<Vec<QuestionAnswerInput>>,
}

impl QuestionUpdateInput {
    pub fn new(
        question_name: Option<QuestionName>,
        question_text: Option<QuestionText>,
        question_type: Option<QuestionType>,
        points_possible: Option<PointsPossible>,
        correct_comments: Option<String>,
        incorrect_comments: Option<String>,
        neutral_comments: Option<String>,
        answers: Option<Vec<QuestionAnswerInput>>,
    ) -> Result<Self, CanvasError> {
        if question_name.is_none()
            && question_text.is_none()
            && question_type.is_none()
            && points_possible.is_none()
            && correct_comments.is_none()
            && incorrect_comments.is_none()
            && neutral_comments.is_none()
            && answers.is_none()
        {
            return Err(CanvasError::InvalidQuestionUpdate("no_fields".to_string()));
        }
        Ok(Self {
            question_name,
            question_text,
            question_type,
            points_possible,
            correct_comments,
            incorrect_comments,
            neutral_comments,
            answers,
        })
    }

    pub fn from_json(raw: &str) -> Result<Self, CanvasError> {
        let parsed: QuestionJsonInput = serde_json::from_str(raw)
            .map_err(|err| CanvasError::InvalidQuestionJson(err.to_string()))?;
        parsed.into_update()
    }
}

#[derive(Debug, Deserialize)]
struct QuestionJsonInput {
    #[serde(default)]
    question_name: Option<String>,
    #[serde(default)]
    question_text: Option<String>,
    #[serde(default)]
    question_type: Option<String>,
    #[serde(default)]
    points_possible: Option<f64>,
    #[serde(default)]
    correct_comments: Option<String>,
    #[serde(default)]
    incorrect_comments: Option<String>,
    #[serde(default)]
    neutral_comments: Option<String>,
    #[serde(default)]
    answers: Option<Vec<QuestionAnswerInput>>,
}

impl QuestionJsonInput {
    fn into_create(self) -> Result<QuestionCreateInput, CanvasError> {
        let text_raw = self
            .question_text
            .ok_or_else(|| CanvasError::InvalidQuestionText("missing".to_string()))?;
        let question_text = text_raw
            .parse::<QuestionText>()
            .map_err(|_| CanvasError::InvalidQuestionText(text_raw))?;
        let type_raw = self
            .question_type
            .ok_or_else(|| CanvasError::InvalidQuestionType("missing".to_string()))?;
        let question_type = type_raw
            .parse::<QuestionType>()
            .map_err(|_| CanvasError::InvalidQuestionType(type_raw))?;
        let question_name = match self.question_name {
            Some(raw) => Some(
                raw.parse::<QuestionName>()
                    .map_err(|_| CanvasError::InvalidQuestionName(raw))?,
            ),
            None => None,
        };
        let points_possible = match self.points_possible {
            Some(value) => Some(
                value
                    .to_string()
                    .parse::<PointsPossible>()
                    .map_err(|_| CanvasError::InvalidPointsPossible(value.to_string()))?,
            ),
            None => None,
        };
        Ok(QuestionCreateInput::new(
            question_name,
            question_text,
            question_type,
            points_possible,
            self.correct_comments,
            self.incorrect_comments,
            self.neutral_comments,
            self.answers,
        ))
    }

    fn into_update(self) -> Result<QuestionUpdateInput, CanvasError> {
        let question_name = match self.question_name {
            Some(raw) => Some(
                raw.parse::<QuestionName>()
                    .map_err(|_| CanvasError::InvalidQuestionName(raw))?,
            ),
            None => None,
        };
        let question_text = match self.question_text {
            Some(raw) => Some(
                raw.parse::<QuestionText>()
                    .map_err(|_| CanvasError::InvalidQuestionText(raw))?,
            ),
            None => None,
        };
        let question_type = match self.question_type {
            Some(raw) => Some(
                raw.parse::<QuestionType>()
                    .map_err(|_| CanvasError::InvalidQuestionType(raw))?,
            ),
            None => None,
        };
        let points_possible = match self.points_possible {
            Some(value) => Some(
                value
                    .to_string()
                    .parse::<PointsPossible>()
                    .map_err(|_| CanvasError::InvalidPointsPossible(value.to_string()))?,
            ),
            None => None,
        };
        QuestionUpdateInput::new(
            question_name,
            question_text,
            question_type,
            points_possible,
            self.correct_comments,
            self.incorrect_comments,
            self.neutral_comments,
            self.answers,
        )
    }
}

#[derive(Debug, Serialize)]
struct QuestionCreateForm {
    #[serde(rename = "question[question_name]", skip_serializing_if = "Option::is_none")]
    question_name: Option<String>,
    #[serde(rename = "question[question_text]")]
    question_text: String,
    #[serde(rename = "question[question_type]")]
    question_type: String,
    #[serde(rename = "question[points_possible]", skip_serializing_if = "Option::is_none")]
    points_possible: Option<f64>,
    #[serde(rename = "question[correct_comments]", skip_serializing_if = "Option::is_none")]
    correct_comments: Option<String>,
    #[serde(rename = "question[incorrect_comments]", skip_serializing_if = "Option::is_none")]
    incorrect_comments: Option<String>,
    #[serde(rename = "question[neutral_comments]", skip_serializing_if = "Option::is_none")]
    neutral_comments: Option<String>,
    #[serde(rename = "question[answers]", skip_serializing_if = "Option::is_none")]
    answers: Option<Vec<QuestionAnswerInput>>,
}

#[derive(Debug, Serialize)]
struct QuestionUpdateForm {
    #[serde(rename = "question[question_name]", skip_serializing_if = "Option::is_none")]
    question_name: Option<String>,
    #[serde(rename = "question[question_text]", skip_serializing_if = "Option::is_none")]
    question_text: Option<String>,
    #[serde(rename = "question[question_type]", skip_serializing_if = "Option::is_none")]
    question_type: Option<String>,
    #[serde(rename = "question[points_possible]", skip_serializing_if = "Option::is_none")]
    points_possible: Option<f64>,
    #[serde(rename = "question[correct_comments]", skip_serializing_if = "Option::is_none")]
    correct_comments: Option<String>,
    #[serde(rename = "question[incorrect_comments]", skip_serializing_if = "Option::is_none")]
    incorrect_comments: Option<String>,
    #[serde(rename = "question[neutral_comments]", skip_serializing_if = "Option::is_none")]
    neutral_comments: Option<String>,
    #[serde(rename = "question[answers]", skip_serializing_if = "Option::is_none")]
    answers: Option<Vec<QuestionAnswerInput>>,
}

impl QuestionCreateForm {
    fn from_input(input: &QuestionCreateInput) -> Self {
        Self {
            question_name: input
                .question_name
                .as_ref()
                .map(|value| value.as_str().to_string()),
            question_text: input.question_text.as_str().to_string(),
            question_type: input.question_type.as_str().to_string(),
            points_possible: input.points_possible.map(PointsPossible::value),
            correct_comments: input.correct_comments.clone(),
            incorrect_comments: input.incorrect_comments.clone(),
            neutral_comments: input.neutral_comments.clone(),
            answers: input.answers.clone(),
        }
    }
}

impl QuestionUpdateForm {
    fn from_input(input: &QuestionUpdateInput) -> Self {
        Self {
            question_name: input
                .question_name
                .as_ref()
                .map(|value| value.as_str().to_string()),
            question_text: input
                .question_text
                .as_ref()
                .map(|value| value.as_str().to_string()),
            question_type: input
                .question_type
                .as_ref()
                .map(|value| value.as_str().to_string()),
            points_possible: input.points_possible.map(PointsPossible::value),
            correct_comments: input.correct_comments.clone(),
            incorrect_comments: input.incorrect_comments.clone(),
            neutral_comments: input.neutral_comments.clone(),
            answers: input.answers.clone(),
        }
    }
}

pub fn list_questions(
    config: &CanvasConfig,
    bank_id: QuestionBankId,
) -> Result<Vec<QuestionSummary>, CanvasError> {
    let client = CanvasClient::new(config)?;
    let questions: Vec<ApiQuestion> =
        client.get_paginated(&format!("/question_banks/{}/questions", bank_id.get()))?;
    Ok(questions.into_iter().map(ApiQuestion::into_summary).collect())
}

pub fn create_question(
    config: &CanvasConfig,
    bank_id: QuestionBankId,
    input: &QuestionCreateInput,
) -> Result<QuestionSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let form = QuestionCreateForm::from_input(input);
    let question: ApiQuestion = client
        .post_json(
            &format!("/question_banks/{}/questions", bank_id.get()),
            &form,
        )?;
    Ok(question.into_summary())
}

pub fn update_question(
    config: &CanvasConfig,
    bank_id: QuestionBankId,
    question_id: QuestionId,
    input: &QuestionUpdateInput,
) -> Result<QuestionSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let form = QuestionUpdateForm::from_input(input);
    let question: ApiQuestion = client.put_json(
        &format!(
            "/question_banks/{}/questions/{}",
            bank_id.get(),
            question_id.get()
        ),
        &form,
    )?;
    Ok(question.into_summary())
}

pub fn delete_question(
    config: &CanvasConfig,
    bank_id: QuestionBankId,
    question_id: QuestionId,
) -> Result<QuestionSummary, CanvasError> {
    let client = CanvasClient::new(config)?;
    let question: ApiQuestion = client.delete_json(&format!(
        "/question_banks/{}/questions/{}",
        bank_id.get(),
        question_id.get()
    ))?;
    Ok(question.into_summary())
}
