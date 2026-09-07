use crate::commands::cover_letters::CoverLetterDetail;
use crate::commands::documents::{is_text_extension, DocumentSummary};
use crate::commands::downloads::DownloadRecord;
use crate::commands::error_logs::ErrorAuditLog;
use crate::commands::hr_templates::HrTemplateItem;
use crate::commands::inbox::InboxJob;
use crate::commands::jobs::JobPayload;
use crate::commands::outreach::OutreachLeadItem;
use crate::commands::resumes::ResumeDetail;
use crate::AppState;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

/// Keys that are sensitive (secrets, credentials) or per-installation
/// runtime values that must never be exported or overwritten by imports.
const SENSITIVE_EXACT_KEYS: &[&str] = &["extension_secret", "active_server_port"];

/// Prefix patterns — any key starting with one of these is sensitive.
const SENSITIVE_PREFIXES: &[&str] = &["s3_", "aws_", "cloud_"];

/// Substring patterns — any key containing one of these is sensitive.
const SENSITIVE_SUBSTRINGS: &[&str] = &[
    "api_key",
    "secret",
    "token",
    "password",
    "credential",
    "bucket",
];

pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();

    if SENSITIVE_EXACT_KEYS.iter().any(|k| lower == *k) {
        return true;
    }

    if SENSITIVE_PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return true;
    }

    if SENSITIVE_SUBSTRINGS.iter().any(|s| lower.contains(s)) {
        return true;
    }

    false
}

/// Snapshot all sensitive settings from the database.
fn snapshot_sensitive_settings(conn: &Connection) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .unwrap_or_else(|_| panic!("Failed to prepare snapshot query"));

    stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })
    .unwrap_or_else(|_| panic!("Failed to query settings"))
    .filter_map(|r| r.ok())
    .filter(|(k, _)| is_sensitive_key(k))
    .collect()
}

/// Restore previously-snapshotted sensitive settings back into the database.
fn restore_sensitive_settings(conn: &Connection, snapshot: &[(String, String)]) {
    for (key, value) in snapshot {
        let _ = conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [key, value],
        );
    }
}

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

fn default_true() -> bool {
    true
}

fn default_timestamp() -> String {
    chrono::Local::now().to_rfc3339()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TailoredResumeExport {
    pub id: String,
    pub job_id: String,
    pub base_resume_id: String,
    pub final_latex_content: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TailoredCoverLetterExport {
    pub id: String,
    pub job_id: String,
    pub base_cl_id: String,
    pub final_latex_content: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThemeExport {
    pub id: String,
    pub name: String,
    pub config: String,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettingExport {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocumentFileExport {
    pub doc_id: String,
    pub rel_path: String,
    pub content: String,
    #[serde(default)]
    pub size_bytes: i64,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppDataExport {
    #[serde(default)]
    pub schema_version: Option<u32>,
    #[serde(default)]
    pub jobs: Vec<JobPayload>,
    #[serde(default)]
    pub base_resumes: Vec<ResumeDetail>,
    #[serde(default)]
    pub base_cover_letters: Vec<CoverLetterDetail>,
    #[serde(default)]
    pub tailored_resumes: Vec<TailoredResumeExport>,
    #[serde(default)]
    pub tailored_cover_letters: Vec<TailoredCoverLetterExport>,
    #[serde(default)]
    pub downloads: Vec<DownloadRecord>,
    #[serde(default)]
    pub themes: Vec<ThemeExport>,
    #[serde(default)]
    pub app_settings: Vec<SettingExport>,
    #[serde(default)]
    pub inbox_jobs: Vec<InboxJob>,
    #[serde(default)]
    pub compiler_state: Option<String>,
    #[serde(default)]
    pub documents: Vec<DocumentSummary>,
    #[serde(default)]
    pub document_files: Vec<DocumentFileExport>,
    #[serde(default)]
    pub hr_templates: Vec<HrTemplateItem>,
    #[serde(default)]
    pub outreach_leads: Vec<OutreachLeadItem>,
    #[serde(default)]
    pub error_audit_logs: Vec<ErrorAuditLog>,
    #[serde(default = "default_timestamp")]
    pub exported_at: String,
}

/// Parses backup data from a `serde_json::Value`.
/// Unwraps nested wrapper structures (`data`, `backup`, `export`, `payload`, `vault`),
/// validates against known RoleTect keys, and deserializes with safe defaults.
pub fn parse_backup_value(value: serde_json::Value) -> Result<AppDataExport, String> {
    match value {
        serde_json::Value::String(s) => parse_backup_json(&s),
        serde_json::Value::Object(map) => {
            // Check for wrapper keys commonly used in API responses or export tools
            for wrapper_key in &["data", "backup", "export", "payload", "vault"] {
                if let Some(inner) = map.get(*wrapper_key) {
                    if inner.is_object() {
                        return parse_backup_value(inner.clone());
                    }
                }
            }

            // Check if map contains any recognized RoleTect keys to avoid accidentally
            // deserializing unrelated JSON (like package.json) into empty vectors
            const RECOGNIZED_KEYS: &[&str] = &[
                "jobs",
                "base_resumes",
                "base_cover_letters",
                "tailored_resumes",
                "tailored_cover_letters",
                "downloads",
                "themes",
                "app_settings",
                "inbox_jobs",
                "compiler_state",
                "documents",
                "document_files",
                "hr_templates",
                "outreach_leads",
                "error_audit_logs",
                "exported_at",
                "schema_version",
            ];

            let has_recognized_key = map.keys().any(|k| RECOGNIZED_KEYS.contains(&k.as_str()));
            if !has_recognized_key {
                return Err(
                    "Invalid backup file: Not a recognized RoleTect backup structure.".to_string(),
                );
            }

            serde_json::from_value::<AppDataExport>(serde_json::Value::Object(map))
                .map_err(|e| format!("Failed to parse backup data: {}", e))
        }
        _ => Err("Invalid backup file: Expected a JSON object or string.".to_string()),
    }
}

/// Robust JSON string parser that handles UTF-8 Byte Order Marks (BOM),
/// leading/trailing whitespace, and delegates to `parse_backup_value`.
pub fn parse_backup_json(raw: &str) -> Result<AppDataExport, String> {
    let mut trimmed = raw.trim();
    // Strip UTF-8 BOM if present
    if trimmed.starts_with('\u{feff}') {
        trimmed = &trimmed['\u{feff}'.len_utf8()..];
    }
    let trimmed = trimmed.trim();
    if trimmed.is_empty() {
        return Err("Backup file is empty.".to_string());
    }

    let val: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("Invalid JSON format: {}", e))?;

    parse_backup_value(val)
}

#[tauri::command]
pub fn export_all_data_core(state: &AppState) -> Result<AppDataExport, String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    // 1. Fetch Jobs
    let mut stmt = conn
        .prepare(
            "SELECT id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, requirements, core_responsibilities,
                custom_instruction, reference_name, 
                reference_email, social_link, job_url,
                base_resume_id, base_cl_id,
                salary, applied_date, interview_date, offer_date, rejected_date, joining_date,
                created_at, updated_at
         FROM jobs",
        )
        .map_err(|e| e.to_string())?;

    let jobs = stmt
        .query_map([], |row| {
            Ok(JobPayload {
                id: row.get(0)?,
                company_name: row.get(1)?,
                job_title: row.get(2)?,
                work_model: row.get(3)?,
                employment_type: row.get(4)?,
                status: row.get(5)?,
                raw_jd: row.get(6)?,
                requirements: row.get(7)?,
                core_responsibilities: row.get(8)?,
                custom_instruction: row.get(9)?,
                reference_name: row.get(10)?,
                reference_email: row.get(11)?,
                social_link: row.get(12)?,
                job_url: row.get(13)?,
                base_resume_id: row.get(14)?,
                base_cl_id: row.get(15)?,
                salary: row.get(16)?,
                applied_date: row.get(17)?,
                interview_date: row.get(18)?,
                offer_date: row.get(19)?,
                rejected_date: row.get(20)?,
                joining_date: row.get(21)?,
                created_at: Some(row.get(22)?),
                updated_at: Some(row.get(23)?),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 2. Fetch Base Resumes
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, latex_content, created_at, updated_at FROM base_resumes",
        )
        .map_err(|e| e.to_string())?;

    let base_resumes = stmt
        .query_map([], |row| {
            Ok(ResumeDetail {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                latex_content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 2b. Fetch Base Cover Letters
    let mut stmt = conn.prepare(
        "SELECT id, name, category, latex_content, created_at, updated_at FROM base_cover_letters"
    ).map_err(|e| e.to_string())?;

    let base_cover_letters = stmt
        .query_map([], |row| {
            Ok(CoverLetterDetail {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                latex_content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 3. Fetch Tailored Resumes
    let mut stmt = conn.prepare(
        "SELECT id, job_id, base_resume_id, final_latex_content, is_active, created_at, updated_at 
         FROM tailored_resumes"
    ).map_err(|e| e.to_string())?;

    let tailored_resumes = stmt
        .query_map([], |row| {
            Ok(TailoredResumeExport {
                id: row.get(0)?,
                job_id: row.get(1)?,
                base_resume_id: row.get(2)?,
                final_latex_content: row.get(3)?,
                is_active: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 3b. Fetch Tailored Cover Letters
    let mut stmt = conn
        .prepare(
            "SELECT id, job_id, base_cl_id, final_latex_content, is_active, created_at, updated_at 
         FROM tailored_cover_letters",
        )
        .map_err(|e| e.to_string())?;

    let tailored_cover_letters = stmt
        .query_map([], |row| {
            Ok(TailoredCoverLetterExport {
                id: row.get(0)?,
                job_id: row.get(1)?,
                base_cl_id: row.get(2)?,
                final_latex_content: row.get(3)?,
                is_active: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 4. Fetch Downloads
    let mut stmt = conn
        .prepare(
            "SELECT id, filename, download_type, job_id, content_id, created_at FROM downloads",
        )
        .map_err(|e| e.to_string())?;

    let downloads = stmt
        .query_map([], |row| {
            Ok(DownloadRecord {
                id: row.get(0)?,
                filename: row.get(1)?,
                download_type: row.get(2)?,
                job_id: row.get(3)?,
                content_id: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 5. Fetch Themes
    let mut stmt = conn
        .prepare("SELECT id, name, config, is_builtin, created_at FROM themes")
        .map_err(|e| e.to_string())?;

    let themes = stmt
        .query_map([], |row| {
            Ok(ThemeExport {
                id: row.get(0)?,
                name: row.get(1)?,
                config: row.get(2)?,
                is_builtin: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 6. Fetch App Settings (excluding sensitive keys)
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| e.to_string())?;

    let app_settings: Vec<SettingExport> = stmt
        .query_map([], |row| {
            Ok(SettingExport {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|s| !is_sensitive_key(&s.key))
        .collect();

    // 7. Fetch Inbox Jobs
    let mut stmt = conn
        .prepare("SELECT id, url, raw_description, status, created_at FROM inbox_jobs")
        .map_err(|e| e.to_string())?;

    let inbox_jobs = stmt
        .query_map([], |row| {
            Ok(InboxJob {
                id: row.get(0)?,
                url: row.get(1)?,
                raw_description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 8. Fetch Compiler State
    let compiler_state: Option<String> = conn
        .query_row(
            "SELECT latex_content FROM compiler_state WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .flatten();

    // 9. Fetch Documents (metadata)
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, tags, starred, main_file, last_compiled_at, compile_status, created_at, updated_at FROM documents ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let documents = stmt
        .query_map([], |row| {
            Ok(DocumentSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                tags: row.get(3)?,
                starred: row.get::<_, i64>(4)? != 0,
                main_file: row.get(5)?,
                last_compiled_at: row.get(6)?,
                compile_status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 10. Fetch Document Files (text only; binaries excluded).
    let mut stmt = conn
        .prepare("SELECT doc_id, rel_path, content, size_bytes, updated_at FROM document_files ORDER BY doc_id, rel_path")
        .map_err(|e| e.to_string())?;

    let mut skipped_binary_files: Vec<String> = Vec::new();
    let document_files: Vec<DocumentFileExport> = stmt
        .query_map([], |row| {
            Ok(DocumentFileExport {
                doc_id: row.get(0)?,
                rel_path: row.get(1)?,
                content: row.get(2)?,
                size_bytes: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|f| {
            if is_text_extension(&f.rel_path) {
                true
            } else {
                skipped_binary_files.push(format!("{}::{}", f.doc_id, f.rel_path));
                false
            }
        })
        .collect();

    if !skipped_binary_files.is_empty() {
        eprintln!(
            "[backup] Excluded {} binary/non-text document file(s) from export \
             (.png/.jpg/.pdf etc. are not part of the backup):",
            skipped_binary_files.len()
        );
        for entry in skipped_binary_files.iter().take(20) {
            eprintln!("[backup]   - {}", entry);
        }
        if skipped_binary_files.len() > 20 {
            eprintln!(
                "[backup]   - ... and {} more",
                skipped_binary_files.len() - 20
            );
        }
    }

    // 11. Fetch HR Templates
    let mut stmt = conn
        .prepare("SELECT id, name, category, content, created_at, updated_at FROM hr_templates")
        .map_err(|e| e.to_string())?;
    let hr_templates = stmt
        .query_map([], |row| {
            Ok(HrTemplateItem {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 12. Fetch Outreach Leads
    let mut stmt = conn
        .prepare(
            "SELECT id, person_name, profile_url, headline, raw_bio, recent_posts, 
                    template_id, char_limit, tailored_message, status, created_at, updated_at 
             FROM outreach_leads",
        )
        .map_err(|e| e.to_string())?;
    let outreach_leads = stmt
        .query_map([], |row| {
            let posts_raw: String = row.get::<_, Option<String>>(5)?.unwrap_or_default();
            let recent_posts: Vec<String> = serde_json::from_str(&posts_raw).unwrap_or_default();

            Ok(OutreachLeadItem {
                id: row.get(0)?,
                person_name: row.get(1)?,
                profile_url: row.get(2)?,
                headline: row.get(3)?,
                raw_bio: row.get(4)?,
                recent_posts,
                template_id: row.get(6)?,
                char_limit: row.get(7)?,
                tailored_message: row.get(8)?,
                status: row
                    .get::<_, Option<String>>(9)?
                    .unwrap_or_else(|| "Draft".to_string()),
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 13. Fetch Error Audit Logs
    let mut stmt = conn
        .prepare(
            "SELECT id, task_type, error_type, message, details, source, created_at 
             FROM error_audit_logs ORDER BY created_at DESC LIMIT 500",
        )
        .map_err(|e| e.to_string())?;
    let error_audit_logs = stmt
        .query_map([], |row| {
            Ok(ErrorAuditLog {
                id: row.get(0)?,
                task_type: row.get(1)?,
                error_type: row.get(2)?,
                message: row.get(3)?,
                details: row.get(4)?,
                source: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(AppDataExport {
        schema_version: Some(CURRENT_SCHEMA_VERSION),
        jobs,
        base_resumes,
        base_cover_letters,
        tailored_resumes,
        tailored_cover_letters,
        downloads,
        themes,
        app_settings,
        inbox_jobs,
        compiler_state,
        documents,
        document_files,
        hr_templates,
        outreach_leads,
        error_audit_logs,
        exported_at: chrono::Local::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn export_all_data(state: State<'_, AppState>) -> Result<AppDataExport, String> {
    export_all_data_core(&state)
}

/// Core database mutation logic for importing vault backup data into any SQLite connection.
/// Handles foreign keys safely, ensures transactional integrity, and respects mode ('merge' vs 'overwrite').
pub fn import_data_to_conn(
    conn: &mut Connection,
    data: &AppDataExport,
    mode: &str,
) -> Result<(), String> {
    let normalized_mode = mode.trim().to_lowercase();
    if normalized_mode != "merge" && normalized_mode != "overwrite" {
        return Err(format!(
            "Invalid restore mode '{}'. Must be 'merge' or 'overwrite'.",
            mode
        ));
    }

    // Snapshot ALL sensitive settings BEFORE any mutations so they survive import.
    let sensitive_snapshot = snapshot_sensitive_settings(conn);

    // Disable foreign keys so restoring entities with inter-table references
    // never causes constraint failures during the batch operation.
    conn.execute("PRAGMA foreign_keys = OFF;", [])
        .map_err(|e| format!("Failed to disable foreign keys: {}", e))?;

    let import_res = (|| -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        if normalized_mode == "overwrite" {
            // Clear all 15 tables in correct dependency order
            tx.execute("DELETE FROM downloads", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM tailored_cover_letters", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM tailored_resumes", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM jobs", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM base_cover_letters", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM base_resumes", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM compiler_state", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM themes WHERE is_builtin = 0", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM inbox_jobs", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM document_files", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM documents", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM outreach_leads", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM hr_templates", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM error_audit_logs", [])
                .map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM app_settings", [])
                .map_err(|e| e.to_string())?;
        }

        let now = chrono::Local::now().to_rfc3339();

        // 1. Import Base Resumes
        for resume in &data.base_resumes {
            let created_at = if resume.created_at.is_empty() { &now } else { &resume.created_at };
            let updated_at = if resume.updated_at.is_empty() { &now } else { &resume.updated_at };
            tx.execute(
                "INSERT INTO base_resumes (id, name, category, latex_content, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET 
                    name=excluded.name, 
                    category=excluded.category, 
                    latex_content=excluded.latex_content,
                    updated_at=excluded.updated_at",
                (
                    &resume.id,
                    &resume.name,
                    &resume.category,
                    &resume.latex_content,
                    created_at,
                    updated_at,
                ),
            )
            .map_err(|e| format!("Failed to import base resume {}: {}", resume.id, e))?;
        }

        // 1b. Import Base Cover Letters
        for cl in &data.base_cover_letters {
            let created_at = if cl.created_at.is_empty() { &now } else { &cl.created_at };
            let updated_at = if cl.updated_at.is_empty() { &now } else { &cl.updated_at };
            tx.execute(
                "INSERT INTO base_cover_letters (id, name, category, latex_content, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET 
                    name=excluded.name, 
                    category=excluded.category, 
                    latex_content=excluded.latex_content,
                    updated_at=excluded.updated_at",
                (
                    &cl.id,
                    &cl.name,
                    &cl.category,
                    &cl.latex_content,
                    created_at,
                    updated_at,
                ),
            ).map_err(|e| format!("Failed to import base cover letter {}: {}", cl.id, e))?;
        }

        // 2. Import Jobs
        for job in &data.jobs {
            let created_at = job.created_at.as_deref().unwrap_or(&now);
            let updated_at = job.updated_at.as_deref().unwrap_or(&now);
            tx.execute(
                "INSERT INTO jobs (
                    id, company_name, job_title, work_model, employment_type, 
                    status, raw_jd, requirements, core_responsibilities,
                    custom_instruction, reference_name, 
                    reference_email, social_link, job_url,
                    base_resume_id, base_cl_id,
                    salary, applied_date, interview_date, offer_date, rejected_date, joining_date,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)
                ON CONFLICT(id) DO UPDATE SET 
                    company_name=excluded.company_name,
                    job_title=excluded.job_title,
                    work_model=excluded.work_model,
                    employment_type=excluded.employment_type,
                    status=excluded.status,
                    raw_jd=excluded.raw_jd,
                    requirements=excluded.requirements,
                    core_responsibilities=excluded.core_responsibilities,
                    custom_instruction=excluded.custom_instruction,
                    reference_name=excluded.reference_name,
                    reference_email=excluded.reference_email,
                    social_link=excluded.social_link,
                    job_url=excluded.job_url,
                    base_resume_id=excluded.base_resume_id,
                    base_cl_id=excluded.base_cl_id,
                    salary=excluded.salary,
                    applied_date=excluded.applied_date,
                    interview_date=excluded.interview_date,
                    offer_date=excluded.offer_date,
                    rejected_date=excluded.rejected_date,
                    joining_date=excluded.joining_date,
                    updated_at=excluded.updated_at",
                rusqlite::params![
                    &job.id,
                    &job.company_name,
                    &job.job_title,
                    &job.work_model,
                    &job.employment_type,
                    &job.status,
                    &job.raw_jd,
                    &job.requirements,
                    &job.core_responsibilities,
                    &job.custom_instruction,
                    &job.reference_name,
                    &job.reference_email,
                    &job.social_link,
                    &job.job_url,
                    &job.base_resume_id,
                    &job.base_cl_id,
                    &job.salary,
                    &job.applied_date,
                    &job.interview_date,
                    &job.offer_date,
                    &job.rejected_date,
                    &job.joining_date,
                    created_at,
                    updated_at,
                ],
            )
            .map_err(|e| format!("Failed to import job {}: {}", job.id, e))?;
        }

        // 3. Import Tailored Resumes
        for tailored in &data.tailored_resumes {
            let created_at = if tailored.created_at.is_empty() { &now } else { &tailored.created_at };
            let updated_at = if tailored.updated_at.is_empty() { &now } else { &tailored.updated_at };
            tx.execute(
                "INSERT INTO tailored_resumes (id, job_id, base_resume_id, final_latex_content, is_active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET 
                    job_id=excluded.job_id,
                    base_resume_id=excluded.base_resume_id,
                    final_latex_content=excluded.final_latex_content,
                    is_active=excluded.is_active,
                    updated_at=excluded.updated_at",
                (
                    &tailored.id,
                    &tailored.job_id,
                    &tailored.base_resume_id,
                    &tailored.final_latex_content,
                    &tailored.is_active,
                    created_at,
                    updated_at,
                ),
            ).map_err(|e| format!("Failed to import tailored resume {}: {}", tailored.id, e))?;
        }

        // 3b. Import Tailored Cover Letters
        for tailored in &data.tailored_cover_letters {
            let created_at = if tailored.created_at.is_empty() { &now } else { &tailored.created_at };
            let updated_at = if tailored.updated_at.is_empty() { &now } else { &tailored.updated_at };
            tx.execute(
                "INSERT INTO tailored_cover_letters (id, job_id, base_cl_id, final_latex_content, is_active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET 
                    job_id=excluded.job_id,
                    base_cl_id=excluded.base_cl_id,
                    final_latex_content=excluded.final_latex_content,
                    is_active=excluded.is_active,
                    updated_at=excluded.updated_at",
                (
                    &tailored.id,
                    &tailored.job_id,
                    &tailored.base_cl_id,
                    &tailored.final_latex_content,
                    &tailored.is_active,
                    created_at,
                    updated_at,
                ),
            ).map_err(|e| format!("Failed to import tailored cover letter {}: {}", tailored.id, e))?;
        }

        // 4. Import Downloads
        for download in &data.downloads {
            let created_at = if download.created_at.is_empty() { &now } else { &download.created_at };
            tx.execute(
                "INSERT INTO downloads (id, filename, download_type, job_id, content_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    filename=excluded.filename,
                    download_type=excluded.download_type,
                    job_id=excluded.job_id,
                    content_id=excluded.content_id",
                (
                    &download.id,
                    &download.filename,
                    &download.download_type,
                    &download.job_id,
                    &download.content_id,
                    created_at,
                ),
            )
            .map_err(|e| format!("Failed to import download record {}: {}", download.id, e))?;
        }

        // 5. Import Themes (Bulletproof collision handling: matches by id or name)
        for theme in &data.themes {
            if theme.is_builtin {
                continue;
            }
            let is_name_builtin: bool = tx
                .query_row(
                    "SELECT is_builtin FROM themes WHERE name = ?1",
                    [&theme.name],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if is_name_builtin {
                // Built-in theme with same name takes precedence
                continue;
            }

            // Safely delete any non-builtin theme matching either id or name to avoid unique constraint collisions
            let _ = tx.execute(
                "DELETE FROM themes WHERE (id = ?1 OR name = ?2) AND is_builtin = 0",
                [&theme.id, &theme.name],
            );

            let created_at = if theme.created_at.is_empty() { &now } else { &theme.created_at };
            tx.execute(
                "INSERT INTO themes (id, name, config, is_builtin, created_at)
                 VALUES (?1, ?2, ?3, 0, ?4)",
                (
                    &theme.id,
                    &theme.name,
                    &theme.config,
                    created_at,
                ),
            )
            .map_err(|e| format!("Failed to import theme {}: {}", theme.name, e))?;
        }

        // 6. Import App Settings (skipping sensitive keys)
        for setting in &data.app_settings {
            if is_sensitive_key(&setting.key) {
                continue;
            }
            if normalized_mode == "merge" {
                // In merge mode, preserve local settings
                tx.execute(
                    "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO NOTHING",
                    (&setting.key, &setting.value),
                )
                .map_err(|e| format!("Failed to import setting {}: {}", setting.key, e))?;
            } else {
                // In overwrite mode, update settings
                tx.execute(
                    "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                    (&setting.key, &setting.value),
                )
                .map_err(|e| format!("Failed to import setting {}: {}", setting.key, e))?;
            }
        }

        // 7. Import Compiler State
        if let Some(content) = &data.compiler_state {
            if !content.trim().is_empty() {
                if normalized_mode == "merge" {
                    let exists: bool = tx
                        .query_row("SELECT 1 FROM compiler_state WHERE id = 1", [], |_| Ok(true))
                        .unwrap_or(false);
                    if !exists {
                        tx.execute(
                            "INSERT INTO compiler_state (id, latex_content) VALUES (1, ?1)",
                            [content],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                } else {
                    tx.execute(
                        "INSERT INTO compiler_state (id, latex_content) VALUES (1, ?1)
                         ON CONFLICT(id) DO UPDATE SET latex_content=excluded.latex_content",
                        [content],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }

        // 8. Import Inbox Jobs
        for job in &data.inbox_jobs {
            let created_at = if job.created_at.is_empty() { &now } else { &job.created_at };
            tx.execute(
                "INSERT INTO inbox_jobs (id, url, raw_description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                    url=excluded.url,
                    raw_description=excluded.raw_description,
                    status=excluded.status",
                (
                    &job.id,
                    &job.url,
                    &job.raw_description,
                    &job.status,
                    created_at,
                ),
            )
            .map_err(|e| format!("Failed to import inbox job {}: {}", job.id, e))?;
        }

        // 9. Import Documents (metadata)
        for doc in &data.documents {
            let created_at = if doc.created_at.is_empty() { &now } else { &doc.created_at };
            let updated_at = if doc.updated_at.is_empty() { &now } else { &doc.updated_at };
            tx.execute(
                "INSERT INTO documents (id, title, description, tags, starred, main_file, last_compiled_at, compile_status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    description=excluded.description,
                    tags=excluded.tags,
                    starred=excluded.starred,
                    main_file=excluded.main_file,
                    last_compiled_at=excluded.last_compiled_at,
                    compile_status=excluded.compile_status,
                    updated_at=excluded.updated_at",
                rusqlite::params![
                    &doc.id,
                    &doc.title,
                    &doc.description,
                    &doc.tags,
                    doc.starred as i64,
                    &doc.main_file,
                    &doc.last_compiled_at,
                    &doc.compile_status,
                    created_at,
                    updated_at,
                ],
            )
            .map_err(|e| format!("Failed to import document {}: {}", doc.id, e))?;
        }

        // 10. Import Document Files
        for file in &data.document_files {
            if !is_text_extension(&file.rel_path) {
                continue;
            }
            if normalize_rel_path_check(&file.rel_path).is_err() {
                eprintln!(
                    "Skipping suspicious document file path during import: {}",
                    file.rel_path
                );
                continue;
            }
            let updated_at = if file.updated_at.is_empty() { &now } else { &file.updated_at };
            if normalized_mode == "merge" {
                tx.execute(
                    "INSERT INTO document_files (doc_id, rel_path, content, size_bytes, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(doc_id, rel_path) DO NOTHING",
                    rusqlite::params![
                        &file.doc_id,
                        &file.rel_path,
                        &file.content,
                        file.size_bytes,
                        updated_at,
                    ],
                )
                .map_err(|e| format!("Failed to import document file {}: {}", file.rel_path, e))?;
            } else {
                tx.execute(
                    "INSERT INTO document_files (doc_id, rel_path, content, size_bytes, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(doc_id, rel_path) DO UPDATE SET
                        content=excluded.content,
                        size_bytes=excluded.size_bytes,
                        updated_at=excluded.updated_at",
                    rusqlite::params![
                        &file.doc_id,
                        &file.rel_path,
                        &file.content,
                        file.size_bytes,
                        updated_at,
                    ],
                )
                .map_err(|e| format!("Failed to import document file {}: {}", file.rel_path, e))?;
            }
        }

        // 11. Import HR Templates
        for tmpl in &data.hr_templates {
            let created_at = if tmpl.created_at.is_empty() { &now } else { &tmpl.created_at };
            let updated_at = if tmpl.updated_at.is_empty() { &now } else { &tmpl.updated_at };
            tx.execute(
                "INSERT INTO hr_templates (id, name, category, content, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    name=excluded.name,
                    category=excluded.category,
                    content=excluded.content,
                    updated_at=excluded.updated_at",
                (
                    &tmpl.id,
                    &tmpl.name,
                    &tmpl.category,
                    &tmpl.content,
                    created_at,
                    updated_at,
                ),
            )
            .map_err(|e| format!("Failed to import HR template {}: {}", tmpl.id, e))?;
        }

        // 12. Import Outreach Leads
        for lead in &data.outreach_leads {
            let posts_json =
                serde_json::to_string(&lead.recent_posts).unwrap_or_else(|_| "[]".to_string());
            let created_at = if lead.created_at.is_empty() { &now } else { &lead.created_at };
            let updated_at = if lead.updated_at.is_empty() { &now } else { &lead.updated_at };
            tx.execute(
                "INSERT INTO outreach_leads (
                    id, person_name, profile_url, headline, raw_bio, recent_posts,
                    template_id, char_limit, tailored_message, status, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(id) DO UPDATE SET
                    person_name=excluded.person_name,
                    profile_url=excluded.profile_url,
                    headline=excluded.headline,
                    raw_bio=excluded.raw_bio,
                    recent_posts=excluded.recent_posts,
                    template_id=excluded.template_id,
                    char_limit=excluded.char_limit,
                    tailored_message=excluded.tailored_message,
                    status=excluded.status,
                    updated_at=excluded.updated_at",
                rusqlite::params![
                    &lead.id,
                    &lead.person_name,
                    &lead.profile_url,
                    &lead.headline,
                    &lead.raw_bio,
                    &posts_json,
                    &lead.template_id,
                    &lead.char_limit,
                    &lead.tailored_message,
                    &lead.status,
                    created_at,
                    updated_at,
                ],
            )
            .map_err(|e| format!("Failed to import outreach lead {}: {}", lead.id, e))?;
        }

        // 13. Import Error Audit Logs
        for log in &data.error_audit_logs {
            let created_at = if log.created_at.is_empty() { &now } else { &log.created_at };
            let _ = tx.execute(
                "INSERT INTO error_audit_logs (id, task_type, error_type, message, details, source, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO NOTHING",
                rusqlite::params![
                    &log.id,
                    &log.task_type,
                    &log.error_type,
                    &log.message,
                    &log.details,
                    &log.source,
                    created_at,
                ],
            );
        }

        // Restore sensitive settings that were snapshotted before the import.
        restore_sensitive_settings(&tx, &sensitive_snapshot);

        tx.commit().map_err(|e| format!("Transaction commit failed: {}", e))?;
        Ok(())
    })();

    // Always re-enable foreign keys after transaction completes or fails
    let _ = conn.execute("PRAGMA foreign_keys = ON;", []);

    import_res
}

pub fn import_data_core(
    state: &AppState,
    app: &AppHandle,
    data: AppDataExport,
    mode: String,
) -> Result<(), String> {
    let mut db_guard = state.db.lock().map_err(|e| format!("Mutex error: {}", e))?;
    let conn = db_guard.as_mut().ok_or("Database connection lost")?;

    import_data_to_conn(conn, &data, &mode)?;

    // After commit, recreate on-disk text files for imported documents. This is
    // best-effort: failures are logged but do not abort the import.
    if let Err(e) = restore_document_filesystem(app, &data.document_files) {
        eprintln!("Document filesystem restore partial failure: {}", e);
    }

    state.mark_dirty();
    Ok(())
}

#[tauri::command]
pub async fn import_data(
    app: AppHandle,
    state: State<'_, AppState>,
    data: serde_json::Value,
    mode: String,
) -> Result<(), String> {
    let parsed = parse_backup_value(data)?;
    import_data_core(&state, &app, parsed, mode)
}

/// Lightweight path validation used during import so a malicious backup can't
/// smuggle `..` or absolute paths into the file system.
fn normalize_rel_path_check(rel_path: &str) -> Result<String, String> {
    use std::path::Path;
    if rel_path.is_empty() {
        return Err("Empty path".to_string());
    }
    if rel_path.contains('\0') {
        return Err("Null byte in path".to_string());
    }
    let normalized = rel_path.replace('\\', "/");
    let p = Path::new(&normalized);
    if p.is_absolute() {
        return Err("Absolute path not allowed".to_string());
    }
    for component in p.components() {
        let s = component.as_os_str().to_string_lossy();
        if s == ".." {
            return Err("Path traversal detected".to_string());
        }
    }
    Ok(normalized)
}

/// Write the embedded text files for imported documents to disk, mirroring the
/// on-disk layout the app expects. Skips binary assets. Failures are logged.
fn restore_document_filesystem(
    app: &AppHandle,
    files: &[DocumentFileExport],
) -> Result<(), String> {
    let root: PathBuf = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?
        .join("documents");

    for file in files {
        if !is_text_extension(&file.rel_path) {
            continue;
        }
        let normalized = match normalize_rel_path_check(&file.rel_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "Skipping bad path during restore: {} ({})",
                    file.rel_path, e
                );
                continue;
            }
        };
        let target = root.join(&file.doc_id).join(&normalized);

        // Containment check: ensure target is inside the per-doc dir.
        let doc_root = root.join(&file.doc_id);
        if let Some(parent) = target.parent() {
            if !parent.starts_with(&doc_root) {
                eprintln!("Skipping out-of-tree path: {}", target.display());
                continue;
            }
            let _ = std::fs::create_dir_all(parent);
        }
        if std::str::from_utf8(file.content.as_bytes()).is_err() {
            eprintln!("Skipping non-UTF8 file during restore: {}", normalized);
            continue;
        }
        if let Err(e) = std::fs::write(&target, file.content.as_bytes()) {
            eprintln!(
                "Failed to restore {} to {}: {}",
                file.rel_path,
                target.display(),
                e
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn auto_local_backup(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Manager;

    let data = export_all_data_core(&state)?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;

    let docs_dir = app
        .path()
        .document_dir()
        .map_err(|_| "Could not find documents directory".to_string())?;
    let backup_dir = docs_dir.join("RoleTect-Backups");

    if !backup_dir.exists() {
        std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M");
    let file_path = backup_dir.join(format!("RoleTect_Backup_{}.json", timestamp));

    std::fs::write(&file_path, json).map_err(|e| format!("Failed to write local backup: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_sensitive_key tests ---

    #[test]
    fn exact_keys_are_sensitive() {
        assert!(is_sensitive_key("extension_secret"));
        assert!(is_sensitive_key("active_server_port"));
    }

    #[test]
    fn exact_keys_are_case_insensitive() {
        assert!(is_sensitive_key("Extension_Secret"));
        assert!(is_sensitive_key("Active_Server_Port"));
    }

    #[test]
    fn prefix_keys_are_sensitive() {
        assert!(is_sensitive_key("s3_bucket_name"));
        assert!(is_sensitive_key("s3_region"));
        assert!(is_sensitive_key("aws_access_key_id"));
        assert!(is_sensitive_key("aws_secret_access_key"));
        assert!(is_sensitive_key("cloud_backup_url"));
    }

    #[test]
    fn substring_keys_are_sensitive() {
        assert!(is_sensitive_key("some_api_key_for_thing"));
        assert!(is_sensitive_key("my_secret_value"));
        assert!(is_sensitive_key("auth_token"));
        assert!(is_sensitive_key("db_password"));
        assert!(is_sensitive_key("bedrock_credential"));
        assert!(is_sensitive_key("backup_bucket"));
    }

    #[test]
    fn non_sensitive_keys_pass_through() {
        assert!(!is_sensitive_key("active_theme"));
        assert!(!is_sensitive_key("latex_workspace"));
        assert!(!is_sensitive_key("last_opened_file"));
        assert!(!is_sensitive_key("font_family"));
        assert!(!is_sensitive_key("font_size"));
        assert!(!is_sensitive_key("font_weight"));
        assert!(!is_sensitive_key("font_style"));
        assert!(!is_sensitive_key("auto_compile"));
        assert!(!is_sensitive_key("diagram_workspace"));
        assert!(!is_sensitive_key("last_opened_diagram"));
        assert!(!is_sensitive_key("ai_provider"));
        assert!(!is_sensitive_key("ai_model"));
        assert!(!is_sensitive_key("openai_custom_model"));
        assert!(!is_sensitive_key("openai_custom_base_url"));
        assert!(!is_sensitive_key("anthropic_custom_model"));
        assert!(!is_sensitive_key("ollama_custom_base_url"));
    }

    // --- snapshot / restore integration tests ---

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn snapshot_captures_only_sensitive_keys() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO app_settings VALUES ('ai_provider', 'gemini')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('ai_model', 'gemini-2.5-pro')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('extension_secret', 'abc123')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('active_server_port', '1420')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('active_theme', 'dracula')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO app_settings VALUES ('font_size', '14')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('s3_bucket_name', 'my-bucket')",
            [],
        )
        .unwrap();

        let snapshot = snapshot_sensitive_settings(&conn);
        let keys: Vec<&str> = snapshot.iter().map(|(k, _)| k.as_str()).collect();

        assert!(!keys.contains(&"ai_provider"));
        assert!(!keys.contains(&"ai_model"));
        assert!(keys.contains(&"extension_secret"));
        assert!(keys.contains(&"active_server_port"));
        assert!(keys.contains(&"s3_bucket_name"));
        assert!(!keys.contains(&"active_theme"));
        assert!(!keys.contains(&"font_size"));
    }

    #[test]
    fn restore_brings_back_sensitive_keys_after_wipe() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO app_settings VALUES ('active_server_port', '1420')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('extension_secret', 'my-secret-123')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO app_settings VALUES ('active_theme', 'dracula')",
            [],
        )
        .unwrap();

        let snapshot = snapshot_sensitive_settings(&conn);

        // Wipe everything (simulating overwrite mode)
        conn.execute("DELETE FROM app_settings", []).unwrap();

        // Import some foreign settings
        conn.execute(
            "INSERT INTO app_settings VALUES ('active_theme', 'nord-dark')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO app_settings VALUES ('font_size', '16')", [])
            .unwrap();

        // Restore sensitive keys
        restore_sensitive_settings(&conn, &snapshot);

        // Sensitive keys restored
        let port: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_server_port'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(port, "1420");

        let secret: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'extension_secret'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(secret, "my-secret-123");

        // Non-sensitive keys from import are untouched
        let theme: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_theme'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(theme, "nord-dark");

        let font: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'font_size'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(font, "16");
    }

    #[test]
    fn import_skips_sensitive_keys_from_incoming_data() {
        // Verify the guard: even if a backup file somehow contains sensitive keys,
        // they should be skipped during import.
        let incoming = vec![
            SettingExport {
                key: "active_theme".to_string(),
                value: "monokai".to_string(),
            },
            SettingExport {
                key: "ai_provider".to_string(),
                value: "openai".to_string(),
            },
            SettingExport {
                key: "extension_secret".to_string(),
                value: "LEAKED".to_string(),
            },
            SettingExport {
                key: "s3_bucket_name".to_string(),
                value: "evil-bucket".to_string(),
            },
            SettingExport {
                key: "font_family".to_string(),
                value: "Inter".to_string(),
            },
        ];

        let safe: Vec<&SettingExport> = incoming
            .iter()
            .filter(|s| !is_sensitive_key(&s.key))
            .collect();

        // active_theme, ai_provider, font_family are allowed; extension_secret and s3_bucket_name are skipped
        assert_eq!(safe.len(), 3);
        assert_eq!(safe[0].key, "active_theme");
        assert_eq!(safe[1].key, "ai_provider");
        assert_eq!(safe[2].key, "font_family");
    }

    // --- Documents backup recursive-nested-path tests ---

    #[test]
    fn nested_document_paths_are_backup_eligible() {
        // Text files nested inside subdirectories must be eligible for backup
        // (recursive children must roundtrip through local + S3 backups).
        assert!(is_text_extension("chapters/intro.tex"));
        assert!(is_text_extension("sections/related/notes.bib"));
        assert!(is_text_extension("a/b/c/preamble.sty"));
        assert!(is_text_extension("figures/_folder.keep.txt"));
    }

    #[test]
    fn nested_document_paths_can_be_validated() {
        // The defensive validator used during import must accept nested paths and
        // reject traversal attempts, regardless of extension.
        assert!(normalize_rel_path_check("chapters/intro.tex").is_ok());
        assert!(normalize_rel_path_check("a/b/c/d/notes.md").is_ok());

        // Reject traversal.
        assert!(normalize_rel_path_check("../etc/passwd").is_err());
        assert!(normalize_rel_path_check("a/../../escape.tex").is_err());
        assert!(normalize_rel_path_check("/absolute/path.tex").is_err());
        assert!(normalize_rel_path_check("").is_err());
    }

    // --- Bulletproof Backup Schema & JSON Parser Tests ---

    #[test]
    fn parse_backup_json_with_bom() {
        let json_with_bom = "\u{feff}{\"schema_version\": 1, \"jobs\": [], \"exported_at\": \"2026-09-08T00:00:00Z\"}";
        let parsed = parse_backup_json(json_with_bom);
        assert!(parsed.is_ok(), "BOM JSON should parse without error: {:?}", parsed.err());
        let data = parsed.unwrap();
        assert_eq!(data.schema_version, Some(1));
    }

    #[test]
    fn parse_backup_json_wrapped_payload() {
        let wrapped_json = r#"{
            "data": {
                "schema_version": 1,
                "jobs": [
                    {
                        "id": "job-1",
                        "company_name": "Acme",
                        "job_title": "Engineer",
                        "work_model": "Remote",
                        "employment_type": "Full-time",
                        "status": "Drafting",
                        "raw_jd": "Job description here"
                    }
                ],
                "exported_at": "2026-09-08T00:00:00Z"
            }
        }"#;
        let parsed = parse_backup_json(wrapped_json);
        assert!(parsed.is_ok(), "Wrapped JSON should unwrap successfully: {:?}", parsed.err());
        let data = parsed.unwrap();
        assert_eq!(data.jobs.len(), 1);
        assert_eq!(data.jobs[0].company_name, "Acme");
    }

    #[test]
    fn parse_backup_json_rejects_unrelated_json() {
        let unrelated = r#"{"name": "my-node-app", "version": "1.0.0", "dependencies": {}}"#;
        let parsed = parse_backup_json(unrelated);
        assert!(parsed.is_err());
        assert!(parsed.unwrap_err().contains("Not a recognized RoleTect backup structure"));
    }

    #[test]
    fn parse_backup_json_backwards_compatibility_defaults() {
        // Minimal backup from legacy format that omits newer tables like document_files or compiler_state
        let legacy = r#"{
            "jobs": [],
            "base_resumes": [
                {
                    "id": "res-1",
                    "name": "Software Resume",
                    "latex_content": "\\documentclass{article}"
                }
            ],
            "exported_at": "2024-01-01T00:00:00Z"
        }"#;
        let parsed = parse_backup_json(legacy);
        assert!(parsed.is_ok(), "Legacy format should deserialize with defaults: {:?}", parsed.err());
        let data = parsed.unwrap();
        assert_eq!(data.base_resumes.len(), 1);
        assert_eq!(data.base_resumes[0].category, ""); // default empty string
        assert!(data.document_files.is_empty());
        assert!(data.documents.is_empty());
        assert!(data.compiler_state.is_none());
    }

    fn setup_full_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE app_settings (
                key TEXT PRIMARY KEY, 
                value TEXT NOT NULL
            );
            CREATE TABLE base_resumes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                latex_content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE base_cover_letters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                latex_content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE jobs (
                id TEXT PRIMARY KEY,
                company_name TEXT NOT NULL,
                job_title TEXT NOT NULL,
                work_model TEXT DEFAULT 'Remote',
                employment_type TEXT DEFAULT 'Full-time',
                status TEXT NOT NULL DEFAULT 'Drafting',
                raw_jd TEXT NOT NULL,
                requirements TEXT,
                core_responsibilities TEXT,
                custom_instruction TEXT,
                reference_name TEXT,
                reference_email TEXT,
                social_link TEXT,
                job_url TEXT,
                base_resume_id TEXT,
                base_cl_id TEXT,
                salary TEXT,
                applied_date TEXT,
                interview_date TEXT,
                offer_date TEXT,
                rejected_date TEXT,
                joining_date TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (base_resume_id) REFERENCES base_resumes(id),
                FOREIGN KEY (base_cl_id) REFERENCES base_cover_letters(id)
            );
            CREATE TABLE tailored_resumes (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                base_resume_id TEXT NOT NULL,
                final_latex_content TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (job_id) REFERENCES jobs(id),
                FOREIGN KEY (base_resume_id) REFERENCES base_resumes(id)
            );
            CREATE TABLE tailored_cover_letters (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL,
                base_cl_id TEXT NOT NULL,
                final_latex_content TEXT NOT NULL,
                is_active BOOLEAN DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (job_id) REFERENCES jobs(id),
                FOREIGN KEY (base_cl_id) REFERENCES base_cover_letters(id)
            );
            CREATE TABLE compiler_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                latex_content TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE downloads (
                id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                download_type TEXT NOT NULL,
                job_id TEXT,
                content_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (job_id) REFERENCES jobs(id)
            );
            CREATE TABLE themes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                config TEXT NOT NULL,
                is_builtin BOOLEAN DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE inbox_jobs (
                id TEXT PRIMARY KEY,
                url TEXT,
                raw_description TEXT NOT NULL,
                status TEXT DEFAULT 'Pending',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                starred INTEGER NOT NULL DEFAULT 0,
                main_file TEXT,
                last_compiled_at TEXT,
                compile_status TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE document_files (
                doc_id TEXT NOT NULL,
                rel_path TEXT NOT NULL,
                content TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (doc_id, rel_path),
                FOREIGN KEY (doc_id) REFERENCES documents(id) ON DELETE CASCADE
            );
            CREATE TABLE hr_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'Outreach',
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE outreach_leads (
                id TEXT PRIMARY KEY,
                person_name TEXT NOT NULL,
                profile_url TEXT NOT NULL,
                headline TEXT DEFAULT '',
                raw_bio TEXT NOT NULL,
                recent_posts TEXT DEFAULT '[]',
                template_id TEXT,
                char_limit INTEGER NOT NULL DEFAULT 250,
                tailored_message TEXT DEFAULT '',
                status TEXT NOT NULL DEFAULT 'Draft',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (template_id) REFERENCES hr_templates(id) ON DELETE SET NULL
            );
            CREATE TABLE error_audit_logs (
                id TEXT PRIMARY KEY,
                task_type TEXT NOT NULL,
                error_type TEXT NOT NULL,
                message TEXT NOT NULL,
                details TEXT,
                source TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )
        .unwrap();
        conn
    }

    #[test]
    fn import_data_to_conn_merge_preserves_local_settings() {
        let mut conn = setup_full_test_db();
        // Insert local existing setting
        conn.execute("INSERT INTO app_settings VALUES ('active_theme', 'local-dark')", []).unwrap();
        conn.execute("INSERT INTO app_settings VALUES ('extension_secret', 'my-secret')", []).unwrap();

        let incoming_data = AppDataExport {
            schema_version: Some(1),
            jobs: vec![],
            base_resumes: vec![],
            base_cover_letters: vec![],
            tailored_resumes: vec![],
            tailored_cover_letters: vec![],
            downloads: vec![],
            themes: vec![],
            app_settings: vec![
                SettingExport { key: "active_theme".to_string(), value: "incoming-light".to_string() },
                SettingExport { key: "new_setting".to_string(), value: "new_val".to_string() },
            ],
            inbox_jobs: vec![],
            compiler_state: None,
            documents: vec![],
            document_files: vec![],
            hr_templates: vec![],
            outreach_leads: vec![],
            error_audit_logs: vec![],
            exported_at: "2026-09-08T00:00:00Z".to_string(),
        };

        let res = import_data_to_conn(&mut conn, &incoming_data, "merge");
        assert!(res.is_ok(), "Merge failed: {:?}", res.err());

        // Local active_theme must be preserved (not overwritten by incoming)
        let theme: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'active_theme'", [], |r| r.get(0)).unwrap();
        assert_eq!(theme, "local-dark");

        // New setting was added
        let new_val: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'new_setting'", [], |r| r.get(0)).unwrap();
        assert_eq!(new_val, "new_val");

        // Sensitive setting remains untouched
        let secret: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'extension_secret'", [], |r| r.get(0)).unwrap();
        assert_eq!(secret, "my-secret");
    }

    #[test]
    fn import_data_to_conn_overwrite_replaces_vault_and_preserves_sensitive() {
        let mut conn = setup_full_test_db();
        conn.execute("INSERT INTO app_settings VALUES ('active_theme', 'old-theme')", []).unwrap();
        conn.execute("INSERT INTO app_settings VALUES ('aws_secret_access_key', 'super-secret')", []).unwrap();
        conn.execute(
            "INSERT INTO base_resumes VALUES ('res-old', 'Old Resume', 'SWE', 'content', '2025-01-01', '2025-01-01')",
            [],
        ).unwrap();

        let incoming_data = AppDataExport {
            schema_version: Some(1),
            jobs: vec![],
            base_resumes: vec![
                ResumeDetail {
                    id: "res-new".to_string(),
                    name: "New Resume".to_string(),
                    category: "AI".to_string(),
                    latex_content: "new latex".to_string(),
                    created_at: "2026-01-01".to_string(),
                    updated_at: "2026-01-01".to_string(),
                }
            ],
            base_cover_letters: vec![],
            tailored_resumes: vec![],
            tailored_cover_letters: vec![],
            downloads: vec![],
            themes: vec![],
            app_settings: vec![
                SettingExport { key: "active_theme".to_string(), value: "new-theme".to_string() },
            ],
            inbox_jobs: vec![],
            compiler_state: None,
            documents: vec![],
            document_files: vec![],
            hr_templates: vec![],
            outreach_leads: vec![],
            error_audit_logs: vec![],
            exported_at: "2026-09-08T00:00:00Z".to_string(),
        };

        let res = import_data_to_conn(&mut conn, &incoming_data, "overwrite");
        assert!(res.is_ok(), "Overwrite failed: {:?}", res.err());

        // Old resume deleted, new resume present
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM base_resumes WHERE id = 'res-old'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
        let new_count: i64 = conn.query_row("SELECT COUNT(*) FROM base_resumes WHERE id = 'res-new'", [], |r| r.get(0)).unwrap();
        assert_eq!(new_count, 1);

        // Setting updated
        let theme: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'active_theme'", [], |r| r.get(0)).unwrap();
        assert_eq!(theme, "new-theme");

        // Sensitive setting survived the overwrite!
        let secret: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'aws_secret_access_key'", [], |r| r.get(0)).unwrap();
        assert_eq!(secret, "super-secret");
    }

    #[test]
    fn import_data_to_conn_resolves_theme_conflicts() {
        let mut conn = setup_full_test_db();
        // Insert existing custom theme with id: "t-1", name: "Cyberpunk"
        conn.execute(
            "INSERT INTO themes (id, name, config, is_builtin, created_at) VALUES ('t-1', 'Cyberpunk', '{\"bg\":\"#000\"}', 0, '2025-01-01')",
            [],
        ).unwrap();

        // Incoming theme has DIFFERENT id: "t-99", but SAME name: "Cyberpunk"
        let incoming_data = AppDataExport {
            schema_version: Some(1),
            jobs: vec![],
            base_resumes: vec![],
            base_cover_letters: vec![],
            tailored_resumes: vec![],
            tailored_cover_letters: vec![],
            downloads: vec![],
            themes: vec![
                ThemeExport {
                    id: "t-99".to_string(),
                    name: "Cyberpunk".to_string(),
                    config: "{\"bg\":\"#111\"}".to_string(),
                    is_builtin: false,
                    created_at: "2026-09-08T00:00:00Z".to_string(),
                }
            ],
            app_settings: vec![],
            inbox_jobs: vec![],
            compiler_state: None,
            documents: vec![],
            document_files: vec![],
            hr_templates: vec![],
            outreach_leads: vec![],
            error_audit_logs: vec![],
            exported_at: "2026-09-08T00:00:00Z".to_string(),
        };

        // In merge mode, this should NOT throw UNIQUE constraint failed: themes.name
        let res = import_data_to_conn(&mut conn, &incoming_data, "merge");
        assert!(res.is_ok(), "Theme conflict resolution failed: {:?}", res.err());

        // The theme was safely replaced with the incoming definition
        let config: String = conn.query_row("SELECT config FROM themes WHERE name = 'Cyberpunk'", [], |r| r.get(0)).unwrap();
        assert_eq!(config, "{\"bg\":\"#111\"}");
    }

    #[test]
    fn import_data_to_conn_rejects_invalid_mode() {
        let mut conn = setup_full_test_db();
        let data = AppDataExport {
            schema_version: Some(1),
            jobs: vec![],
            base_resumes: vec![],
            base_cover_letters: vec![],
            tailored_resumes: vec![],
            tailored_cover_letters: vec![],
            downloads: vec![],
            themes: vec![],
            app_settings: vec![],
            inbox_jobs: vec![],
            compiler_state: None,
            documents: vec![],
            document_files: vec![],
            hr_templates: vec![],
            outreach_leads: vec![],
            error_audit_logs: vec![],
            exported_at: "2026-09-08T00:00:00Z".to_string(),
        };

        let res = import_data_to_conn(&mut conn, &data, "drop_table");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid restore mode"));
    }

    #[test]
    fn import_data_to_conn_enforces_foreign_keys_after_completion() {
        let mut conn = setup_full_test_db();
        let data = AppDataExport {
            schema_version: Some(1),
            jobs: vec![],
            base_resumes: vec![],
            base_cover_letters: vec![],
            tailored_resumes: vec![],
            tailored_cover_letters: vec![],
            downloads: vec![],
            themes: vec![],
            app_settings: vec![],
            inbox_jobs: vec![],
            compiler_state: None,
            documents: vec![],
            document_files: vec![],
            hr_templates: vec![],
            outreach_leads: vec![],
            error_audit_logs: vec![],
            exported_at: "2026-09-08T00:00:00Z".to_string(),
        };

        let _ = import_data_to_conn(&mut conn, &data, "merge");

        // Verify foreign_keys pragma is ON
        let fk_enabled: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        assert_eq!(fk_enabled, 1, "foreign_keys PRAGMA must be 1 (ON) after import completes");
    }
}
