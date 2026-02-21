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

    pub fn summarize(&self, text: &str, redact_sensitive: Option<bool>) -> Result<String> {
        let redact = redact_sensitive.unwrap_or(false);
        let redact_rule = if redact {
            "Redact or generalize any company-sensitive details (customer names, internal URLs, credentials, proprietary architecture, unreleased features, incident specifics, and exact revenue/security metrics) while preserving the business impact and contribution narrative."
        } else {
            ""
        };

        let prompt = format!(
            "You are a senior software engineer writing a professional daily work journal from git commits.

            Task:
            Turn the commit list into a clear, supervisor-friendly daily summary of contributions.

            Goal:
            Explain what I accomplished today, why it mattered, and what outcomes were delivered.

            STYLE RULES:
            - Write in first person, active voice.
            - Use plain, professional language.
            - Focus on impact, ownership, and outcomes.
            - Group related commits into coherent work themes.
            - Do not copy commit messages verbatim.
            - Avoid low-level implementation details unless they are necessary for understanding impact.
            - Avoid hype and vague claims.

            OUTPUT RULES:
            - Output in Markdown.
            - Do NOT output a raw bullet list of commits.
            - Start with a short \"Daily Summary\" paragraph (3-5 sentences).
            - Then add 2-4 short sections with headings (e.g., \"Product/Feature Work\", \"Reliability & Fixes\", \"Documentation\").
            - Under each section, write concise prose describing completed work and results.
            - If commit context is ambiguous, make a conservative, clearly stated inference.
            - Do not include introductions, meta commentary, or conclusions outside the requested format.
             {redact_rule}

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
