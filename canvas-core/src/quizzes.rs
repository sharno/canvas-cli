use canvas_models::{
    CourseId, PointsPossible, PublishState, QuizAccessCode, QuizAvailability,
    QuizId, QuizSubmissionId, QuizTimeLimit, QuizTitle, Score,
};
use serde::{Deserialize, Serialize};

use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Deserialize)]
struct ApiQuiz {
    id: u64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    points_possible: Option<f64>,
    #[serde(default)]
    due_at: Option<String>,
    #[serde(default)]
    published: Option<bool>,
    #[serde(default)]
    workflow_state: Option<String>,
    #[serde(default)]
    time_limit: Option<u32>,
    #[serde(default)]
    access_code: Option<String>,
    #[serde(default)]
    unlock_at: Option<String>,
    #[serde(default)]
    lock_at: Option<String>,
}

impl ApiQuiz {
    fn into_summary(self) -> QuizSummary {
        QuizSummary {
            id: self.id,
            title: self.title,
            points_possible: self.points_possible,
            due_at: self.due_at,
            published: self.published,
            workflow_state: self.workflow_state,
            time_limit: self.time_limit,
            access_code: self.access_code,
            unlock_at: self.unlock_at,
            lock_at: self.lock_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuizSummary {
    pub id: u64,
    pub title: Option<String>,
    pub points_possible: Option<f64>,
    pub due_at: Option<String>,
    pub published: Option<bool>,
    pub workflow_state: Option<String>,
    pub time_limit: Option<u32>,
    pub access_code: Option<String>,
    pub unlock_at: Option<String>,
    pub lock_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuizCreateInput {
    pub title: QuizTitle,
    pub points_possible: Option<PointsPossible>,
    pub publish_state: Option<PublishState>,
    pub time_limit: Option<QuizTimeLimit>,
    pub access_code: Option<QuizAccessCode>,
    pub availability: Option<QuizAvailability>,
}

impl QuizCreateInput {
    pub fn new(
        title: QuizTitle,
        points_possible: Option<PointsPossible>,
        publish_state: Option<PublishState>,
        time_limit: Option<QuizTimeLimit>,
        access_code: Option<QuizAccessCode>,
        availability: Option<QuizAvailability>,
    ) -> Self {
        Self {
            title,
            points_possible,
            publish_state,
            time_limit,
            access_code,
            availability,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuizUpdateInput {
    pub title: Option<QuizTitle>,
    pub points_possible: Option<PointsPossible>,
    pub publish_state: Option<PublishState>,
    pub time_limit: Option<QuizTimeLimit>,
    pub access_code: Option<QuizAccessCode>,
    pub availability: Option<QuizAvailability>,
}

impl QuizUpdateInput {
    pub fn new(
        title: Option<QuizTitle>,
        points_possible: Option<PointsPossible>,
        publish_state: Option<PublishState>,
        time_limit: Option<QuizTimeLimit>,
        access_code: Option<QuizAccessCode>,
        availability: Option<QuizAvailability>,
    ) -> Result<Self, CanvasError> {
        if title.is_none()
            && points_possible.is_none()
            && publish_state.is_none()
            && time_limit.is_none()
            && access_code.is_none()
            && availability.is_none()
        {
            return Err(CanvasError::InvalidQuizUpdate("no_fields".to_string()));
        }
        Ok(Self {
            title,
            points_possible,
            publish_state,
            time_limit,
            access_code,
            availability,
        })
    }
}

#[derive(Debug, Serialize)]
struct QuizCreateForm {
    #[serde(rename = "quiz[title]")]
    title: String,
    #[serde(rename = "quiz[points_possible]", skip_serializing_if = "Option::is_none")]
    points_possible: Option<f64>,
    #[serde(rename = "quiz[published]", skip_serializing_if = "Option::is_none")]
    published: Option<bool>,
    #[serde(rename = "quiz[time_limit]", skip_serializing_if = "Option::is_none")]
    time_limit: Option<u32>,
    #[serde(rename = "quiz[access_code]", skip_serializing_if = "Option::is_none")]
    access_code: Option<String>,
    #[serde(rename = "quiz[unlock_at]", skip_serializing_if = "Option::is_none")]
    unlock_at: Option<String>,
    #[serde(rename = "quiz[due_at]", skip_serializing_if = "Option::is_none")]
    due_at: Option<String>,
    #[serde(rename = "quiz[lock_at]", skip_serializing_if = "Option::is_none")]
    lock_at: Option<String>,
}

impl QuizCreateForm {
    fn from_input(input: &QuizCreateInput) -> Self {
        let (unlock_at, due_at, lock_at) =
            availability_fields(input.availability.as_ref());
        Self {
            title: input.title.as_str().to_string(),
            points_possible: input.points_possible.map(PointsPossible::value),
            published: input.publish_state.map(|state| match state {
                PublishState::Published => true,
                PublishState::Unpublished => false,
            }),
            time_limit: input.time_limit.map(QuizTimeLimit::minutes),
            access_code: input
                .access_code
                .as_ref()
                .map(|code| code.as_str().to_string()),
            unlock_at,
            due_at,
            lock_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct QuizUpdateForm {
    #[serde(rename = "quiz[title]", skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(rename = "quiz[points_possible]", skip_serializing_if = "Option::is_none")]
    points_possible: Option<f64>,
    #[serde(rename = "quiz[published]", skip_serializing_if = "Option::is_none")]
    published: Option<bool>,
    #[serde(rename = "quiz[time_limit]", skip_serializing_if = "Option::is_none")]
    time_limit: Option<u32>,
    #[serde(rename = "quiz[access_code]", skip_serializing_if = "Option::is_none")]
    access_code: Option<String>,
    #[serde(rename = "quiz[unlock_at]", skip_serializing_if = "Option::is_none")]
    unlock_at: Option<String>,
    #[serde(rename = "quiz[due_at]", skip_serializing_if = "Option::is_none")]
    due_at: Option<String>,
    #[serde(rename = "quiz[lock_at]", skip_serializing_if = "Option::is_none")]
    lock_at: Option<String>,
}

impl QuizUpdateForm {
    fn from_input(input: &QuizUpdateInput) -> Self {
        let (unlock_at, due_at, lock_at) =
            availability_fields(input.availability.as_ref());
        Self {
            title: input.title.as_ref().map(|title| title.as_str().to_string()),
            points_possible: input.points_possible.map(PointsPossible::value),
            published: input.publish_state.map(|state| match state {
                PublishState::Published => true,
                PublishState::Unpublished => false,
            }),
            time_limit: input.time_limit.map(QuizTimeLimit::minutes),
            access_code: input
                .access_code
                .as_ref()
                .map(|code| code.as_str().to_string()),
            unlock_at,
            due_at,
            lock_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiQuizSubmission {
    id: u64,
    #[serde(default)]
    user_id: u64,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    submitted_at: Option<String>,
    #[serde(default)]
    workflow_state: Option<String>,
}

impl ApiQuizSubmission {
    fn into_summary(self) -> QuizSubmissionSummary {
        QuizSubmissionSummary {
            id: self.id,
            user_id: self.user_id,
            score: self.score,
            submitted_at: self.submitted_at,
            workflow_state: self.workflow_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuizSubmissionSummary {
    pub id: u64,
    pub user_id: u64,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub workflow_state: Option<String>,
}

#[derive(Debug, Serialize)]
struct QuizSubmissionGradeForm {
    #[serde(rename = "quiz_submissions[0][score]")]
    score: f64,
}

fn availability_fields(
    availability: Option<&QuizAvailability>,
) -> (Option<String>, Option<String>, Option<String>) {
    match availability {
        Some(availability) => (
            availability.unlock_at().map(|date| date.as_rfc3339()),
            availability.due_at().map(|date| date.as_rfc3339()),
            availability.lock_at().map(|date| date.as_rfc3339()),
        ),
        None => (None, None, None),
    }
}

pub fn list_quizzes(
    config: &CanvasConfig,
    course_id: CourseId,
) -> Result<Vec<QuizSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let quizzes: Vec<ApiQuiz> = client
        .get_paginated(&format!(
            "/courses/{}/quizzes?per_page=100",
            course_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(quizzes.into_iter().map(ApiQuiz::into_summary).collect())
}

pub fn get_quiz(
    config: &CanvasConfig,
    course_id: CourseId,
    quiz_id: QuizId,
) -> Result<QuizSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let quiz: ApiQuiz = client
        .get_json(&format!(
            "/courses/{}/quizzes/{}",
            course_id.get(),
            quiz_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(quiz.into_summary())
}

pub fn create_quiz(
    config: &CanvasConfig,
    course_id: CourseId,
    input: &QuizCreateInput,
) -> Result<QuizSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = QuizCreateForm::from_input(input);
    let quiz: ApiQuiz = client
        .post_json(&format!("/courses/{}/quizzes", course_id.get()), &form)
        .map_err(CanvasError::Api)?;
    Ok(quiz.into_summary())
}

pub fn update_quiz(
    config: &CanvasConfig,
    course_id: CourseId,
    quiz_id: QuizId,
    input: &QuizUpdateInput,
) -> Result<QuizSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = QuizUpdateForm::from_input(input);
    let quiz: ApiQuiz = client
        .put_json(
            &format!(
                "/courses/{}/quizzes/{}",
                course_id.get(),
                quiz_id.get()
            ),
            &form,
        )
        .map_err(CanvasError::Api)?;
    Ok(quiz.into_summary())
}

pub fn delete_quiz(
    config: &CanvasConfig,
    course_id: CourseId,
    quiz_id: QuizId,
) -> Result<QuizSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let quiz: ApiQuiz = client
        .delete_json(&format!(
            "/courses/{}/quizzes/{}",
            course_id.get(),
            quiz_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(quiz.into_summary())
}

pub fn list_quiz_submissions(
    config: &CanvasConfig,
    course_id: CourseId,
    quiz_id: QuizId,
) -> Result<Vec<QuizSubmissionSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let submissions: Vec<ApiQuizSubmission> = client
        .get_paginated(&format!(
            "/courses/{}/quizzes/{}/submissions?per_page=100",
            course_id.get(),
            quiz_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(submissions
        .into_iter()
        .map(ApiQuizSubmission::into_summary)
        .collect())
}

pub fn grade_quiz_submission(
    config: &CanvasConfig,
    course_id: CourseId,
    quiz_id: QuizId,
    submission_id: QuizSubmissionId,
    score: Score,
) -> Result<QuizSubmissionSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = QuizSubmissionGradeForm {
        score: score.value(),
    };
    let submission: ApiQuizSubmission = client
        .put_json(
            &format!(
                "/courses/{}/quizzes/{}/submissions/{}",
                course_id.get(),
                quiz_id.get(),
                submission_id.get()
            ),
            &form,
        )
        .map_err(CanvasError::Api)?;
    Ok(submission.into_summary())
}

pub fn ensure_quiz_points(
    quiz_id: QuizId,
    points_possible: Option<f64>,
) -> Result<f64, CanvasError> {
    match points_possible {
        Some(points) if points.is_finite() && points > 0.0 => Ok(points),
        _ => Err(CanvasError::MissingQuizPoints(quiz_id.get())),
    }
}

#[cfg(test)]
mod tests {
    use super::{QuizCreateForm, QuizUpdateInput};
    use canvas_models::{
        DueDate, PointsPossible, PublishState, QuizAccessCode, QuizAvailability,
        QuizTimeLimit, QuizTitle,
    };

    #[test]
    fn quiz_update_requires_fields() {
        let update = QuizUpdateInput::new(None, None, None, None, None, None);
        assert!(update.is_err());
    }

    #[test]
    fn quiz_create_form_maps_fields() {
        let title: QuizTitle = "Midterm".parse().expect("title");
        let points: PointsPossible = "20".parse().expect("points");
        let time_limit: QuizTimeLimit = "30".parse().expect("time limit");
        let access_code: QuizAccessCode = "secret".parse().expect("access");
        let unlock_at: DueDate =
            "2025-01-01T00:00:00Z".parse().expect("unlock");
        let due_at: DueDate =
            "2025-01-02T00:00:00Z".parse().expect("due");
        let lock_at: DueDate =
            "2025-01-03T00:00:00Z".parse().expect("lock");
        let availability =
            QuizAvailability::new(Some(unlock_at), Some(due_at), Some(lock_at))
                .expect("availability");
        let input = super::QuizCreateInput::new(
            title,
            Some(points),
            Some(PublishState::Published),
            Some(time_limit),
            Some(access_code),
            Some(availability),
        );
        let form = QuizCreateForm::from_input(&input);
        assert_eq!(form.title, "Midterm");
        assert_eq!(form.points_possible, Some(20.0));
        assert_eq!(form.published, Some(true));
        assert_eq!(form.time_limit, Some(30));
        assert_eq!(form.access_code.as_deref(), Some("secret"));
        assert_eq!(form.unlock_at.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(form.due_at.as_deref(), Some("2025-01-02T00:00:00Z"));
        assert_eq!(form.lock_at.as_deref(), Some("2025-01-03T00:00:00Z"));
    }
}
