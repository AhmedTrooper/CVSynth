use crate::ai::{self};
use crate::commands::TailoredContent;
use crate::AppState;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoverLetterItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoverLetterDetail {
    pub id: String,
    pub name: String,
    pub category: String,
    pub latex_content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCoverLetterArgs {
    pub name: String,
    pub category: String,
    pub latex_content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCoverLetterArgs {
    pub cl_id: String,
    pub name: String,
    pub category: String,
    pub latex_content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCoverLetterArgs {
    pub cl_id: String,
}

#[tauri::command]
pub fn get_all_cover_letters(state: State<'_, AppState>) -> Result<Vec<CoverLetterItem>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        let mut stmt = conn
            .prepare("SELECT id, name, category, created_at, updated_at FROM base_cover_letters ORDER BY created_at DESC")
            .map_err(|e| format!("Query prepare error: {}", e))?;

        let cls = stmt
            .query_map([], |row| {
                Ok(CoverLetterItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(|e| format!("Query error: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row collection error: {}", e))?;

        Ok(cls)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn get_cover_letter_by_id(
    state: State<'_, AppState>,
    cl_id: String,
) -> Result<CoverLetterDetail, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        let mut stmt = conn
            .prepare("SELECT id, name, category, latex_content, created_at, updated_at FROM base_cover_letters WHERE id = ?1")
            .map_err(|e| format!("Query prepare error: {}", e))?;

        let cl = stmt
            .query_row([cl_id], |row| {
                Ok(CoverLetterDetail {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    latex_content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Cover letter not found: {}", e))?;

        Ok(cl)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn create_new_cover_letter(
    state: State<'_, AppState>,
    args: CreateCoverLetterArgs,
) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        let cl_id = nanoid!(10);

        conn.execute(
            "INSERT INTO base_cover_letters (id, name, category, latex_content) VALUES (?1, ?2, ?3, ?4)",
            [&cl_id, &args.name, &args.category, &args.latex_content],
        )
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(cl_id)
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn update_cover_letter(
    state: State<'_, AppState>,
    args: UpdateCoverLetterArgs,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "UPDATE base_cover_letters SET name = ?1, category = ?2, latex_content = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?4",
            [&args.name, &args.category, &args.latex_content, &args.cl_id],
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub fn delete_cover_letter(
    state: State<'_, AppState>,
    args: DeleteCoverLetterArgs,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "DELETE FROM base_cover_letters WHERE id = ?1",
            [&args.cl_id],
        )
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    } else {
        Err("Database connection lost".to_string())
    }
}

#[tauri::command]
pub async fn tailor_cover_letter(
    state: State<'_, AppState>,
    provider: String,
    model: String,
    api_key: String,
    job_id: String,
    base_cl_id: String,
    custom_instruction: Option<String>,
) -> Result<String, String> {
    // 1. Fetch job and cover letter data
    let (raw_job_content, requirements, core_responsibilities, base_latex) = {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

        if let Some(conn) = db_guard.as_mut() {
            let mut stmt = conn
                .prepare(
                    "SELECT raw_jd, requirements, core_responsibilities FROM jobs WHERE id = ?1",
                )
                .map_err(|e| format!("Query prepare error: {}", e))?;

            let (raw_job, reqs, resps): (String, Option<String>, Option<String>) = stmt
                .query_row([&job_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map_err(|_| format!("Job not found: {}", job_id))?;

            let mut stmt = conn
                .prepare("SELECT latex_content FROM base_cover_letters WHERE id = ?1")
                .map_err(|e| format!("Query prepare error: {}", e))?;

            let latex: String = stmt
                .query_row([&base_cl_id], |row| row.get(0))
                .map_err(|_| format!("Base cover letter not found: {}", base_cl_id))?;

            (raw_job, reqs, resps, latex)
        } else {
            return Err("Database connection lost".to_string());
        }
    };

    // 2. Prepare tailored prompt content
    let job_context = format!(
        "Raw JD:\n{}\n\nExtracted Requirements:\n{}\n\nExtracted Responsibilities:\n{}",
        raw_job_content,
        requirements.unwrap_or_default(),
        core_responsibilities.unwrap_or_default()
    );

    // 3. Call AI to tailor the cover letter
    let tailored_latex = ai::tailor_latex_for_cover_letter(
        &provider,
        &model,
        &api_key,
        &base_latex,
        &job_context,
        custom_instruction.as_deref(),
    )
    .await?;

    // 4. Save to database
    {
        let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

        if let Some(conn) = db_guard.as_mut() {
            let tailored_id = nanoid!(10);

            conn.execute(
                "INSERT INTO tailored_cover_letters (id, job_id, base_cl_id, final_latex_content, is_active)
                 VALUES (?1, ?2, ?3, ?4, 1)",
                [
                    &tailored_id,
                    &job_id,
                    &base_cl_id,
                    &tailored_latex,
                ],
            ).map_err(|e| format!("Database error: {}", e))?;

            Ok(tailored_id)
        } else {
            Err("Database connection lost".to_string())
        }
    }
}

#[tauri::command]
pub async fn get_tailored_cover_letter(
    state: State<'_, AppState>,
    id: String,
) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare("SELECT final_latex_content FROM tailored_cover_letters WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let content: String = stmt
        .query_row([&id], |row| row.get(0))
        .map_err(|_| "Tailored cover letter not found".to_string())?;

    Ok(content)
}

#[tauri::command]
pub async fn get_latest_tailored_cover_letter(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<TailoredContent>, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    let mut stmt = conn
        .prepare(
            "SELECT id, final_latex_content FROM tailored_cover_letters 
         WHERE job_id = ?1 
         ORDER BY created_at DESC LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let result: Option<TailoredContent> = match stmt.query_row([&job_id], |row| {
        Ok(TailoredContent {
            id: row.get(0)?,
            content: row.get(1)?,
        })
    }) {
        Ok(v) => Some(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(e.to_string()),
    };

    Ok(result)
}

#[tauri::command]
pub async fn update_tailored_cover_letter(
    state: State<'_, AppState>,
    job_id: String,
    latex_content: String,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    conn.execute(
        "UPDATE tailored_cover_letters SET final_latex_content = ?1, updated_at = CURRENT_TIMESTAMP 
         WHERE job_id = ?2",
        [&latex_content, &job_id],
    )
    .map_err(|e| format!("Database error: {}", e))?;

    Ok(())
}
