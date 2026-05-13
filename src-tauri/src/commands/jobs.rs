use crate::AppState;
use nanoid::nanoid;
use tauri::State;

#[tauri::command]
pub async fn create_job(
    state: State<'_, AppState>,
    title: String,
    company: String,
) -> Result<String, String> {
    // Generate a unique 10-character slug (e.g., "V1StGXR8_Z")
    let job_slug = nanoid!(10);

    // Safely lock the database mutex
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;

    if let Some(conn) = db_guard.as_mut() {
        // Notice we are assuming `id` is now a TEXT field in SQLite, not INTEGER
        conn.execute(
            "INSERT INTO jobs (id, job_title, company_name, raw_jd) VALUES (?1, ?2, ?3, '')",
            [&job_slug, &title, &company],
        )
        .map_err(|e| format!("Database error: {}", e))?;

        // Return the slug so the Vue frontend can immediately route to /job/{job_slug}
        Ok(job_slug)
    } else {
        Err("Database connection lost".to_string())
    }
}
