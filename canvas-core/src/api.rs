use std::time::Duration;

use canvas_models::CanvasToken;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

use crate::CanvasConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorCode {
    Network,
    Timeout,
    RateLimited,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    InvalidRequest,
    ServerError,
    Decode,
    UnexpectedStatus,
}

impl ApiErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            ApiErrorCode::Network => "network_error",
            ApiErrorCode::Timeout => "timeout",
            ApiErrorCode::RateLimited => "rate_limited",
            ApiErrorCode::Unauthorized => "unauthorized",
            ApiErrorCode::Forbidden => "forbidden",
            ApiErrorCode::NotFound => "not_found",
            ApiErrorCode::Conflict => "conflict",
            ApiErrorCode::InvalidRequest => "invalid_request",
            ApiErrorCode::ServerError => "server_error",
            ApiErrorCode::Decode => "decode_error",
            ApiErrorCode::UnexpectedStatus => "unexpected_status",
        }
    }
}

#[derive(Debug, Error, Clone)]
#[error("{message}")]
pub struct ApiError {
    code: ApiErrorCode,
    message: String,
    status: Option<u16>,
    request_id: Option<String>,
    body: Option<String>,
}

impl ApiError {
    fn from_status(status: StatusCode, body: String, request_id: Option<String>) -> Self {
        let code = match status {
            StatusCode::UNAUTHORIZED => ApiErrorCode::Unauthorized,
            StatusCode::FORBIDDEN => ApiErrorCode::Forbidden,
            StatusCode::NOT_FOUND => ApiErrorCode::NotFound,
            StatusCode::CONFLICT => ApiErrorCode::Conflict,
            StatusCode::BAD_REQUEST => ApiErrorCode::InvalidRequest,
            StatusCode::TOO_MANY_REQUESTS => ApiErrorCode::RateLimited,
            status if status.is_server_error() => ApiErrorCode::ServerError,
            _ => ApiErrorCode::UnexpectedStatus,
        };
        let message = format!("request failed with status {}", status.as_u16());
        Self {
            code,
            message,
            status: Some(status.as_u16()),
            request_id,
            body: if body.trim().is_empty() { None } else { Some(body) },
        }
    }

    fn from_reqwest(err: reqwest::Error) -> Self {
        let code = if err.is_timeout() {
            ApiErrorCode::Timeout
        } else {
            ApiErrorCode::Network
        };
        Self {
            code,
            message: err.to_string(),
            status: err.status().map(|status| status.as_u16()),
            request_id: None,
            body: None,
        }
    }

    pub fn code(&self) -> ApiErrorCode {
        self.code
    }

    pub fn status(&self) -> Option<u16> {
        self.status
    }

    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasClient {
    client: Client,
    base_url: String,
    token: CanvasToken,
    retry: RetryPolicy,
}

impl CanvasClient {
    pub fn new(config: &CanvasConfig) -> Result<Self, ApiError> {
        let client = Client::new();
        Ok(Self {
            client,
            base_url: format!("{}/api/v1", config.auth.host.as_str()),
            token: config.auth.token.clone(),
            retry: RetryPolicy::default(),
        })
    }

    pub fn with_retry_policy(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self.send_with_retry(Method::GET, &self.build_url(path))?;
        response.json::<T>().map_err(ApiError::from_reqwest)
    }

    pub fn get_paginated<T: DeserializeOwned>(&self, path: &str) -> Result<Vec<T>, ApiError> {
        let mut url = self.build_url(path);
        let mut items = Vec::new();
        loop {
            let response = self.send_with_retry(Method::GET, &url)?;
            let headers = response.headers().clone();
            let mut page = response
                .json::<Vec<T>>()
                .map_err(ApiError::from_reqwest)?;
            items.append(&mut page);
            match next_link(&headers) {
                Some(next) => url = next,
                None => break,
            }
        }
        Ok(items)
    }

    pub fn put_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.build_url(path);
        let response = self.send_with_retry_request(Method::PUT, &url, |client, method, url| {
            client.request(method, url).json(body)
        })?;
        response.json::<T>().map_err(ApiError::from_reqwest)
    }

    pub fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.build_url(path);
        let response = self.send_with_retry_request(Method::POST, &url, |client, method, url| {
            client.request(method, url).json(body)
        })?;
        response.json::<T>().map_err(ApiError::from_reqwest)
    }

    pub fn delete_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let response = self.send_with_retry(Method::DELETE, &self.build_url(path))?;
        response.json::<T>().map_err(ApiError::from_reqwest)
    }

    fn build_url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            return path.to_string();
        }
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }

    fn send_with_retry(&self, method: Method, url: &str) -> Result<Response, ApiError> {
        self.send_with_retry_request(method, url, |client, method, url| {
            client.request(method, url)
        })
    }

    fn send_with_retry_request<F>(
        &self,
        method: Method,
        url: &str,
        mut build: F,
    ) -> Result<Response, ApiError>
    where
        F: FnMut(&Client, Method, &str) -> reqwest::blocking::RequestBuilder,
    {
        let mut attempt = 0;
        loop {
            let response = build(&self.client, method.clone(), url)
                .bearer_auth(self.token.as_str())
                .send();
            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }
                    let status = response.status();
                    let request_id = response
                        .headers()
                        .get("x-request-id")
                        .and_then(|value| value.to_str().ok())
                        .map(|value| value.to_string());
                    if should_retry(&method, Some(status))
                        && attempt < self.retry.max_retries
                    {
                        let delay = retry_delay(attempt, response.headers(), &self.retry);
                        std::thread::sleep(delay);
                        attempt += 1;
                        continue;
                    }
                    let body = response.text().unwrap_or_default();
                    return Err(ApiError::from_status(status, body, request_id));
                }
                Err(err) => {
                    if should_retry_error(&method, &err) && attempt < self.retry.max_retries {
                        let delay = retry_delay(attempt, &HeaderMap::new(), &self.retry);
                        std::thread::sleep(delay);
                        attempt += 1;
                        continue;
                    }
                    return Err(ApiError::from_reqwest(err));
                }
            }
        }
    }
}

fn should_retry(method: &Method, status: Option<StatusCode>) -> bool {
    if !is_idempotent(method) {
        return false;
    }
    matches!(
        status,
        Some(StatusCode::TOO_MANY_REQUESTS)
            | Some(StatusCode::BAD_GATEWAY)
            | Some(StatusCode::SERVICE_UNAVAILABLE)
            | Some(StatusCode::GATEWAY_TIMEOUT)
            | Some(StatusCode::INTERNAL_SERVER_ERROR)
    )
}

fn should_retry_error(method: &Method, err: &reqwest::Error) -> bool {
    if !is_idempotent(method) {
        return false;
    }
    err.is_timeout() || err.is_connect()
}

fn is_idempotent(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::PUT | Method::DELETE | Method::OPTIONS
    )
}

fn retry_delay(attempt: usize, headers: &HeaderMap, policy: &RetryPolicy) -> Duration {
    if let Some(delay) = retry_after(headers) {
        return delay;
    }
    let multiplier = 2_u32.saturating_pow(attempt as u32);
    let delay = policy.base_delay.saturating_mul(multiplier);
    if delay > policy.max_delay {
        policy.max_delay
    } else {
        delay
    }
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?;
    let raw = value.to_str().ok()?;
    let seconds = raw.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds))
}

fn next_link(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("link")?.to_str().ok()?;
    parse_next_link(value)
}

fn parse_next_link(link_header: &str) -> Option<String> {
    for segment in link_header.split(',') {
        let mut parts = segment.split(';');
        let link_part = parts.next()?.trim();
        let rel_part = parts.any(|part| part.trim() == r#"rel="next""#);
        if !rel_part {
            continue;
        }
        let trimmed = link_part.trim_start_matches('<').trim_end_matches('>');
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{parse_next_link, CanvasClient};
    use crate::{AuthConfig, CanvasConfig, DefaultsConfig};
    use canvas_models::{CanvasHost, CanvasToken};

    #[test]
    fn parse_next_link_finds_next() {
        let header = r#"<https://example.com/api/v1/courses?page=2>; rel="next", <https://example.com/api/v1/courses?page=1>; rel="current""#;
        let next = parse_next_link(header);
        assert_eq!(
            next.as_deref(),
            Some("https://example.com/api/v1/courses?page=2")
        );
    }

    #[test]
    fn parse_next_link_returns_none_without_next() {
        let header =
            r#"<https://example.com/api/v1/courses?page=1>; rel="current""#;
        assert!(parse_next_link(header).is_none());
    }

    #[test]
    fn build_url_joins_paths() {
        let host: CanvasHost = "https://example.com".parse().expect("host");
        let token: CanvasToken = "token123".parse().expect("token");
        let config = CanvasConfig {
            auth: AuthConfig { host, token },
            defaults: DefaultsConfig::default(),
        };
        let client = CanvasClient::new(&config).expect("client");
        let url = client.build_url("/courses");
        assert_eq!(url, "https://example.com/api/v1/courses");
    }
}
