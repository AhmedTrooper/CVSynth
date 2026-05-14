use rig::providers::{gemini, openai, groq, anthropic};
use rig::completion::Prompt;
use rig::client::CompletionClient;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct JobDetails {
    pub job_title: String,
    pub company_name: String,
    pub work_model: String,      // Remote, Hybrid, On-site, Other
    pub employment_type: String, // Full-time, Part-time, Contract, Freelance, Temporary, Internship
    pub requirements: Vec<String>,
    pub core_responsibilities: Vec<String>,
}

pub async fn parse_job_description(
    provider: &str,
    model: &str,
    api_key: &str, 
    raw_jd: &str
) -> Result<JobDetails, String> {
    let model = model.trim();
    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor.extract(raw_jd).await.map_err(|e| format!("Gemini AI Parsing Error: {}", e))
        },
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor.extract(raw_jd).await.map_err(|e| format!("OpenAI Parsing Error: {}", e))
        },
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor.extract(raw_jd).await.map_err(|e| format!("Groq Parsing Error: {}", e))
        },
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor.extract(raw_jd).await.map_err(|e| format!("Anthropic Parsing Error: {}", e))
        },
        _ => Err(format!("Unsupported provider: {}", provider))
    }
}

pub async fn tailor_latex_for_job(
    provider: &str,
    model: &str,
    api_key: &str,
    base_latex: &str,
    raw_job_content: &str,
    custom_instruction: Option<&str>,
) -> Result<String, String> {
    let model = model.trim();
    let system_prompt = r#"You are an expert resume tailoring AI. Your task is to take a base LaTeX resume template and tailor it to match a specific job description. 
    
Rules:
1. Only modify the resume content, NOT the structure or LaTeX commands
2. Highlight keywords and experiences that match the job description
3. Keep all original sections and formatting
4. Output ONLY valid LaTeX code with no markdown, no explanations, no code fences
5. Ensure the output is a valid, compilable LaTeX document

If custom instructions are provided, prioritize them."#;

    let user_prompt = format!(
        r#"Base LaTeX Resume:
{}

Job Description:
{}

{}

Please tailor the resume to match the job description. Return only the modified LaTeX code."#,
        base_latex,
        raw_job_content,
        custom_instruction
            .map(|ci| format!("Custom Instructions:\n{}", ci))
            .unwrap_or_default()
    );

    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent.prompt(&user_prompt).await.map_err(|e| format!("Gemini AI Tailoring Error: {}", e))
        },
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent.prompt(&user_prompt).await.map_err(|e| format!("OpenAI Tailoring Error: {}", e))
        },
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent.prompt(&user_prompt).await.map_err(|e| format!("Groq Tailoring Error: {}", e))
        },
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent.prompt(&user_prompt).await.map_err(|e| format!("Anthropic Tailoring Error: {}", e))
        },
        _ => Err(format!("Unsupported provider: {}", provider))
    }
}
