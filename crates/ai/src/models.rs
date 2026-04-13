use reqwest::Client;
use serde::Deserialize;
use url::Url;

use crate::error::{AiError, Result};
use crate::settings::ProviderType;

/// Check whether a model belongs to one of the whitelisted families.
fn is_allowed_model(model_id: &str) -> bool {
    let id = model_id.to_lowercase();
    id.contains("sonnet")
        || id.contains("opus")
        || id.contains("glm")
        || (id.contains("gemini") && id.contains("pro"))
        || id.contains("kimi")
        || id.contains("deepseek")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

impl std::fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.display_name.is_empty() || self.display_name == self.id {
            f.write_str(&self.id)
        } else {
            write!(f, "{} ({})", self.display_name, self.id)
        }
    }
}

pub async fn fetch_models(provider_type: ProviderType, api_key: &str) -> Result<Vec<ModelInfo>> {
    let client = Client::new();
    let base =
        Url::parse(provider_type.base_url()).map_err(|e| AiError::InvalidUrl(e.to_string()))?;

    match provider_type {
        ProviderType::OpenAI | ProviderType::OpenRouter => {
            let url = base
                .join("models")
                .map_err(|e| AiError::InvalidUrl(e.to_string()))?;
            let resp = client
                .get(url)
                .bearer_auth(api_key)
                .send()
                .await?
                .error_for_status()?;
            let body: OpenAiModelsResponse = resp
                .json()
                .await
                .map_err(|e| AiError::InvalidResponse(e.to_string()))?;
            let mut models: Vec<ModelInfo> = body
                .data
                .into_iter()
                .map(|m| ModelInfo {
                    display_name: m.name.unwrap_or_else(|| m.id.clone()),
                    id: m.id,
                })
                .filter(|m| is_allowed_model(&m.id))
                .collect();
            models.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(models)
        }
        ProviderType::Anthropic => {
            let url = base
                .join("models?limit=100")
                .map_err(|e| AiError::InvalidUrl(e.to_string()))?;
            let resp = client
                .get(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await?
                .error_for_status()?;
            let body: AnthropicModelsResponse = resp
                .json()
                .await
                .map_err(|e| AiError::InvalidResponse(e.to_string()))?;
            let mut models: Vec<ModelInfo> = body
                .data
                .into_iter()
                .map(|m| ModelInfo {
                    display_name: m.display_name.unwrap_or_else(|| m.id.clone()),
                    id: m.id,
                })
                .filter(|m| is_allowed_model(&m.id))
                .collect();
            models.sort_by(|a, b| a.id.cmp(&b.id));
            Ok(models)
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
    #[serde(alias = "name")]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModel {
    id: String,
    display_name: Option<String>,
}
