use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub const DEFAULT_SYSTEM_PROMPT: &str = "Remove filler words (um, uh, like, you know, basically, I mean, so, right, okay) and fix punctuation. Do not rephrase, summarize, or rewrite. Keep the speakers original words and sentence structure. Output only the result.";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiType {
    Ollama,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotExample {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub endpoint: String,
    pub model: String,
    pub system_prompt: String,
    pub api_type: ApiType,
    #[serde(default = "default_few_shot_examples")]
    pub few_shot_examples: Vec<FewShotExample>,
}

pub fn default_few_shot_examples() -> Vec<FewShotExample> {
    vec![
        FewShotExample {
            input: "so um basically I think we should uh we should go with the first option you know".to_string(),
            output: "I think we should go with the first option.".to_string(),
        },
        FewShotExample {
            input: "so I was talking to Mike and he said that basically the deadline is uh gonna be pushed back by like two weeks because you know the design team needs more time and I think thats fine but we should let the client know".to_string(),
            output: "I was talking to Mike and he said that the deadline is gonna be pushed back by two weeks because the design team needs more time. I think that's fine, but we should let the client know.".to_string(),
        },
        FewShotExample {
            input: "okay so first we need to uh set up the database second we need to like write the API endpoints and third um we need to build the frontend".to_string(),
            output: "First, we need to set up the database. Second, we need to write the API endpoints. Third, we need to build the frontend.".to_string(),
        },
        FewShotExample {
            input: "Hello world.".to_string(),
            output: "Hello world.".to_string(),
        },
    ]
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "mistral".to_string(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            api_type: ApiType::Ollama,
            few_shot_examples: default_few_shot_examples(),
        }
    }
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: ChatMessage,
}

pub async fn cleanup_text(config: &LlmConfig, raw_text: &str) -> Result<String, AppError> {
    let client = reqwest::Client::new();

    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: config.system_prompt.clone(),
        },
    ];

    // Add few-shot examples from config
    for example in &config.few_shot_examples {
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: example.input.clone(),
        });
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: example.output.clone(),
        });
    }

    // Actual transcription to clean
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: raw_text.to_string(),
    });

    match config.api_type {
        ApiType::Ollama => {
            let url = format!("{}/api/chat", config.endpoint.trim_end_matches('/'));
            let body = OllamaChatRequest {
                model: config.model.clone(),
                messages,
                stream: false,
            };

            let resp = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Llm(format!("Request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(AppError::Llm(format!("Ollama error {}: {}", status, text)));
            }

            let parsed: OllamaChatResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Llm(format!("Parse error: {}", e)))?;

            Ok(parsed.message.content.trim().to_string())
        }
        ApiType::OpenAI => {
            let url = format!(
                "{}/v1/chat/completions",
                config.endpoint.trim_end_matches('/')
            );
            let body = OpenAIChatRequest {
                model: config.model.clone(),
                messages,
            };

            let resp = client
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Llm(format!("Request failed: {}", e)))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(AppError::Llm(format!("OpenAI error {}: {}", status, text)));
            }

            let parsed: OpenAIChatResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Llm(format!("Parse error: {}", e)))?;

            parsed
                .choices
                .first()
                .map(|c| c.message.content.trim().to_string())
                .ok_or_else(|| AppError::Llm("No response from LLM".into()))
        }
    }
}

pub async fn test_connection(config: &LlmConfig) -> Result<String, AppError> {
    cleanup_text(config, "Hello, this is a test.").await
}
