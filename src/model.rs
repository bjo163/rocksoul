use std::{env, net::IpAddr};

use anyhow::{Context, Result, bail};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "http://localhost:11434/v1";

#[derive(Clone)]
pub struct ModelConfig {
    pub base_url: String,
    pub model: String,
    api_key: Option<String>,
}

impl ModelConfig {
    pub fn from_env() -> Option<Self> {
        let model = env::var("ROCKSOUL_MODEL")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())?;

        let base_url = env::var("ROCKSOUL_BASE_URL")
            .ok()
            .map(|value| value.trim().trim_end_matches('/').to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());

        let api_key = env::var("ROCKSOUL_API_KEY")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());

        Some(Self {
            base_url,
            model,
            api_key,
        })
    }

    pub async fn complete(
        &self,
        client: &Client,
        messages: &[ChatMessage],
    ) -> Result<ModelCompletion> {
        let base_url = validate_base_url(&self.base_url)?;
        let endpoint = format!(
            "{}/chat/completions",
            base_url.as_str().trim_end_matches('/')
        );

        let payload = ChatCompletionRequest {
            model: &self.model,
            messages,
            stream: false,
        };

        let mut request = client.post(endpoint).json(&payload);
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .send()
            .await
            .context("could not reach the configured model endpoint")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let summary = body.chars().take(1_200).collect::<String>();
            bail!("model endpoint returned {status}: {summary}");
        }

        let response: ChatCompletionResponse = response
            .json()
            .await
            .context("model endpoint returned an invalid chat-completions response")?;

        let choice = response
            .choices
            .into_iter()
            .next()
            .context("model endpoint returned no assistant message")?;
        let content = choice.message.content;
        if content.trim().is_empty() {
            bail!("model endpoint returned an empty assistant message");
        }

        Ok(ModelCompletion {
            content,
            output_tokens: response.usage.map(|usage| usage.completion_tokens),
        })
    }
}

/// A completion plus provider-reported usage when the endpoint supplies it.
pub struct ModelCompletion {
    pub content: String,
    pub output_tokens: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    role: String,
    content: String,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<ChatCompletionUsage>,
}

#[derive(Deserialize)]
struct ChatCompletionUsage {
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

fn validate_base_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).context("ROCKSOUL_BASE_URL must be a valid absolute URL")?;
    let host = url
        .host_str()
        .context("ROCKSOUL_BASE_URL must include a hostname")?;

    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        bail!("ROCKSOUL_BASE_URL must not contain credentials, a query string, or a fragment");
    }

    let loopback = is_loopback_host(host);
    let secure_remote = url.scheme() == "https";
    let local_http = url.scheme() == "http" && loopback;

    if !secure_remote && !local_http {
        bail!("model endpoints must use HTTPS; plain HTTP is allowed only for loopback hosts");
    }

    Ok(url)
}

fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return true;
    }

    let normalized = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);

    normalized
        .parse::<IpAddr>()
        .map(|address| address.is_loopback())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::validate_base_url;

    #[test]
    fn accepts_https_remote_endpoints() {
        assert!(validate_base_url("https://api.example.com/v1").is_ok());
    }

    #[test]
    fn accepts_loopback_http_endpoints() {
        assert!(validate_base_url("http://localhost:11434/v1").is_ok());
        assert!(validate_base_url("http://127.0.0.1:11434/v1").is_ok());
        assert!(validate_base_url("http://[::1]:11434/v1").is_ok());
    }

    #[test]
    fn rejects_remote_http_endpoints() {
        assert!(validate_base_url("http://api.example.com/v1").is_err());
    }

    #[test]
    fn rejects_credentials_in_endpoint_url() {
        assert!(validate_base_url("https://user:pass@example.com/v1").is_err());
    }
}
