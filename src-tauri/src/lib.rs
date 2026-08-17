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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
