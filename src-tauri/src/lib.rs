mod config;
mod fs;
mod pty;
mod stats;

use tauri::Manager;

use pty::PtyState;
use stats::StatsState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PtyState::new())
        .manage(StatsState::default())
        .invoke_handler(tauri::generate_handler![
            pty::pty_spawn,
            pty::pty_write,
            pty::pty_resize,
            pty::pty_close,
            pty::pty_secure_input,
            stats::get_system_stats,
            stats::kill_process,
            stats::set_process_stream,
            config::get_config,
            config::save_config,
            fs::fs_list_dir,
            fs::fs_home_dir,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let wanted = app.state::<StatsState>().processes_wanted.clone();
            stats::start_stats_emitter(handle, 1000, wanted);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
