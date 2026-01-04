use canvas_models::CourseId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("invalid course id: {0}")]
    InvalidCourseId(String),
}

pub fn parse_course_id(raw: &str) -> Result<CourseId, CanvasError> {
    raw.parse::<CourseId>()
        .map_err(|_| CanvasError::InvalidCourseId(raw.to_string()))
}
