use log::warn;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::de::DeserializeOwned;

const BASE_URL: &str = "https://api.rm.smartsheet.com/api/v1";
const MAX_RETRIES: u32 = 5;

pub struct RmClient {
    http: Client,
    token: String,
}

impl RmClient {
    pub fn new(token: &str) -> Result<Self, String> {
        let http = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|error| error.to_string())?;

        Ok(Self {
            http,
            token: token.trim().to_string(),
        })
    }

    pub fn get_json<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T, String> {
        self.request_json("GET", path, query, None)
    }

    pub fn request_json<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: Option<serde_json::Value>,
    ) -> Result<T, String> {
        let url = format!(
            "{}/{}",
            BASE_URL.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        for attempt in 0..=MAX_RETRIES {
            let mut request = match method {
                "GET" => self.http.get(&url),
                "POST" => self.http.post(&url),
                "PUT" => self.http.put(&url),
                "DELETE" => self.http.delete(&url),
                other => return Err(format!("Unsupported HTTP method: {other}")),
            };

            request = request.headers(self.auth_headers()?);

            if !query.is_empty() {
                request = request.query(query);
            }

            if let Some(ref payload) = body {
                request = request.json(payload);
            }

            let response = request.send().map_err(|error| error.to_string())?;

            if response.status().as_u16() == 429 {
                if attempt == MAX_RETRIES {
                    return Err("Resource Management rate limit exceeded. Try again shortly.".to_string());
                }

                let wait = rate_limit_wait(&response);
                thread::sleep(wait);
                continue;
            }

            let status = response.status();
            let body_text = response.text().map_err(|error| error.to_string())?;

            if !status.is_success() {
                warn!("RM API {method} {path} failed ({status}): {body_text}");
                return Err(format_http_error(status.as_u16(), &body_text));
            }

            if body_text.trim().is_empty() {
                return Err("Resource Management returned an empty response.".to_string());
            }

            return serde_json::from_str(&body_text).map_err(|error| {
                format!("Could not parse Resource Management response: {error}")
            });
        }

        Err("Resource Management request failed after retries.".to_string())
    }

    fn auth_headers(&self) -> Result<HeaderMap, String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "auth",
            HeaderValue::from_str(&self.token)
                .map_err(|error| format!("Invalid API token: {error}"))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(headers)
    }
}

fn rate_limit_wait(response: &reqwest::blocking::Response) -> Duration {
    if let Some(reset) = response
        .headers()
        .get("X-RateLimit-Reset")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if reset > now {
            return Duration::from_secs(reset - now + 1);
        }
    }

    Duration::from_secs(2)
}

fn format_http_error(status: u16, body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(message) = value.get("message").and_then(|item| item.as_str()) {
            return format!("Resource Management error ({status}): {message}");
        }
    }

    let snippet: String = body.chars().take(200).collect();
    format!("Resource Management error ({status}): {snippet}")
}
