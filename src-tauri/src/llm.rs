use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// LLMs sometimes wrap their response in XML tags (e.g. `<cleaned>...</cleaned>`)
/// mimicking the `<transcription>` tags in the input. Extract the inner content
/// of the first such tag pair, or return the original text if no tags found.
fn extract_from_tags(text: &str) -> String {
    if let Some(open_start) = text.find('<') {
        if let Some(rel_end) = text[open_start..].find('>') {
            let start_end = open_start + rel_end;
            let tag_content = &text[open_start + 1..start_end];
            if !tag_content.is_empty() && !tag_content.starts_with('/') {
                // Extract tag name only (ignore attributes)
                let tag_name = tag_content
                    .split(|c: char| c.is_whitespace() || c == '/')
                    .next()
                    .unwrap_or("");
                if !tag_name.is_empty() {
                    let close_tag = format!("</{}>", tag_name);
                    if let Some(close_pos) = text.find(&close_tag) {
                        let inner = &text[start_end + 1..close_pos];
                        return inner.trim().to_string();
                    }
                }
            }
        }
    }
    text.to_string()
}

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
            input: "so um basically I think we should uh we should go with the pasta tonight you know".to_string(),
            output: "I think we should go with the pasta tonight.".to_string(),
        },
        FewShotExample {
            input: "so I was talking to Sarah and she said that basically the weather is uh gonna be really nice this weekend like maybe seventy five degrees and I think we should you know go to the park or something".to_string(),
            output: "I was talking to Sarah and she said that the weather is gonna be really nice this weekend, maybe seventy-five degrees. I think we should go to the park or something.".to_string(),
        },
        FewShotExample {
            input: "okay so first I need to uh pick up the groceries second I need to like drop off the dry cleaning and third um I have to get gas on the way home".to_string(),
            output: "First, I need to pick up the groceries. Second, I need to drop off the dry cleaning. Third, I have to get gas on the way home.".to_string(),
        },
        FewShotExample {
            input: "The quick brown fox jumped over the lazy dog.".to_string(),
            output: "The quick brown fox jumped over the lazy dog.".to_string(),
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

    // Add few-shot examples from config, wrapped in tags
    for example in &config.few_shot_examples {
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!("<transcription>{}</transcription>", example.input),
        });
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: example.output.clone(),
        });
    }

    // Actual transcription to clean, same tag format
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: format!("<transcription>{}</transcription>", raw_text),
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

            Ok(extract_from_tags(parsed.message.content.trim()))
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
                .map(|c| extract_from_tags(c.message.content.trim()))
                .ok_or_else(|| AppError::Llm("No response from LLM".into()))
        }
    }
}

pub async fn test_connection(config: &LlmConfig) -> Result<String, AppError> {
    cleanup_text(config, "Hello, this is a test.").await
}
