use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// Create a struct to easily return both settings to the frontend
#[derive(Serialize, Deserialize)]
pub struct AiConfig {
    provider: String,
    model: String,
}

#[tauri::command]
pub async fn save_model_pref(
    state: State<'_, AppState>,
    provider: String,
    model: String,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_mut() {
        // Save Provider
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('ai_provider', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [&provider],
        ).map_err(|e| e.to_string())?;

        // Save Model
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
pub async fn get_model_pref(state: State<'_, AppState>) -> Result<AiConfig, String> {
    let mut db_guard = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(conn) = db_guard.as_mut() {
        // Fetch Provider (Default to 'openai')
        let provider: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_provider'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "openai".to_string());

        // Fetch Model (Default to 'gpt-4o')
        let model: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'ai_model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "gpt-4o".to_string());

        Ok(AiConfig { provider, model })
    } else {
        Err("Database not initialized".to_string())
    }
}
