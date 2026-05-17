use tauri::command;
use std::path::PathBuf;
use crate::ai;

#[command]
pub async fn refine_latex_with_ai(
    provider: String,
    model: String,
    api_key: String,
    current_latex: String,
    instruction: String,
) -> Result<String, String> {
    ai::refine_tailored_resume(&provider, &model, &api_key, &current_latex, &instruction).await
}

#[command]
pub async fn fix_latex_with_ai(
    provider: String,
    model: String,
    api_key: String,
    broken_latex: String,
    error_logs: String,
) -> Result<String, String> {
    ai::fix_latex_errors(&provider, &model, &api_key, &broken_latex, &error_logs).await
}

#[command]
pub async fn compile_resume_to_pdf(latex_code: String) -> Result<Vec<u8>, String> {
    // TeX engines are notoriously stack-heavy. Segfaults are frequently caused by
    // stack overflows in threads with default sizes. We use a dedicated thread
    // with a 10MB stack size to ensure the heavy TeX logic has enough room to run safely.
    tokio::task::spawn_blocking(move || {
        let thread_handle = std::thread::Builder::new()
            .name("tectonic-compiler".into())
            .stack_size(10 * 1024 * 1024) // 10MB
            .spawn(move || {
                tectonic::latex_to_pdf(latex_code)
                    .map_err(|e| format!("Tectonic compilation error: {}", e))
            })
            .map_err(|e| format!("Failed to spawn compiler thread: {}", e))?;

        thread_handle
            .join()
            .map_err(|_| "Compiler thread panicked or exited unexpectedly".to_string())?
    })
    .await
    .map_err(|e| format!("Blocking task failed: {}", e))?
}

#[command]
pub async fn compile_workspace_to_pdf(workspace_dir: String, main_file_name: String) -> Result<Vec<u8>, String> {
    let workspace_path = PathBuf::from(&workspace_dir);
    
    // 1. Pre-flight checks on the main thread to avoid spawning a heavy thread for obvious errors
    if !workspace_path.is_dir() {
        return Err(format!("Workspace path '{}' is not a valid directory.", workspace_dir));
    }

    let main_file_path = workspace_path.join(&main_file_name);
    if !main_file_path.is_file() {
        return Err(format!("Main TeX file '{}' not found in workspace.", main_file_name));
    }

    tokio::task::spawn_blocking(move || {
        let thread_handle = std::thread::Builder::new()
            .name("tectonic-workspace-compiler".into())
            .stack_size(10 * 1024 * 1024)
            .spawn(move || {
                // Re-instantiate PathBuf inside the thread
                let workspace_path = PathBuf::from(&workspace_dir);
                
                // We use Tectonic's ProcessingSession for full filesystem access
                let mut status = tectonic::status::NoopStatusBackend::default();
                
                let config_loader = tectonic::config::PersistentConfig::default();
                let bundle = config_loader
                    .default_bundle(false)
                    .map_err(|e| format!("Failed to load Tectonic bundle: {}", e))?;

                let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
                sb.bundle(bundle)
                    .primary_input_path(&workspace_path.join(&main_file_name))
                    .filesystem_root(&workspace_path)
                    .format_name("latex")
                    .output_format(tectonic::driver::OutputFormat::Pdf)
                    .build_date(std::time::SystemTime::now());

                let mut sess = sb.create(&mut status)
                    .map_err(|e| format!("Failed to create Tectonic processing session: {}", e))?;

                sess.run(&mut status)
                    .map_err(|e| format!("Tectonic compilation failed: {}", e))?;

                // Find the output PDF in the workspace
                let main_path = PathBuf::from(&main_file_name);
                let pdf_file_name = main_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| format!("{}.pdf", s))
                    .ok_or("Failed to determine generated PDF name from input file stem")?;
                
                let pdf_path = workspace_path.join(&pdf_file_name);
                
                if pdf_path.exists() {
                    std::fs::read(&pdf_path).map_err(|e| format!("Failed to read generated PDF from disk: {}", e))
                } else {
                    Err(format!("Compilation appeared successful, but the PDF file '{}' was not found in the workspace.", pdf_file_name))
                }
            })
            .map_err(|e| format!("Failed to spawn compiler thread: {}", e))?;

        thread_handle
            .join()
            .map_err(|_| "The compiler thread panicked. This usually indicates a fatal error in the TeX engine.".to_string())?
    })
    .await
    .map_err(|e| format!("The asynchronous task failed: {}", e))?
}
