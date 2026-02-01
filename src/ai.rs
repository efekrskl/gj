use anyhow::{Context, Result};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

pub struct AiClient {
    pub client: Ollama,
    pub model: String,
}

impl AiClient {
    pub fn new(model: Option<String>) -> Self {
        Self {
            client: Ollama::default(),
            model: model.unwrap_or("llama3.2".to_string()),
        }
    }

    pub fn summarize(&self, text: &str) -> Result<String> {
        let prompt = format!(
            "You are a senior software engineer writing a daily work log.

            Rewrite the following git commits into impact-focused work log entries.

            STRICT RULES:
            - Output ONLY the bullet list. No introductions, explanations, examples, or conclusions.
            - Group related commits into single work items.
            - Focus on problem solved, responsibility taken, and outcome.
            - Remove low-level implementation and git details.
            - Use first person, active voice.
            - Be concise and professional. Do not exaggerate.

            FORMAT:
            - Bullet list
            - 1–2 sentences per bullet
            - Each bullet = one meaningful contribution

            COMMITS:
            {text}",
        );

        let request = GenerationRequest::new(self.model.clone(), prompt);
        let rt = tokio::runtime::Runtime::new().context("Failed to create async runtime.")?;

        let response = rt
            .block_on(async { self.client.generate(request).await })
            .context("Ollama request failed. Is the server running?")?;

        Ok(response.response)
    }
}
