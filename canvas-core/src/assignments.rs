use canvas_models::{
    AssignmentId, AssignmentName, CourseId, DueDate, PointsPossible, PublishState,
    RubricAssessment, Score, UserId,
};
use serde::{Deserialize, Serialize};

use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Deserialize)]
struct ApiAssignment {
    id: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    points_possible: Option<f64>,
    #[serde(default)]
    due_at: Option<String>,
    #[serde(default)]
    published: Option<bool>,
    #[serde(default)]
    workflow_state: Option<String>,
}

impl ApiAssignment {
    fn into_summary(self) -> AssignmentSummary {
        AssignmentSummary {
            id: self.id,
            name: self.name,
            points_possible: self.points_possible,
            due_at: self.due_at,
            published: self.published,
            workflow_state: self.workflow_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentSummary {
    pub id: u64,
    pub name: Option<String>,
    pub points_possible: Option<f64>,
    pub due_at: Option<String>,
    pub published: Option<bool>,
    pub workflow_state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AssignmentCreateInput {
    pub name: AssignmentName,
    pub points_possible: Option<PointsPossible>,
    pub due_at: Option<DueDate>,
    pub publish_state: Option<PublishState>,
}

impl AssignmentCreateInput {
    pub fn new(
        name: AssignmentName,
        points_possible: Option<PointsPossible>,
        due_at: Option<DueDate>,
        publish_state: Option<PublishState>,
    ) -> Self {
        Self {
            name,
            points_possible,
            due_at,
            publish_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentUpdateInput {
    pub name: Option<AssignmentName>,
    pub points_possible: Option<PointsPossible>,
    pub due_at: Option<DueDate>,
    pub publish_state: Option<PublishState>,
}

impl AssignmentUpdateInput {
    pub fn new(
        name: Option<AssignmentName>,
        points_possible: Option<PointsPossible>,
        due_at: Option<DueDate>,
        publish_state: Option<PublishState>,
    ) -> Result<Self, CanvasError> {
        if name.is_none()
            && points_possible.is_none()
            && due_at.is_none()
            && publish_state.is_none()
        {
            return Err(CanvasError::InvalidAssignmentUpdate(
                "no_fields".to_string(),
            ));
        }
        Ok(Self {
            name,
            points_possible,
            due_at,
            publish_state,
        })
    }
}

#[derive(Debug, Serialize)]
struct AssignmentCreateForm {
    #[serde(rename = "assignment[name]")]
    name: String,
    #[serde(
        rename = "assignment[points_possible]",
        skip_serializing_if = "Option::is_none"
    )]
    points_possible: Option<f64>,
    #[serde(
        rename = "assignment[due_at]",
        skip_serializing_if = "Option::is_none"
    )]
    due_at: Option<String>,
    #[serde(
        rename = "assignment[published]",
        skip_serializing_if = "Option::is_none"
    )]
    published: Option<bool>,
}

impl AssignmentCreateForm {
    fn from_input(input: &AssignmentCreateInput) -> Self {
        Self {
            name: input.name.as_str().to_string(),
            points_possible: input.points_possible.map(PointsPossible::value),
            due_at: input.due_at.map(|date| date.as_rfc3339()),
            published: input.publish_state.map(|state| match state {
                PublishState::Published => true,
                PublishState::Unpublished => false,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
struct AssignmentUpdateForm {
    #[serde(
        rename = "assignment[name]",
        skip_serializing_if = "Option::is_none"
    )]
    name: Option<String>,
    #[serde(
        rename = "assignment[points_possible]",
        skip_serializing_if = "Option::is_none"
    )]
    points_possible: Option<f64>,
    #[serde(
        rename = "assignment[due_at]",
        skip_serializing_if = "Option::is_none"
    )]
    due_at: Option<String>,
    #[serde(
        rename = "assignment[published]",
        skip_serializing_if = "Option::is_none"
    )]
    published: Option<bool>,
}

impl AssignmentUpdateForm {
    fn from_input(input: &AssignmentUpdateInput) -> Self {
        Self {
            name: input.name.as_ref().map(|name| name.as_str().to_string()),
            points_possible: input.points_possible.map(PointsPossible::value),
            due_at: input.due_at.map(|date| date.as_rfc3339()),
            published: input.publish_state.map(|state| match state {
                PublishState::Published => true,
                PublishState::Unpublished => false,
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiSubmission {
    id: Option<u64>,
    #[serde(default)]
    user_id: u64,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    submitted_at: Option<String>,
    #[serde(default)]
    graded_at: Option<String>,
    #[serde(default)]
    workflow_state: Option<String>,
}

impl ApiSubmission {
    fn into_summary(self) -> SubmissionSummary {
        SubmissionSummary {
            id: self.id,
            user_id: self.user_id,
            score: self.score,
            submitted_at: self.submitted_at,
            graded_at: self.graded_at,
            workflow_state: self.workflow_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubmissionSummary {
    pub id: Option<u64>,
    pub user_id: u64,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub workflow_state: Option<String>,
}

#[derive(Debug, Serialize)]
struct SubmissionGradeForm {
    #[serde(rename = "submission[posted_grade]")]
    posted_grade: f64,
    #[serde(rename = "rubric_assessment", skip_serializing_if = "Option::is_none")]
    rubric_assessment: Option<RubricAssessment>,
}

#[derive(Debug, Serialize)]
struct SubmissionGradeValueForm {
    #[serde(rename = "submission[posted_grade]")]
    posted_grade: f64,
}

pub fn list_assignments(
    config: &CanvasConfig,
    course_id: CourseId,
) -> Result<Vec<AssignmentSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let assignments: Vec<ApiAssignment> = client
        .get_paginated(&format!(
            "/courses/{}/assignments?per_page=100",
            course_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(assignments.into_iter().map(ApiAssignment::into_summary).collect())
}

pub fn get_assignment(
    config: &CanvasConfig,
    course_id: CourseId,
    assignment_id: AssignmentId,
) -> Result<AssignmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let assignment: ApiAssignment = client
        .get_json(&format!(
            "/courses/{}/assignments/{}",
            course_id.get(),
            assignment_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(assignment.into_summary())
}

pub fn create_assignment(
    config: &CanvasConfig,
    course_id: CourseId,
    input: &AssignmentCreateInput,
) -> Result<AssignmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = AssignmentCreateForm::from_input(input);
    let assignment: ApiAssignment = client
        .post_json(&format!("/courses/{}/assignments", course_id.get()), &form)
        .map_err(CanvasError::Api)?;
    Ok(assignment.into_summary())
}

pub fn update_assignment(
    config: &CanvasConfig,
    course_id: CourseId,
    assignment_id: AssignmentId,
    input: &AssignmentUpdateInput,
) -> Result<AssignmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = AssignmentUpdateForm::from_input(input);
    let assignment: ApiAssignment = client
        .put_json(
            &format!(
                "/courses/{}/assignments/{}",
                course_id.get(),
                assignment_id.get()
            ),
            &form,
        )
        .map_err(CanvasError::Api)?;
    Ok(assignment.into_summary())
}

pub fn delete_assignment(
    config: &CanvasConfig,
    course_id: CourseId,
    assignment_id: AssignmentId,
) -> Result<AssignmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let assignment: ApiAssignment = client
        .delete_json(&format!(
            "/courses/{}/assignments/{}",
            course_id.get(),
            assignment_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(assignment.into_summary())
}

pub fn list_submissions(
    config: &CanvasConfig,
    course_id: CourseId,
    assignment_id: AssignmentId,
) -> Result<Vec<SubmissionSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let submissions: Vec<ApiSubmission> = client
        .get_paginated(&format!(
            "/courses/{}/assignments/{}/submissions?per_page=100",
            course_id.get(),
            assignment_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(submissions.into_iter().map(ApiSubmission::into_summary).collect())
}

pub fn grade_submission(
    config: &CanvasConfig,
    course_id: CourseId,
    assignment_id: AssignmentId,
    user_id: UserId,
    score: Score,
    rubric_assessment: Option<RubricAssessment>,
) -> Result<SubmissionSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let assignment_path = format!(
        "/courses/{}/assignments/{}/submissions/{}",
        course_id.get(),
        assignment_id.get(),
        user_id.get()
    );
    let submission: ApiSubmission = match rubric_assessment {
        Some(rubric_assessment) => {
            let form = SubmissionGradeForm {
                posted_grade: score.value(),
                rubric_assessment: Some(rubric_assessment),
            };
            client.put_json(&assignment_path, &form)
        }
        None => {
            let form = SubmissionGradeValueForm {
                posted_grade: score.value(),
            };
            client.put_json(&assignment_path, &form)
        }
    }
    .map_err(CanvasError::Api)?;
    Ok(submission.into_summary())
}

pub fn ensure_assignment_points(
    assignment_id: AssignmentId,
    points_possible: Option<f64>,
) -> Result<f64, CanvasError> {
    match points_possible {
        Some(points) if points.is_finite() && points > 0.0 => Ok(points),
        _ => Err(CanvasError::MissingAssignmentPoints(
            assignment_id.get(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{AssignmentUpdateInput, AssignmentUpdateForm, SubmissionGradeForm};
    use canvas_models::{AssignmentName, DueDate, PointsPossible, PublishState, Score};

    #[test]
    fn assignment_update_requires_fields() {
        let update = AssignmentUpdateInput::new(None, None, None, None);
        assert!(update.is_err());
    }

    #[test]
    fn assignment_update_form_maps_fields() {
        let name: AssignmentName = "Quiz 1".parse().expect("name");
        let points: PointsPossible = "10".parse().expect("points");
        let due_at: DueDate = "2025-01-01T00:00:00Z".parse().expect("due");
        let update = AssignmentUpdateInput::new(
            Some(name),
            Some(points),
            Some(due_at),
            Some(PublishState::Published),
        )
        .expect("update");
        let form = AssignmentUpdateForm::from_input(&update);
        assert_eq!(form.name.as_deref(), Some("Quiz 1"));
        assert_eq!(form.points_possible, Some(10.0));
        assert_eq!(form.due_at.as_deref(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(form.published, Some(true));
    }

    #[test]
    fn submission_grade_form_includes_rubric() {
        let score = Score::new(5.0, 10.0).expect("score");
        let form = SubmissionGradeForm {
            posted_grade: score.value(),
            rubric_assessment: None,
        };
        let serialized = serde_json::to_string(&form).expect("json");
        assert!(serialized.contains("posted_grade"));
    }
}
