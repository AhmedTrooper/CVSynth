use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::{anthropic, gemini, groq, openai};
use scraper::Html;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct JobDetails {
    pub is_valid_job: bool,      // AI will set this to false if the content is not a job description
    pub job_title: String,
    pub company_name: String,
    pub work_model: String,      // Remote, Hybrid, On-site, Other
    pub employment_type: String, // Full-time, Part-time, Contract, Freelance, Temporary, Internship
    pub requirements: Vec<String>,
    pub core_responsibilities: Vec<String>,
}

async fn fetch_url_content(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(url).send().await.map_err(|e| format!("Failed to fetch URL: {}. Please ensure the link is correct and public.", e))?;
    let html = resp.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;

    let document = Html::parse_document(&html);
    let mut text_content = String::new();
    
    // Iterate over all text nodes in the document
    for node in document.root_element().text() {
        text_content.push_str(node);
        text_content.push(' ');
    }

    let cleaned = text_content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.len() < 100 {
        return Err("The fetched page content seems too short to be a job description. Try copying the description manually.".to_string());
    }

    // Limit text to avoid hitting context limits, but keep enough for JD
    let limited = if cleaned.len() > 10000 {
        cleaned.chars().take(10000).collect()
    } else {
        cleaned
    };

    Ok(limited)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobParseResult {
    pub details: JobDetails,
    pub raw_description: String,
}

pub async fn parse_job_description(
    provider: &str,
    model: &str,
    api_key: &str,
    raw_jd: &str,
    job_url: Option<&str>,
) -> Result<JobParseResult, String> {
    let mut input_text = raw_jd.trim().to_string();

    if input_text.is_empty() {
        if let Some(url) = job_url {
            input_text = fetch_url_content(url).await?;
        } else {
            return Err("Either a job description or a URL must be provided.".to_string());
        }
    }

    let model = model.trim();
    
    let prompt_prefix = "You are a job description extractor. Analyze the following text and extract structured details. 
    IMPORTANT: If the text DOES NOT contain a job description (e.g., it's a login page, cookie notice, generic website content, or just noise), you MUST set 'is_valid_job' to false. Otherwise, set it to true.\n\nText to analyze:\n";

    let full_prompt = format!("{}{}", prompt_prefix, input_text);

    let details = match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor
                .extract(&full_prompt)
                .await
                .map_err(|e| format!("Gemini AI Parsing Error: {}", e))?
        }
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor
                .extract(&full_prompt)
                .await
                .map_err(|e| format!("OpenAI Parsing Error: {}", e))?
        }
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor
                .extract(&full_prompt)
                .await
                .map_err(|e| format!("Groq Parsing Error: {}", e))?
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let extractor = client.extractor::<JobDetails>(model).build();
            extractor
                .extract(&full_prompt)
                .await
                .map_err(|e| format!("Anthropic Parsing Error: {}", e))?
        }
        _ => return Err(format!("Unsupported provider: {}", provider)),
    };

    if !details.is_valid_job {
        return Err("The content provided (or fetched) does not appear to be a job description. Please paste the description manually.".to_string());
    }

    Ok(JobParseResult {
        details,
        raw_description: input_text,
    })
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
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Gemini AI Tailoring Error: {}", e))
        }
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("OpenAI Tailoring Error: {}", e))
        }
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Groq Tailoring Error: {}", e))
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Anthropic Tailoring Error: {}", e))
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

pub async fn tailor_latex_for_cover_letter(
    provider: &str,
    model: &str,
    api_key: &str,
    base_latex: &str,
    raw_job_content: &str,
    custom_instruction: Option<&str>,
) -> Result<String, String> {
    let model = model.trim();
    let system_prompt = r#"You are an expert cover letter tailoring AI. Your task is to take a base LaTeX cover letter template and tailor it to match a specific job description. 
    
Rules:
1. Only modify the cover letter content (e.g., recipient info, body paragraphs), NOT the structure or LaTeX commands unless necessary for content.
2. Emphasize how the candidate's skills and experiences align with the job requirements.
3. Maintain a professional, persuasive, and concise tone.
4. Keep all original sections and formatting.
5. Output ONLY valid LaTeX code with no markdown, no explanations, no code fences.
6. Ensure the output is a valid, compilable LaTeX document.

If custom instructions are provided, prioritize them."#;

    let user_prompt = format!(
        r#"Base LaTeX Cover Letter:
{}

Job Description:
{}

{}

Please tailor the cover letter to match the job description. Return only the modified LaTeX code."#,
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
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Gemini AI Tailoring Error: {}", e))
        }
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("OpenAI Tailoring Error: {}", e))
        }
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Groq Tailoring Error: {}", e))
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Anthropic Tailoring Error: {}", e))
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

pub async fn refine_tailored_resume(
    provider: &str,
    model: &str,
    api_key: &str,
    current_latex: &str,
    instruction: &str,
) -> Result<String, String> {
    let model = model.trim();
    let system_prompt = r#"You are an expert LaTeX resume editor. Your task is to take an EXISTING tailored resume and apply specific refinements or formatting changes as requested by the user.

Rules:
1. Preserve all existing content and structure unless specifically asked to change it.
2. Maintain valid LaTeX syntax at all times.
3. Output ONLY the modified LaTeX code with no markdown, no explanations, no code fences.
4. Ensure the output is a valid, compilable LaTeX document."#;

    let user_prompt = format!(
        r#"Current LaTeX Resume:
{}

Requested Refinement:
{}

Please apply the requested changes. Return only the updated LaTeX code."#,
        current_latex, instruction
    );

    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Gemini AI Refinement Error: {}", e))
        }
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("OpenAI Refinement Error: {}", e))
        }
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Groq Refinement Error: {}", e))
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Anthropic Refinement Error: {}", e))
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}

pub async fn fix_latex_errors(
    provider: &str,
    model: &str,
    api_key: &str,
    broken_latex: &str,
    error_logs: &str,
) -> Result<String, String> {
    let model = model.trim();
    let system_prompt = r#"You are an expert LaTeX debugger. Your task is to fix syntax errors, missing packages, or illegal characters in LaTeX code based on provided error logs.

Rules:
1. Fix the specific errors mentioned in the logs.
2. DO NOT change the resume content or structure unless necessary to fix the error.
3. Output ONLY the corrected LaTeX code with no markdown, no explanations, no code fences.
4. Ensure the output is a valid, compilable LaTeX document."#;

    let user_prompt = format!(
        r#"Broken LaTeX Code:
{}

Tectonic Error Logs:
{}

Please fix the LaTeX code so it compiles successfully. Return only the fixed LaTeX code."#,
        broken_latex, error_logs
    );

    match provider {
        "gemini" => {
            let client = gemini::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Gemini AI Fix Error: {}", e))
        }
        "openai" => {
            let client = openai::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("OpenAI Fix Error: {}", e))
        }
        "groq" => {
            let client = groq::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Groq Fix Error: {}", e))
        }
        "anthropic" => {
            let client = anthropic::Client::new(api_key).map_err(|e| e.to_string())?;
            let agent = client.agent(model).preamble(system_prompt).build();
            agent
                .prompt(&user_prompt)
                .await
                .map_err(|e| format!("Anthropic Fix Error: {}", e))
        }
        _ => Err(format!("Unsupported provider: {}", provider)),
    }
}
