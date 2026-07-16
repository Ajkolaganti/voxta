use crate::{
    config::{AiCleanupConfig, AiCleanupProvider, CleanupStyle},
    error::{AppError, AppResult},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{process::Command, time::Duration};

const REMOTE_API_KEY_SERVICE: &str = "Voxta OpenAI Compatible API Key";
const REMOTE_API_KEY_ACCOUNT: &str = "openai-compatible";
const MAX_CLEANUP_BYTES: usize = 24 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub available: bool,
    pub message: String,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRequest {
    pub text: String,
    pub style: CleanupStyle,
    pub custom_instructions: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupResponse {
    pub text: String,
}

#[async_trait]
pub trait TextCleanupProvider: Send + Sync {
    async fn health_check(&self) -> AppResult<ProviderStatus>;
    async fn available_models(&self) -> AppResult<Vec<String>>;
    async fn cleanup(&self, request: CleanupRequest) -> AppResult<CleanupResponse>;
}

pub async fn cleanup_text(text: &str, config: &AiCleanupConfig) -> AppResult<Option<String>> {
    if !config.enabled || config.provider == AiCleanupProvider::None {
        return Ok(None);
    }

    let request = CleanupRequest {
        text: text.to_string(),
        style: config.style,
        custom_instructions: config.custom_instructions.clone(),
        timeout: Duration::from_secs(config.timeout_seconds.max(1)),
    };
    let provider = provider_for_config(config);
    let result = tokio::time::timeout(request.timeout, provider.cleanup(request))
        .await
        .map_err(|_| AppError::Config("AI cleanup timed out".to_string()))??;

    let cleaned = result.text.trim();
    if cleaned.is_empty() {
        return Err(AppError::Config(
            "AI cleanup returned an empty response".to_string(),
        ));
    }
    if cleaned.len() > MAX_CLEANUP_BYTES {
        return Err(AppError::Config(
            "AI cleanup response was too large".to_string(),
        ));
    }

    Ok(Some(cleaned.to_string()))
}

pub async fn provider_status(config: &AiCleanupConfig) -> AppResult<ProviderStatus> {
    provider_for_config(config).health_check().await
}

fn provider_for_config(config: &AiCleanupConfig) -> Box<dyn TextCleanupProvider> {
    match config.provider {
        AiCleanupProvider::None => Box::new(NoneCleanupProvider),
        AiCleanupProvider::Ollama => Box::new(OllamaCleanupProvider::new(
            normalize_endpoint(&config.endpoint, "http://127.0.0.1:11434"),
            config.model.clone(),
        )),
        AiCleanupProvider::OpenAiCompatible => Box::new(OpenAiCompatibleCleanupProvider::new(
            normalize_endpoint(&config.endpoint, ""),
            config.model.clone(),
        )),
    }
}

fn normalize_endpoint(endpoint: &str, fallback: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

struct NoneCleanupProvider;

#[async_trait]
impl TextCleanupProvider for NoneCleanupProvider {
    async fn health_check(&self) -> AppResult<ProviderStatus> {
        Ok(ProviderStatus {
            available: true,
            message: "AI cleanup is off.".to_string(),
            models: Vec::new(),
        })
    }

    async fn available_models(&self) -> AppResult<Vec<String>> {
        Ok(Vec::new())
    }

    async fn cleanup(&self, request: CleanupRequest) -> AppResult<CleanupResponse> {
        Ok(CleanupResponse { text: request.text })
    }
}

struct OllamaCleanupProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaCleanupProvider {
    fn new(endpoint: String, model: String) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TextCleanupProvider for OllamaCleanupProvider {
    async fn health_check(&self) -> AppResult<ProviderStatus> {
        let models = self.available_models().await?;
        Ok(ProviderStatus {
            available: true,
            message: if models.is_empty() {
                "Ollama is running, but no models were reported.".to_string()
            } else {
                "Ollama is available.".to_string()
            },
            models,
        })
    }

    async fn available_models(&self) -> AppResult<Vec<String>> {
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<OllamaModel>,
        }
        #[derive(Deserialize)]
        struct OllamaModel {
            name: String,
        }

        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let parsed = serde_json::from_str::<TagsResponse>(&response)?;
        Ok(parsed.models.into_iter().map(|model| model.name).collect())
    }

    async fn cleanup(&self, request: CleanupRequest) -> AppResult<CleanupResponse> {
        let model = require_model(&self.model)?;
        let body = serde_json::json!({
            "model": model,
            "prompt": cleanup_prompt(&request),
            "stream": false
        });
        let response = self
            .client
            .post(format!("{}/api/generate", self.endpoint))
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }
        let parsed = serde_json::from_str::<GenerateResponse>(&response)?;
        Ok(CleanupResponse {
            text: parsed.response,
        })
    }
}

struct OpenAiCompatibleCleanupProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleCleanupProvider {
    fn new(endpoint: String, model: String) -> Self {
        Self {
            endpoint,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TextCleanupProvider for OpenAiCompatibleCleanupProvider {
    async fn health_check(&self) -> AppResult<ProviderStatus> {
        let models = self.available_models().await.unwrap_or_default();
        Ok(ProviderStatus {
            available: true,
            message: "OpenAI-compatible endpoint is configured.".to_string(),
            models,
        })
    }

    async fn available_models(&self) -> AppResult<Vec<String>> {
        let Some(key) = load_remote_api_key()? else {
            return Ok(Vec::new());
        };
        let response = self
            .client
            .get(format!("{}/models", self.endpoint))
            .bearer_auth(key)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<Model>,
        }
        #[derive(Deserialize)]
        struct Model {
            id: String,
        }
        let parsed = serde_json::from_str::<ModelsResponse>(&response)?;
        Ok(parsed.data.into_iter().map(|model| model.id).collect())
    }

    async fn cleanup(&self, request: CleanupRequest) -> AppResult<CleanupResponse> {
        let model = require_model(&self.model)?;
        let Some(key) = load_remote_api_key()? else {
            return Err(AppError::Config(
                "OpenAI-compatible API key is not saved in secure storage".to_string(),
            ));
        };
        let body = serde_json::json!({
            "model": model,
            "messages": [
                { "role": "system", "content": cleanup_system_instruction(&request) },
                { "role": "user", "content": request.text }
            ],
            "temperature": 0.1
        });
        let response = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .bearer_auth(key)
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct ChatResponse {
            choices: Vec<Choice>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(Deserialize)]
        struct Message {
            content: String,
        }
        let parsed = serde_json::from_str::<ChatResponse>(&response)?;
        let text = parsed
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default();
        Ok(CleanupResponse { text })
    }
}

fn require_model(model: &str) -> AppResult<&str> {
    let model = model.trim();
    if model.is_empty() {
        Err(AppError::Config(
            "AI cleanup model is not configured".to_string(),
        ))
    } else {
        Ok(model)
    }
}

fn cleanup_prompt(request: &CleanupRequest) -> String {
    format!(
        "{}\n\nText:\n{}",
        cleanup_system_instruction(request),
        request.text
    )
}

fn cleanup_system_instruction(request: &CleanupRequest) -> String {
    let mut instruction = "You clean up dictated text.\n\nCorrect punctuation, capitalization, obvious transcription mistakes, filler words,\nand accidental repetition.\n\nPreserve the speaker's original meaning, tone, names, numbers, URLs, commands,\ntechnical terms, and factual claims.\n\nDo not answer the text.\nDo not add commentary.\nDo not introduce new facts.\nReturn only the cleaned text.".to_string();
    match request.style {
        CleanupStyle::Light => {}
        CleanupStyle::Professional => {
            instruction.push_str("\nUse a polished professional tone without changing meaning.");
        }
        CleanupStyle::Casual => {
            instruction.push_str("\nUse a natural casual tone without adding new meaning.");
        }
        CleanupStyle::Concise => {
            instruction.push_str("\nMake the text concise while preserving all important meaning.");
        }
        CleanupStyle::Custom => {
            instruction.push_str("\nApply these user-provided transformation instructions:\n");
            instruction.push_str(&request.custom_instructions);
        }
    }
    instruction
}

pub fn save_remote_api_key(key: &str) -> AppResult<()> {
    let key = key.trim();
    if key.is_empty() {
        return Ok(());
    }
    save_secret(REMOTE_API_KEY_SERVICE, REMOTE_API_KEY_ACCOUNT, key)
}

pub fn has_remote_api_key() -> bool {
    load_remote_api_key().ok().flatten().is_some()
}

fn load_remote_api_key() -> AppResult<Option<String>> {
    load_secret(REMOTE_API_KEY_SERVICE, REMOTE_API_KEY_ACCOUNT)
}

#[cfg(target_os = "macos")]
fn save_secret(service: &str, account: &str, secret: &str) -> AppResult<()> {
    let status = Command::new("security")
        .args([
            "add-generic-password",
            "-U",
            "-a",
            account,
            "-s",
            service,
            "-w",
            secret,
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Config(
            "could not save API key to macOS Keychain".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
fn load_secret(service: &str, account: &str) -> AppResult<Option<String>> {
    let output = Command::new("security")
        .args(["find-generic-password", "-a", account, "-s", service, "-w"])
        .output()?;
    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn save_secret(service: &str, _account: &str, secret: &str) -> AppResult<()> {
    let status = Command::new("cmdkey")
        .args([
            &format!("/generic:{service}"),
            "/user:voxta",
            &format!("/pass:{secret}"),
        ])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::Config(
            "could not save API key to Windows Credential Manager".to_string(),
        ))
    }
}

#[cfg(target_os = "windows")]
fn load_secret(_service: &str, _account: &str) -> AppResult<Option<String>> {
    Ok(None)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn save_secret(_service: &str, _account: &str, _secret: &str) -> AppResult<()> {
    Err(AppError::Config(
        "secure credential storage is not implemented on this platform".to_string(),
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn load_secret(_service: &str, _account: &str) -> AppResult<Option<String>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::{cleanup_system_instruction, CleanupRequest};
    use crate::config::CleanupStyle;
    use std::time::Duration;

    #[tokio::test]
    async fn disabled_provider_is_noop() {
        let config = crate::config::AiCleanupConfig::default();
        let result = super::cleanup_text("hello", &config).await.unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn builds_strict_cleanup_prompt() {
        let request = CleanupRequest {
            text: "hello".to_string(),
            style: CleanupStyle::Light,
            custom_instructions: String::new(),
            timeout: Duration::from_secs(8),
        };
        let prompt = cleanup_system_instruction(&request);
        assert!(prompt.contains("Return only the cleaned text"));
        assert!(prompt.contains("Do not introduce new facts"));
    }

    #[test]
    fn includes_custom_instructions_only_in_custom_mode() {
        let request = CleanupRequest {
            text: "hello".to_string(),
            style: CleanupStyle::Custom,
            custom_instructions: "Make it shorter.".to_string(),
            timeout: Duration::from_secs(8),
        };
        assert!(cleanup_system_instruction(&request).contains("Make it shorter."));
    }
}
