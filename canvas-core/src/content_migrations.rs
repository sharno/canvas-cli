use std::path::PathBuf;
use std::time::Duration;

use canvas_models::{ContentMigrationId, ContentMigrationType, CourseId};
use reqwest::blocking::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

use crate::content::UploadFileInput;
use crate::{CanvasClient, CanvasConfig, CanvasError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentMigrationState {
    PreProcessing,
    PreProcessed,
    Running,
    WaitingForSelect,
    Completed,
    Failed,
    Other(String),
}

impl ContentMigrationState {
    fn from_raw(raw: Option<&str>) -> Self {
        let value = raw.unwrap_or("").trim();
        if value.is_empty() {
            return ContentMigrationState::Other("unknown".to_string());
        }
        match value {
            "pre_processing" => ContentMigrationState::PreProcessing,
            "pre_processed" => ContentMigrationState::PreProcessed,
            "running" => ContentMigrationState::Running,
            "waiting_for_select" => ContentMigrationState::WaitingForSelect,
            "completed" => ContentMigrationState::Completed,
            "failed" => ContentMigrationState::Failed,
            other => ContentMigrationState::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ContentMigrationState::PreProcessing => "pre_processing",
            ContentMigrationState::PreProcessed => "pre_processed",
            ContentMigrationState::Running => "running",
            ContentMigrationState::WaitingForSelect => "waiting_for_select",
            ContentMigrationState::Completed => "completed",
            ContentMigrationState::Failed => "failed",
            ContentMigrationState::Other(raw) => raw.as_str(),
        }
    }

    fn is_complete(&self) -> bool {
        matches!(self, ContentMigrationState::Completed)
    }

    fn is_failed(&self) -> bool {
        matches!(self, ContentMigrationState::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentMigrationTypeValue {
    Known(ContentMigrationType),
    Other(String),
}

impl ContentMigrationTypeValue {
    fn from_raw(raw: Option<String>) -> Option<Self> {
        let raw = raw?;
        match raw.parse::<ContentMigrationType>() {
            Ok(parsed) => Some(ContentMigrationTypeValue::Known(parsed)),
            Err(_) => Some(ContentMigrationTypeValue::Other(raw)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ContentMigrationTypeValue::Known(kind) => kind.as_str(),
            ContentMigrationTypeValue::Other(raw) => raw.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentMigrationSummary {
    pub id: u64,
    pub migration_type: Option<ContentMigrationTypeValue>,
    pub migration_type_title: Option<String>,
    pub workflow_state: ContentMigrationState,
    pub progress_url: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub user_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ContentMigrationProgress {
    pub completion: Option<f64>,
    pub workflow_state: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ContentMigrationCreateInput {
    CourseCopy { source_course_id: CourseId },
    FileImport { file: PathBuf },
}

impl ContentMigrationCreateInput {
    fn migration_type(&self) -> ContentMigrationType {
        match self {
            ContentMigrationCreateInput::CourseCopy { .. } => {
                ContentMigrationType::CourseCopy
            }
            ContentMigrationCreateInput::FileImport { .. } => {
                ContentMigrationType::FileImport
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ContentMigrationWaitOptions {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for ContentMigrationWaitOptions {
    fn default() -> Self {
        Self {
            max_attempts: 20,
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiContentMigration {
    id: u64,
    #[serde(default)]
    migration_type: Option<String>,
    #[serde(default)]
    migration_type_title: Option<String>,
    #[serde(default)]
    workflow_state: Option<String>,
    #[serde(default)]
    progress_url: Option<String>,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default)]
    finished_at: Option<String>,
    #[serde(default)]
    user_id: Option<u64>,
    #[serde(default)]
    pre_attachment: Option<ApiPreAttachment>,
}

impl ApiContentMigration {
    fn into_summary(self) -> ContentMigrationSummary {
        ContentMigrationSummary {
            id: self.id,
            migration_type: ContentMigrationTypeValue::from_raw(self.migration_type),
            migration_type_title: self.migration_type_title,
            workflow_state: ContentMigrationState::from_raw(self.workflow_state.as_deref()),
            progress_url: self.progress_url,
            started_at: self.started_at,
            finished_at: self.finished_at,
            user_id: self.user_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApiPreAttachment {
    #[serde(default)]
    upload_url: Option<String>,
    #[serde(default)]
    upload_params: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiProgress {
    #[serde(default)]
    completion: Option<f64>,
    #[serde(default)]
    workflow_state: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

impl From<ApiProgress> for ContentMigrationProgress {
    fn from(progress: ApiProgress) -> Self {
        Self {
            completion: progress.completion,
            workflow_state: progress.workflow_state,
            message: progress.message,
        }
    }
}

#[derive(Debug, Serialize)]
struct ContentMigrationCreateForm {
    migration_type: String,
    #[serde(
        rename = "settings[source_course_id]",
        skip_serializing_if = "Option::is_none"
    )]
    source_course_id: Option<u64>,
    #[serde(
        rename = "pre_attachment[name]",
        skip_serializing_if = "Option::is_none"
    )]
    pre_attachment_name: Option<String>,
    #[serde(
        rename = "pre_attachment[size]",
        skip_serializing_if = "Option::is_none"
    )]
    pre_attachment_size: Option<u64>,
    #[serde(
        rename = "pre_attachment[content_type]",
        skip_serializing_if = "Option::is_none"
    )]
    pre_attachment_content_type: Option<String>,
}

pub fn list_content_migrations(
    config: &CanvasConfig,
    course_id: CourseId,
) -> Result<Vec<ContentMigrationSummary>, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let migrations: Vec<ApiContentMigration> = client
        .get_paginated(&format!(
            "/courses/{}/content_migrations?per_page=100",
            course_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(migrations
        .into_iter()
        .map(ApiContentMigration::into_summary)
        .collect())
}

pub fn get_content_migration(
    config: &CanvasConfig,
    course_id: CourseId,
    migration_id: ContentMigrationId,
) -> Result<ContentMigrationSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let migration: ApiContentMigration = client
        .get_json(&format!(
            "/courses/{}/content_migrations/{}",
            course_id.get(),
            migration_id.get()
        ))
        .map_err(CanvasError::Api)?;
    Ok(migration.into_summary())
}

pub fn create_content_migration(
    config: &CanvasConfig,
    course_id: CourseId,
    input: &ContentMigrationCreateInput,
) -> Result<ContentMigrationSummary, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let (form, file) = match input {
        ContentMigrationCreateInput::CourseCopy { source_course_id } => (
            ContentMigrationCreateForm {
                migration_type: input.migration_type().as_str().to_string(),
                source_course_id: Some(source_course_id.get()),
                pre_attachment_name: None,
                pre_attachment_size: None,
                pre_attachment_content_type: None,
            },
            None,
        ),
        ContentMigrationCreateInput::FileImport { file } => {
            let upload = UploadFileInput::from_path(file.clone(), None)?;
            (
                ContentMigrationCreateForm {
                    migration_type: input.migration_type().as_str().to_string(),
                    source_course_id: None,
                    pre_attachment_name: Some(upload.file_name.clone()),
                    pre_attachment_size: Some(upload.file_size),
                    pre_attachment_content_type: Some(upload.content_type.clone()),
                },
                Some(upload),
            )
        }
    };
    let migration: ApiContentMigration = client
        .post_json(
            &format!("/courses/{}/content_migrations", course_id.get()),
            &form,
        )
        .map_err(CanvasError::Api)?;
    if let Some(file) = file {
        let pre_attachment = migration.pre_attachment.as_ref().ok_or_else(|| {
            CanvasError::InvalidContentMigrationCreate("missing_pre_attachment".to_string())
        })?;
        upload_pre_attachment(pre_attachment, &file)?;
        let migration_id = migration
            .id
            .to_string()
            .parse::<ContentMigrationId>()
            .map_err(|_| {
                CanvasError::InvalidContentMigrationCreate(
                    "invalid_migration_id".to_string(),
                )
            })?;
        return get_content_migration(config, course_id, migration_id);
    }
    Ok(migration.into_summary())
}

pub fn get_content_migration_progress(
    config: &CanvasConfig,
    progress_url: &str,
) -> Result<ContentMigrationProgress, CanvasError> {
    let client = CanvasClient::new(config).map_err(CanvasError::Api)?;
    let progress: ApiProgress = client.get_json(progress_url).map_err(CanvasError::Api)?;
    Ok(progress.into())
}

pub fn wait_for_content_migration(
    config: &CanvasConfig,
    course_id: CourseId,
    migration_id: ContentMigrationId,
    options: ContentMigrationWaitOptions,
) -> Result<ContentMigrationSummary, CanvasError> {
    let mut attempts = 0;
    loop {
        let migration = get_content_migration(config, course_id, migration_id)?;
        if migration.workflow_state.is_complete() {
            return Ok(migration);
        }
        if migration.workflow_state.is_failed() {
            return Err(CanvasError::ContentMigrationFailed("failed".to_string()));
        }
        attempts += 1;
        if attempts >= options.max_attempts {
            return Err(CanvasError::ContentMigrationTimeout("max_attempts".to_string()));
        }
        std::thread::sleep(backoff_delay(attempts - 1, options));
    }
}

fn backoff_delay(attempt: usize, options: ContentMigrationWaitOptions) -> Duration {
    let multiplier = 2_u32.saturating_pow(attempt as u32);
    let delay = options.base_delay.saturating_mul(multiplier);
    if delay > options.max_delay {
        options.max_delay
    } else {
        delay
    }
}

fn upload_pre_attachment(
    pre_attachment: &ApiPreAttachment,
    file: &UploadFileInput,
) -> Result<(), CanvasError> {
    let upload_url = pre_attachment.upload_url.as_ref().ok_or_else(|| {
        let detail = pre_attachment
            .message
            .clone()
            .unwrap_or_else(|| "missing_upload_url".to_string());
        CanvasError::InvalidContentMigrationCreate(detail)
    })?;
    let mut form = Form::new();
    for (key, value) in &pre_attachment.upload_params {
        if let Some(text) = json_value_to_string(Some(value)) {
            form = form.text(key.clone(), text);
        }
    }
    let mut file_part = Part::file(&file.path)
        .map_err(|err| CanvasError::InvalidContentMigrationCreate(err.to_string()))?;
    file_part = file_part.file_name(file.file_name.clone());
    form = form.part("file", file_part);
    let response = reqwest::blocking::Client::new()
        .post(upload_url)
        .multipart(form)
        .send()
        .map_err(|err| CanvasError::InvalidContentMigrationCreate(err.to_string()))?;
    if !response.status().is_success() {
        return Err(CanvasError::InvalidContentMigrationCreate(format!(
            "upload_status:{}",
            response.status().as_u16()
        )));
    }
    Ok(())
}

fn json_value_to_string(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{backoff_delay, ContentMigrationState, ContentMigrationWaitOptions};
    use std::time::Duration;

    #[test]
    fn content_migration_state_maps_running() {
        let state = ContentMigrationState::from_raw(Some("running"));
        assert_eq!(state.as_str(), "running");
    }

    #[test]
    fn backoff_delay_caps_at_max() {
        let options = ContentMigrationWaitOptions {
            max_attempts: 3,
            base_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(6),
        };
        let delay = backoff_delay(2, options);
        assert_eq!(delay.as_secs(), 6);
    }
}
