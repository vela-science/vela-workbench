mod commands;
pub mod contracts;
mod ports;
mod preferences;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = commands::AppState::load().expect("load Vela Workbench preferences");
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::review_problem_handoff,
            commands::open_problem_handoff,
            commands::review_problem_handoff_source,
            commands::review_problem_handoff_authority,
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
            commands::refresh_decision_inbox,
            commands::select_verification_method,
            commands::preview_verification_record,
            commands::record_verification,
            commands::select_verification_import,
            commands::import_verification,
            commands::preview_decision,
            commands::execute_decision,
            commands::preview_recovery,
            commands::recover_transaction,
            commands::select_opengauss,
            commands::launch_opengauss_handoff,
            commands::refresh_opengauss_handoff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
