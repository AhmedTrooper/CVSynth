use tauri::State;
use nanoid::nanoid;
use crate::AppState;
use crate::ai::{self, JobDetails};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JobPayload {
    pub id: String,
    pub company_name: String,
    pub job_title: String,
    pub work_model: String,
    pub employment_type: String,
    pub status: String,
    pub raw_jd: String,
    pub custom_instruction: Option<String>,
    pub reference_name: Option<String>,
    pub reference_email: Option<String>,
    pub social_link: Option<String>,
}

#[tauri::command]
pub async fn parse_job(
    provider: String,
    model: String,
    api_key: String,
    raw_jd: String,
) -> Result<JobDetails, String> {
    ai::parse_job_description(&provider, &model, &api_key, &raw_jd).await
}

#[tauri::command]
pub async fn save_job(
    state: State<'_, AppState>,
    payload: JobPayload,
) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    conn.execute(
        "INSERT INTO jobs (
            id, company_name, job_title, work_model, employment_type, 
            status, raw_jd, custom_instruction, reference_name, 
            reference_email, social_link
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        [
            &payload.id,
            &payload.company_name,
            &payload.job_title,
            &payload.work_model,
            &payload.employment_type,
            &payload.status,
            &payload.raw_jd,
            &payload.custom_instruction.unwrap_or_default(),
            &payload.reference_name.unwrap_or_default(),
            &payload.reference_email.unwrap_or_default(),
            &payload.social_link.unwrap_or_default(),
        ],
    ).map_err(|e| format!("Database error: {}", e))?;

    Ok(payload.id)
}

#[tauri::command]
pub async fn get_job_by_id(
    state: State<'_, AppState>,
    id: String,
) -> Result<JobPayload, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn.prepare(
        "SELECT id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, custom_instruction, reference_name, 
                reference_email, social_link 
         FROM jobs WHERE id = ?1"
    ).map_err(|e| e.to_string())?;

    let job = stmt.query_row([&id], |row| {
        Ok(JobPayload {
            id: row.get(0)?,
            company_name: row.get(1)?,
            job_title: row.get(2)?,
            work_model: row.get(3)?,
            employment_type: row.get(4)?,
            status: row.get(5)?,
            raw_jd: row.get(6)?,
            custom_instruction: row.get(7)?,
            reference_name: row.get(8)?,
            reference_email: row.get(9)?,
            social_link: row.get(10)?,
        })
    }).map_err(|e| format!("Job not found: {}", e))?;

    Ok(job)
}

#[tauri::command]
pub async fn get_all_jobs(
    state: State<'_, AppState>,
) -> Result<Vec<JobPayload>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn.prepare(
        "SELECT id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, custom_instruction, reference_name, 
                reference_email, social_link 
         FROM jobs ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let job_iter = stmt.query_map([], |row| {
        Ok(JobPayload {
            id: row.get(0)?,
            company_name: row.get(1)?,
            job_title: row.get(2)?,
            work_model: row.get(3)?,
            employment_type: row.get(4)?,
            status: row.get(5)?,
            raw_jd: row.get(6)?,
            custom_instruction: row.get(7)?,
            reference_name: row.get(8)?,
            reference_email: row.get(9)?,
            social_link: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut jobs = Vec::new();
    for job in job_iter {
        jobs.push(job.map_err(|e| e.to_string())?);
    }
    Ok(jobs)
}

#[tauri::command]
pub async fn get_tailored_resume(
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn.prepare("SELECT final_latex_content FROM tailored_resumes WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    
    let content: String = stmt.query_row([&id], |row| row.get(0))
        .map_err(|_| "Tailored resume not found".to_string())?;
    
    Ok(content)
}

#[tauri::command]
pub async fn tailor_resume(
    state: State<'_, AppState>,
    provider: String,
    model: String,
    api_key: String,
    job_id: String,
    base_resume_id: String,
    custom_instruction: Option<String>,
) -> Result<String, String> {
    // 1. Fetch job and resume data
    let (raw_job_content, base_latex) = {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
        
        if let Some(conn) = db_guard.as_mut() {
            let mut stmt = conn
                .prepare("SELECT raw_jd FROM jobs WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;
            
            let raw_job: String = stmt
                .query_row([&job_id], |row| row.get(0))
                .map_err(|_| format!("Job not found: {}", job_id))?;
            
            let mut stmt = conn
                .prepare("SELECT latex_content FROM base_resumes WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;
            
            let latex: String = stmt
                .query_row([&base_resume_id], |row| row.get(0))
                .map_err(|_| format!("Base resume not found: {}", base_resume_id))?;
            
            (raw_job, latex)
        } else {
            return Err("Database connection lost".to_string());
        }
    };
    
    // 2. Call AI to tailor the resume
    let tailored_latex = ai::tailor_latex_for_job(
        &provider,
        &model,
        &api_key,
        &base_latex,
        &raw_job_content,
        custom_instruction.as_deref(),
    )
    .await?;
    
    // 3. Save to database
    {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
        
        if let Some(conn) = db_guard.as_mut() {
            let tailored_id = nanoid!(10);
            
            conn.execute(
                "INSERT INTO tailored_resumes (id, job_id, base_resume_id, final_latex_content, is_active)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                [
                    &tailored_id,
                    &job_id,
                    &base_resume_id,
                    &tailored_latex,
                ],
            ).map_err(|e| format!("Database error: {}", e))?;
            
            Ok(tailored_id)
        } else {
            Err("Database connection lost".to_string())
        }
    }
}
