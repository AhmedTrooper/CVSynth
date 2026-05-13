use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn save_model_pref(state: State<'_, AppState>, model: String) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_mut() {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('ai_model', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [&model],
        ).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Database not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_model_pref(state: State<'_, AppState>) -> Result<String, String> {
    let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_mut() {
        let mut stmt = conn
            .prepare("SELECT value FROM app_settings WHERE key = 'ai_model'")
            .map_err(|e| e.to_string())?;
        let model: String = stmt
            .query_row([], |row| row.get(0))
            .unwrap_or_else(|_| "gpt-4o".to_string());
        Ok(model)
    } else {
        Err("Database not initialized".to_string())
    }
}
