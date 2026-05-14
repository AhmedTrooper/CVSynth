use rusqlite::{Connection, Result};
use std::fs;
use tauri::{AppHandle, Manager};

pub fn init_db(app: &AppHandle) -> Result<Connection> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    let db_path = app_dir.join("cvsynth.db");

    let conn = Connection::open(db_path)?; 

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute_batch(
        "
        -- 1. App Settings Table
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY, 
            value TEXT NOT NULL
        );

        -- 2. Base Resumes Table
        CREATE TABLE IF NOT EXISTS base_resumes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT NOT NULL,
            latex_content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TRIGGER IF NOT EXISTS update_base_resumes_modtime 
            AFTER UPDATE ON base_resumes 
            BEGIN UPDATE base_resumes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 3. Jobs Table
        CREATE TABLE IF NOT EXISTS jobs (
            id TEXT PRIMARY KEY,
            company_name TEXT NOT NULL,
            job_title TEXT NOT NULL,
            work_model TEXT DEFAULT 'Remote',
            employment_type TEXT DEFAULT 'Full-time',
            status TEXT CHECK(status IN ('Drafting', 'Applied', 'Interviewing', 'Offer', 'Rejected')) DEFAULT 'Drafting',
            raw_jd TEXT NOT NULL,
            custom_instruction TEXT,
            reference_name TEXT,
            reference_email TEXT,
            social_link TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TRIGGER IF NOT EXISTS update_jobs_modtime 
            AFTER UPDATE ON jobs 
            BEGIN UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;

        -- 4. Tailored Resumes Table (Generated Output)
        CREATE TABLE IF NOT EXISTS tailored_resumes (
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
        CREATE TRIGGER IF NOT EXISTS update_tailored_resumes_modtime 
            AFTER UPDATE ON tailored_resumes 
            BEGIN UPDATE tailored_resumes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;
        "
    )?;

    // --- MIGRATIONS ---

    // 1. Check if we need to remove CHECK constraints from 'jobs' (for flexibility with Temporary/Internship/etc.)
    let table_sql: String = conn.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='jobs'",
        [],
        |row| row.get(0),
    ).unwrap_or_default();

    if table_sql.contains("CHECK(employment_type IN") || table_sql.contains("CHECK(work_model IN") {
        // Perform migration to flexible schema
        println!("Migrating 'jobs' table to flexible schema...");
        conn.execute_batch(
            "
            PRAGMA foreign_keys=OFF;
            BEGIN TRANSACTION;
            
            ALTER TABLE jobs RENAME TO jobs_old;
            
            CREATE TABLE jobs (
                id TEXT PRIMARY KEY,
                company_name TEXT NOT NULL,
                job_title TEXT NOT NULL,
                work_model TEXT DEFAULT 'Remote',
                employment_type TEXT DEFAULT 'Full-time',
                status TEXT CHECK(status IN ('Drafting', 'Applied', 'Interviewing', 'Offer', 'Rejected')) DEFAULT 'Drafting',
                raw_jd TEXT NOT NULL,
                custom_instruction TEXT,
                reference_name TEXT,
                reference_email TEXT,
                social_link TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            INSERT INTO jobs (
                id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, custom_instruction, reference_name, 
                reference_email, social_link, created_at, updated_at
            ) 
            SELECT 
                id, company_name, job_title, work_model, employment_type, 
                status, raw_jd, custom_instruction, reference_name, 
                reference_email, social_link, created_at, updated_at 
            FROM jobs_old;
            
            DROP TABLE jobs_old;
            
            -- Re-create the trigger for the new table
            DROP TRIGGER IF EXISTS update_jobs_modtime;
            CREATE TRIGGER update_jobs_modtime 
                AFTER UPDATE ON jobs 
                BEGIN UPDATE jobs SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id; END;
                
            COMMIT;
            PRAGMA foreign_keys=ON;
            "
        ).expect("Failed to migrate jobs table");
    }

    // 2. Add missing columns to 'jobs' table (redundant but safe after the recreation above)
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(jobs)")?
        .query_map([], |row| row.get(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.contains(&"reference_name".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN reference_name TEXT", [])?;
    }
    if !columns.contains(&"reference_email".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN reference_email TEXT", [])?;
    }
    if !columns.contains(&"social_link".to_string()) {
        conn.execute("ALTER TABLE jobs ADD COLUMN social_link TEXT", [])?;
    }

    Ok(conn)
}
