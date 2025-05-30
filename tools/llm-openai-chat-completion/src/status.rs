//! This module provides functionality for checking the health of the OpenAI API.
//!
//! It defines the structures for parsing the response from the OpenAI status API
//! and provides a function to check if the API is currently operational.

use {
    chrono::{DateTime, FixedOffset},
    log::{debug, error},
    nexus_toolkit::{AnyResult, StatusCode, WithSerdeErrorPath},
    reqwest::Client,
    serde::Deserialize,
};

const DEFAULT_HEALTHCHECK_URL: &str = "https://status.openai.com/api/v2/status.json";
/// The expected status indicator for a healthy API.
const STATUS_INDICATOR_NONE: &str = "none";
/// The expected status indicator for an almost healthy API (minor hiccups).
const STATUS_INDICATOR_MINOR: &str = "minor";

/// Represents the overall response from the OpenAI status API.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ApiResponse {
    /// Information about the status page.
    page: PageInfo,
    /// The current status of the API.
    status: StatusInfo,
}

/// Represents information about the status page.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PageInfo {
    /// The status.io ID of the page.
    id: String,
    /// The name of the page.
    name: String,
    /// The URL of the page.
    url: String,
    /// The time zone of the page.
    time_zone: Option<String>,
    /// The last time the page was updated.
    updated_at: DateTime<FixedOffset>, // Parsed with original offset
}

/// Represents the status of the API.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct StatusInfo {
    /// The status indicator (e.g., "none", "minor", "major", "critical", ...).
    indicator: String,
    /// A description of the current status.
    description: String,
}

/// Checks the health of the OpenAI API by querying its status endpoint.
///
/// This function sends a GET request to the OpenAI status API and parses the
/// response. It checks if the `status.indicator` is equal to `HEALTH_OK`.
///
/// # Returns
///
/// *   `Ok(StatusCode::OK)` if the API is healthy.
/// *   `Ok(StatusCode::SERVICE_UNAVAILABLE)` if the API is not healthy.
/// *   `Err(e)` if there is an error sending the request or parsing the response.
pub(crate) async fn check_api_health() -> AnyResult<StatusCode> {
    let client = Client::new();
    let raw_response = client.get(healthcheck_url()).send().await?.text().await?;
    debug!("Raw API response: {}", raw_response);

    let wrapped: WithSerdeErrorPath<ApiResponse> = match serde_json::from_str(&raw_response) {
        Ok(val) => val,
        Err(e) => {
            error!("Deserialization error: {}", e);
            return Ok(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response: ApiResponse = wrapped.0;
    if !matches!(
        response.status.indicator.as_str(),
        STATUS_INDICATOR_NONE | STATUS_INDICATOR_MINOR
    ) {
        return Ok(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(StatusCode::OK)
}

fn healthcheck_url() -> String {
    std::env::var("HEALTHCHECK_URL").unwrap_or_else(|_| DEFAULT_HEALTHCHECK_URL.to_string())
}
