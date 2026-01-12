use canvas_models::{
    CourseId, EnrollmentId, SectionId, SectionName, UserId, UserRole,
};
use serde::{Deserialize, Serialize};

use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Deserialize)]
struct ApiSection {
    id: u64,
    #[serde(default)]
    course_id: Option<u64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    total_students: Option<u32>,
}

impl ApiSection {
    fn into_summary(self) -> SectionSummary {
        SectionSummary {
            id: self.id,
            course_id: self.course_id,
            name: self.name,
            enrollment_count: self.total_students,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SectionSummary {
    pub id: u64,
    pub course_id: Option<u64>,
    pub name: Option<String>,
    pub enrollment_count: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SectionCreateInput {
    pub name: SectionName,
}

impl SectionCreateInput {
    pub fn new(name: SectionName) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub struct SectionUpdateInput {
    pub name: Option<SectionName>,
}

impl SectionUpdateInput {
    pub fn new(name: Option<SectionName>) -> Result<Self, CanvasError> {
        if name.is_none() {
            return Err(CanvasError::InvalidSectionUpdate(
                "no_fields".to_string(),
            ));
        }
        Ok(Self { name })
    }
}

#[derive(Debug, Serialize)]
struct SectionCreateForm {
    #[serde(rename = "course_section[name]")]
    name: String,
}

#[derive(Debug, Serialize)]
struct SectionUpdateForm {
    #[serde(
        rename = "course_section[name]",
        skip_serializing_if = "Option::is_none"
    )]
    name: Option<String>,
}

pub fn list_sections(
    config: &CanvasConfig,
    course_id: CourseId,
) -> Result<Vec<SectionSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let sections: Vec<ApiSection> = client
        .get_paginated(&format!(
            "/courses/{}/sections?per_page=100",
            course_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(sections.into_iter().map(ApiSection::into_summary).collect())
}

pub fn create_section(
    config: &CanvasConfig,
    course_id: CourseId,
    input: &SectionCreateInput,
) -> Result<SectionSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = SectionCreateForm {
        name: input.name.as_str().to_string(),
    };
    let section: ApiSection = client
        .post_json(&format!("/courses/{}/sections", course_id.get()), &form)
        .map_err(CanvasError::Api)?;
    Ok(section.into_summary())
}

pub fn update_section(
    config: &CanvasConfig,
    section_id: SectionId,
    input: &SectionUpdateInput,
) -> Result<SectionSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = SectionUpdateForm {
        name: input.name.as_ref().map(|value| value.as_str().to_string()),
    };
    let section: ApiSection = client
        .put_json(
            &format!("/sections/{}", section_id.get()),
            &form,
        )
        .map_err(CanvasError::Api)?;
    Ok(section.into_summary())
}

pub fn delete_section(
    config: &CanvasConfig,
    section_id: SectionId,
) -> Result<SectionSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let section: ApiSection = client
        .delete_json(&format!("/sections/{}", section_id.get()))
        .map_err(CanvasError::Api)?;
    Ok(section.into_summary())
}

#[derive(Debug, Deserialize)]
struct ApiEnrollment {
    id: u64,
    #[serde(default)]
    user_id: Option<u64>,
    #[serde(default)]
    course_section_id: Option<u64>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    enrollment_state: Option<String>,
}

impl ApiEnrollment {
    fn into_summary(self) -> EnrollmentSummary {
        EnrollmentSummary {
            id: self.id,
            user_id: self.user_id,
            section_id: self.course_section_id,
            role: self.role,
            enrollment_state: self.enrollment_state,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnrollmentSummary {
    pub id: u64,
    pub user_id: Option<u64>,
    pub section_id: Option<u64>,
    pub role: Option<String>,
    pub enrollment_state: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct EnrollmentCreateInput {
    pub user_id: UserId,
    pub role: UserRole,
    pub limit_privileges_to_section: bool,
}

impl EnrollmentCreateInput {
    pub fn new(
        user_id: UserId,
        role: UserRole,
        limit_privileges_to_section: bool,
    ) -> Self {
        Self {
            user_id,
            role,
            limit_privileges_to_section,
        }
    }
}

#[derive(Debug, Serialize)]
struct EnrollmentCreateForm {
    #[serde(rename = "enrollment[user_id]")]
    user_id: u64,
    #[serde(rename = "enrollment[type]")]
    enrollment_type: String,
    #[serde(
        rename = "enrollment[limit_privileges_to_course_section]",
        skip_serializing_if = "Option::is_none"
    )]
    limit_privileges_to_course_section: Option<bool>,
}

fn enrollment_type_for_role(role: UserRole) -> &'static str {
    match role {
        UserRole::Student => "StudentEnrollment",
        UserRole::Ta => "TaEnrollment",
        UserRole::Teacher => "TeacherEnrollment",
    }
}

pub fn list_enrollments(
    config: &CanvasConfig,
    section_id: SectionId,
    role_filter: Option<UserRole>,
) -> Result<Vec<EnrollmentSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let mut path = format!(
        "/sections/{}/enrollments?per_page=100",
        section_id.get()
    );
    if let Some(role) = role_filter {
        let enrollment_type = enrollment_type_for_role(role);
        path.push_str(&format!("&type[]={}", enrollment_type));
    }
    let enrollments: Vec<ApiEnrollment> =
        client.get_paginated(&path).map_err(CanvasError::Api)?;
    Ok(enrollments
        .into_iter()
        .map(ApiEnrollment::into_summary)
        .collect())
}

pub fn add_enrollment(
    config: &CanvasConfig,
    section_id: SectionId,
    input: &EnrollmentCreateInput,
) -> Result<EnrollmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let form = EnrollmentCreateForm {
        user_id: input.user_id.get(),
        enrollment_type: enrollment_type_for_role(input.role).to_string(),
        limit_privileges_to_course_section: Some(
            input.limit_privileges_to_section,
        ),
    };
    let enrollment: ApiEnrollment = client
        .post_json(
            &format!("/sections/{}/enrollments", section_id.get()),
            &form,
        )
        .map_err(CanvasError::Api)?;
    Ok(enrollment.into_summary())
}

pub fn remove_enrollment(
    config: &CanvasConfig,
    section_id: SectionId,
    enrollment_id: EnrollmentId,
) -> Result<EnrollmentSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let enrollment: ApiEnrollment = client
        .delete_json(&format!(
            "/sections/{}/enrollments/{}",
            section_id.get(),
            enrollment_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(enrollment.into_summary())
}

#[cfg(test)]
mod tests {
    use super::SectionUpdateInput;
    use canvas_models::SectionName;

    #[test]
    fn section_update_requires_fields() {
        let update = SectionUpdateInput::new(None);
        assert!(update.is_err());
    }

    #[test]
    fn section_update_accepts_name() {
        let name = "Section A".parse::<SectionName>().expect("name");
        let update = SectionUpdateInput::new(Some(name));
        assert!(update.is_ok());
    }
}
