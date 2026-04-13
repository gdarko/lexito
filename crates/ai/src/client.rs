use std::pin::Pin;
use std::time::Duration;

use async_stream::stream;
use futures::{Stream, StreamExt};
use lexito_core::EntryKey;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{AiError, Result};
use crate::prompt::{batch_user_prompt, system_prompt, user_prompt};
use crate::settings::{ProviderProfile, ProviderType, TranslationPreferences};

const BATCH_CHUNK_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct ResolvedProviderProfile {
    pub id: uuid::Uuid,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: url::Url,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub key: EntryKey,
    pub target_locale: String,
    pub msgid: String,
    pub msgid_plural: Option<String>,
    pub msgctxt: Option<String>,
    pub comments: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TranslationMetadata {
    pub provider_name: String,
    pub model: String,
    pub raw_response: String,
}

#[derive(Debug, Clone)]
pub struct TranslationResponse {
    pub singular: String,
    pub plurals: Vec<String>,
    pub metadata: TranslationMetadata,
}

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub request: TranslationRequest,
}

#[derive(Debug, Clone)]
pub struct BatchItemResult {
    pub key: EntryKey,
    pub result: std::result::Result<TranslationResponse, String>,
}

#[derive(Debug, Clone)]
pub enum BatchProgressEvent {
    Started {
        total: usize,
    },
    Item {
        completed: usize,
        total: usize,
        item: BatchItemResult,
    },
    Finished {
        completed: usize,
        total: usize,
    },
}

#[derive(Debug, Clone)]
pub struct AiClient {
    http: Client,
    provider: ResolvedProviderProfile,
    preferences: TranslationPreferences,
}

impl AiClient {
    pub fn new(
        provider: ResolvedProviderProfile,
        preferences: TranslationPreferences,
    ) -> Result<Self> {
        let timeout = Duration::from_secs(preferences.timeout_secs.unwrap_or(60));
        let http = Client::builder().timeout(timeout).build()?;

        Ok(Self {
            http,
            provider,
            preferences,
        })
    }

    // ── Single entry translation (for "AI Translate" button) ─────

    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        match self.provider.provider_type {
            ProviderType::Anthropic => self.translate_anthropic(request).await,
            ProviderType::OpenAI | ProviderType::OpenRouter => self.translate_openai(request).await,
        }
    }

    async fn translate_openai(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let endpoint = self
            .provider
            .base_url
            .join("chat/completions")
            .map_err(|error| AiError::InvalidUrl(error.to_string()))?;

        let payload = ChatCompletionRequest {
            model: self.provider.model.clone(),
            temperature: self.preferences.temperature.unwrap_or(0.2),
            response_format: ResponseFormat {
                kind: "json_object".to_string(),
            },
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt(self.preferences.system_prompt.as_deref()),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt(&request),
                },
            ],
        };

        let response = self
            .http
            .post(endpoint)
            .bearer_auth(&self.provider.api_key)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        let parsed: ChatCompletionResponse = serde_json::from_str(&body)
            .map_err(|error| AiError::InvalidResponse(error.to_string()))?;

        let content = parsed
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("missing choices".to_string()))?;

        let envelope: TranslationEnvelope = serde_json::from_str(&content)
            .map_err(|error| AiError::InvalidResponse(error.to_string()))?;

        Ok(TranslationResponse {
            singular: envelope.singular.unwrap_or_default(),
            plurals: envelope.plurals.unwrap_or_default(),
            metadata: TranslationMetadata {
                provider_name: self.provider.name.clone(),
                model: parsed.model.unwrap_or_else(|| self.provider.model.clone()),
                raw_response: body,
            },
        })
    }

    async fn translate_anthropic(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationResponse> {
        let endpoint = self
            .provider
            .base_url
            .join("messages")
            .map_err(|error| AiError::InvalidUrl(error.to_string()))?;

        let payload = AnthropicMessagesRequest {
            model: self.provider.model.clone(),
            max_tokens: 4096,
            temperature: self.preferences.temperature.unwrap_or(0.2),
            system: system_prompt(self.preferences.system_prompt.as_deref()),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_prompt(&request),
            }],
        };

        let response = self
            .http
            .post(endpoint)
            .header("x-api-key", &self.provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        let content = extract_anthropic_text(&body)?;

        let envelope: TranslationEnvelope = serde_json::from_str(&content)
            .map_err(|error| AiError::InvalidResponse(error.to_string()))?;

        Ok(TranslationResponse {
            singular: envelope.singular.unwrap_or_default(),
            plurals: envelope.plurals.unwrap_or_default(),
            metadata: TranslationMetadata {
                provider_name: self.provider.name.clone(),
                model: extract_model(&body, &self.provider.model),
                raw_response: body,
            },
        })
    }

    // ── Batch translation (multiple entries per call) ────────────

    async fn translate_batch(
        &self,
        requests: &[TranslationRequest],
    ) -> Vec<(EntryKey, std::result::Result<TranslationResponse, String>)> {
        if requests.is_empty() {
            return Vec::new();
        }

        let keys: Vec<EntryKey> = requests.iter().map(|r| r.key.clone()).collect();

        let result = match self.provider.provider_type {
            ProviderType::Anthropic => self.batch_call_anthropic(requests).await,
            ProviderType::OpenAI | ProviderType::OpenRouter => {
                self.batch_call_openai(requests).await
            }
        };

        match result {
            Ok((entries, raw_response, model)) => {
                let metadata = TranslationMetadata {
                    provider_name: self.provider.name.clone(),
                    model,
                    raw_response,
                };
                map_batch_results(&keys, entries, metadata)
            }
            Err(error) => {
                // All entries in this chunk fail with the same error
                keys.into_iter()
                    .map(|key| (key, Err(error.to_string())))
                    .collect()
            }
        }
    }

    async fn batch_call_openai(
        &self,
        requests: &[TranslationRequest],
    ) -> Result<(Vec<BatchEntryEnvelope>, String, String)> {
        let endpoint = self
            .provider
            .base_url
            .join("chat/completions")
            .map_err(|error| AiError::InvalidUrl(error.to_string()))?;

        let payload = ChatCompletionRequest {
            model: self.provider.model.clone(),
            temperature: self.preferences.temperature.unwrap_or(0.2),
            response_format: ResponseFormat {
                kind: "json_object".to_string(),
            },
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt(self.preferences.system_prompt.as_deref()),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: batch_user_prompt(requests),
                },
            ],
        };

        let response = self
            .http
            .post(endpoint)
            .bearer_auth(&self.provider.api_key)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        let parsed: ChatCompletionResponse = serde_json::from_str(&body)
            .map_err(|error| AiError::InvalidResponse(error.to_string()))?;

        let content = parsed
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| AiError::InvalidResponse("missing choices".to_string()))?;

        let entries = parse_batch_response(&content)?;
        let model = parsed.model.unwrap_or_else(|| self.provider.model.clone());

        Ok((entries, body, model))
    }

    async fn batch_call_anthropic(
        &self,
        requests: &[TranslationRequest],
    ) -> Result<(Vec<BatchEntryEnvelope>, String, String)> {
        let endpoint = self
            .provider
            .base_url
            .join("messages")
            .map_err(|error| AiError::InvalidUrl(error.to_string()))?;

        let payload = AnthropicMessagesRequest {
            model: self.provider.model.clone(),
            max_tokens: 16384,
            temperature: self.preferences.temperature.unwrap_or(0.2),
            system: system_prompt(self.preferences.system_prompt.as_deref()),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: batch_user_prompt(requests),
            }],
        };

        let response = self
            .http
            .post(endpoint)
            .header("x-api-key", &self.provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        let content = extract_anthropic_text(&body)?;
        let entries = parse_batch_response(&content)?;
        let model = extract_model(&body, &self.provider.model);

        Ok((entries, body, model))
    }

    // ── Batch stream (chunks of BATCH_CHUNK_SIZE) ───────────────

    pub fn batch_stream(
        &self,
        items: Vec<BatchItem>,
    ) -> Pin<Box<dyn Stream<Item = BatchProgressEvent> + Send>> {
        let client = self.clone();
        let concurrency = self.preferences.batch_concurrency.unwrap_or(4).max(1);

        Box::pin(stream! {
            let total = items.len();
            yield BatchProgressEvent::Started { total };

            // Chunk items into groups for batch API calls
            let chunks: Vec<Vec<TranslationRequest>> = items
                .into_iter()
                .map(|item| item.request)
                .collect::<Vec<_>>()
                .chunks(BATCH_CHUNK_SIZE)
                .map(|c| c.to_vec())
                .collect();

            let mut completed = 0usize;
            let chunk_stream = futures::stream::iter(chunks.into_iter().map(|chunk| {
                let client = client.clone();
                async move {
                    client.translate_batch(&chunk).await
                }
            }))
            .buffer_unordered(concurrency);

            futures::pin_mut!(chunk_stream);

            while let Some(results) = chunk_stream.next().await {
                for (key, result) in results {
                    completed += 1;
                    let item = BatchItemResult {
                        key,
                        result: result.map_err(|e| e.to_string()),
                    };
                    yield BatchProgressEvent::Item {
                        completed,
                        total,
                        item,
                    };
                }
            }

            yield BatchProgressEvent::Finished { completed, total };
        })
    }
}

// ── Response parsing helpers ────────────────────────────────────

fn extract_anthropic_text(body: &str) -> Result<String> {
    let parsed: AnthropicMessagesResponse =
        serde_json::from_str(body).map_err(|error| AiError::InvalidResponse(error.to_string()))?;
    parsed
        .content
        .iter()
        .find(|block| block.kind == "text")
        .and_then(|block| block.text.clone())
        .ok_or_else(|| AiError::InvalidResponse("no text content block".to_string()))
}

fn extract_model(body: &str, fallback: &str) -> String {
    // Try to extract "model" field from raw JSON without full parse
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("model")?.as_str().map(String::from))
        .unwrap_or_else(|| fallback.to_string())
}

fn parse_batch_response(content: &str) -> Result<Vec<BatchEntryEnvelope>> {
    // Strip markdown fences if present
    let trimmed = content.trim();
    let json = if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    };

    // Try parsing as a direct JSON array: [{...}, {...}, ...]
    if let Ok(entries) = serde_json::from_str::<Vec<BatchEntryEnvelope>>(json) {
        return Ok(entries);
    }

    // Try parsing as a JSON object that wraps an array in some field
    // Models often return {"translations": [...]} or {"entries": [...]} or {"results": [...]}
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json) {
        // If it's an object, search all fields for an array of translation objects
        if let Some(obj) = value.as_object() {
            for (_key, val) in obj {
                if val.is_array() {
                    if let Ok(entries) =
                        serde_json::from_value::<Vec<BatchEntryEnvelope>>(val.clone())
                    {
                        return Ok(entries);
                    }
                }
            }
        }

        // Maybe the object itself is a single translation (shouldn't happen in batch, but handle it)
        if let Ok(entry) = serde_json::from_value::<BatchEntryEnvelope>(value) {
            return Ok(vec![entry]);
        }
    }

    // Try extracting a JSON array from within the text (model might have added text around it)
    if let Some(start) = json.find('[') {
        if let Some(end) = json.rfind(']') {
            let slice = &json[start..=end];
            if let Ok(entries) = serde_json::from_str::<Vec<BatchEntryEnvelope>>(slice) {
                return Ok(entries);
            }
        }
    }

    Err(AiError::InvalidResponse(format!(
        "Could not parse batch response. Raw content (first 500 chars): {}",
        &json[..json.len().min(500)]
    )))
}

fn map_batch_results(
    keys: &[EntryKey],
    entries: Vec<BatchEntryEnvelope>,
    metadata: TranslationMetadata,
) -> Vec<(EntryKey, std::result::Result<TranslationResponse, String>)> {
    let mut results: Vec<(EntryKey, std::result::Result<TranslationResponse, String>)> =
        Vec::with_capacity(keys.len());

    for (i, key) in keys.iter().enumerate() {
        let id_str = (i + 1).to_string();
        // Find by id first, fall back to positional index
        let entry = entries
            .iter()
            .find(|e| e.id.as_deref() == Some(id_str.as_str()))
            .or_else(|| entries.get(i));

        let result = match entry {
            Some(e) => Ok(TranslationResponse {
                singular: e.singular.clone().unwrap_or_default(),
                plurals: e.plurals.clone().unwrap_or_default(),
                metadata: metadata.clone(),
            }),
            None => Err(format!("model skipped entry {}", i + 1)),
        };

        results.push((key.clone(), result));
    }

    results
}

// ── OpenAI / OpenRouter request/response ────────────────────────

#[derive(Debug, Clone, Serialize)]
struct ChatCompletionRequest {
    model: String,
    temperature: f32,
    response_format: ResponseFormat,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

// ── Anthropic request/response ──────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct AnthropicMessagesRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Clone, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicMessagesResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

// ── Shared response types ───────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct TranslationEnvelope {
    singular: Option<String>,
    plurals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct BatchEntryEnvelope {
    id: Option<String>,
    singular: Option<String>,
    plurals: Option<Vec<String>>,
}

impl From<(ProviderProfile, String)> for ResolvedProviderProfile {
    fn from((provider, api_key): (ProviderProfile, String)) -> Self {
        Self {
            id: provider.id,
            name: provider.name,
            provider_type: provider.provider_type,
            base_url: provider.base_url,
            model: provider.model,
            api_key,
        }
    }
}
