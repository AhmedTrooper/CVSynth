use axum::extract::State;
use axum::{Json, http::StatusCode};
use std::sync::Arc;
use crate::AppState;
use crate::models::AiConfig;

pub async fn get_ai_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AiConfig>, (StatusCode, String)> {
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let provider: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'selected_provider'",
        [],
        |row| row.get(0)
    ).unwrap_or_else(|_| "openai".to_string());

    let model: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'selected_model'",
        [],
        |row| row.get(0)
    ).unwrap_or_else(|_| "gpt-4o".to_string());

    let has_key: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM app_settings WHERE key = 'api_key_{}'", provider),
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    Ok(Json(AiConfig {
        provider,
        model,
        has_key: has_key > 0,
    }))
}

pub async fn save_ai_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let provider = payload["provider"].as_str().ok_or((StatusCode::BAD_REQUEST, "provider missing".to_string()))?;
    let model = payload["model"].as_str().ok_or((StatusCode::BAD_REQUEST, "model missing".to_string()))?;
    
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('selected_provider', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [provider],
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('selected_model', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [model],
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(api_key) = payload["apiKey"].as_str() {
        if !api_key.is_empty() {
             conn.execute(
                &format!("INSERT INTO app_settings (key, value) VALUES ('api_key_{}', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", provider),
                [api_key],
            ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    Ok(StatusCode::OK)
}

pub async fn get_api_key(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Option<String>>, (StatusCode, String)> {
    let provider = params.get("provider").ok_or((StatusCode::BAD_REQUEST, "provider missing".to_string()))?;
    let conn = state.db.lock().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let key: Option<String> = match conn.query_row(
        &format!("SELECT value FROM app_settings WHERE key = 'api_key_{}'", provider),
        [],
        |row| row.get(0)
    ) {
        Ok(v) => Some(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    Ok(Json(key))
}
