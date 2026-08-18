mod commands;
pub mod contracts;
mod ports;
mod preferences;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = commands::AppState::load().expect("load Vela Workbench preferences");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::select_repository,
            commands::inspect_repository,
            commands::select_vela_binary,
            commands::clear_recents,
            commands::launch_repository,
            commands::preview_worktree,
            commands::create_worktree,
            commands::select_native_tool,
            commands::preview_native_exec,
            commands::run_native_exec,
            commands::cancel_native_exec,
            commands::select_evidence_file,
            commands::preview_evidence_export,
            commands::export_evidence,
            commands::preview_submission_draft,
            commands::submit_submission_draft,
            commands::select_submission_import,
            commands::import_submission,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
