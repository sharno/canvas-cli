use reqwest::blocking::Client;

use crate::{CanvasConfig, CanvasError};

pub fn auth_check(config: &CanvasConfig) -> Result<(), CanvasError> {
    let url = format!("{}/api/v1/users/self", config.auth.host.as_str());
    let client = Client::new();
    let response = client
        .get(url)
        .bearer_auth(config.auth.token.as_str())
        .send()
        .map_err(|err| CanvasError::Http(err.to_string()))?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    Err(CanvasError::AuthCheckFailed(format!(
        "unexpected status {status}"
    )))
}
