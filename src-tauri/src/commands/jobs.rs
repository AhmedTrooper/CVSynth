use tauri::State;
use nanoid::nanoid;
use crate::AppState;
use crate::ai;

#[tauri::command]
pub async fn parse_and_save_job(
    state: State<'_, AppState>, 
    api_key: String, 
    raw_jd: String
) -> Result<String, String> {
    // 1. Call the AI to parse the raw text
    let parsed_data = ai::parse_job_description(&api_key, &raw_jd).await?;
    
    // Convert the Rust struct into a JSON string for SQLite storage
    let parsed_json_string = serde_json::to_string(&parsed_data)
        .map_err(|e| format!("JSON Serialization error: {}", e))?;

    // 2. Generate a unique 10-character slug
    let job_slug = nanoid!(10);
    
    // 3. Save to SQLite
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    
    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "INSERT INTO jobs (id, company_name, job_title, raw_jd, parsed_json, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, 'Drafting')",
            [
                &job_slug, 
                &parsed_data.company, 
                &parsed_data.title, 
                &raw_jd, 
                &parsed_json_string
            ],
        ).map_err(|e| format!("Database error: {}", e))?;
        
        Ok(job_slug)
    } else {
        Err("Database connection lost".to_string())
    }
}