use rig::providers::openai::Client;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct JobDetails {
    pub title: String,
    pub company: String,
    pub requirements: Vec<String>,
    pub core_responsibilities: Vec<String>,
}

pub async fn parse_job_description(api_key: &str, raw_jd: &str) -> Result<JobDetails, String> {
    // FIX: Unpack the Result. If Client::new fails, it safely returns the error as a String.
    let client = Client::new(api_key)
        .map_err(|e| format!("Failed to initialize AI client: {}", e))?;
    
    // Now 'client' is safely unwrapped, and we can build the extractor
    let extractor = client.extractor::<JobDetails>("gpt-4o").build();
        
    let result = extractor.extract(raw_jd).await.map_err(|e| format!("AI Parsing Error: {}", e))?;
    
    Ok(result)
}