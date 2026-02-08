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
                    .unwrap();
                if !tag_name.is_empty() {
                    let close_tag = format!("</{}>", tag_name);
                    if let Some(rel_close) = text[start_end + 1..].find(&close_tag) {
                        let inner = &text[start_end + 1..start_end + 1 + rel_close];
                        return inner.trim().to_string();
                    }
                }
            }
        }
    }
    text.to_string()
}

pub const DEFAULT_SYSTEM_PROMPT: &str = "You are a speech-to-text post-processor. Clean up the transcription while preserving the speaker's meaning and voice. Your tasks:\n- Remove filler words (um, uh, like, you know, basically, I mean, so, right, okay)\n- Handle self-corrections by keeping only the final intended version\n- Fix grammar, punctuation, and sentence structure\n- Format with structure: use bullet points or numbered lists when items are enumerated, add paragraph breaks between distinct topics\n- Improve clarity and conciseness without changing the meaning or making it sound robotic\nOutput only the cleaned result.";

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
            input: "so um I was thinking we should meet on Tuesday no wait Wednesday at like 2 PM to go over the uh the project proposal you know".to_string(),
            output: "I was thinking we should meet on Wednesday at 2 PM to go over the project proposal.".to_string(),
        },
        FewShotExample {
            input: "okay so for the grocery store I need um bananas oranges apples and oh also we need like milk and bread and uh some eggs I think".to_string(),
            output: "Grocery list:\n- Bananas\n- Oranges\n- Apples\n- Milk\n- Bread\n- Eggs".to_string(),
        },
        FewShotExample {
            input: "so basically to set up the project you first need to um clone the repo and then you install the dependencies with npm install and then uh you need to create a dot env file with your API key and finally just run npm run dev to start it".to_string(),
            output: "To set up the project:\n1. Clone the repo\n2. Install dependencies with `npm install`\n3. Create a `.env` file with your API key\n4. Run `npm run dev` to start it".to_string(),
        },
        FewShotExample {
            input: "so the meeting went really well um we decided to launch the new feature next month and uh Sarah is going to handle the design and Mike will do the backend and oh we also need to hire like two more engineers for the mobile team because they're basically swamped right now".to_string(),
            output: "The meeting went really well. We decided to launch the new feature next month. Sarah is going to handle the design and Mike will do the backend.\n\nWe also need to hire two more engineers for the mobile team because they're swamped right now.".to_string(),
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
