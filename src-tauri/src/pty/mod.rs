use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

pub struct PtySession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
}

#[derive(Default)]
pub struct PtyState {
    sessions: Mutex<HashMap<String, PtySession>>,
}

impl PtyState {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct PtyOutputPayload {
    session_id: String,
    data: String,
}

#[tauri::command]
pub fn pty_spawn(
    app: AppHandle,
    state: tauri::State<'_, PtyState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;

    let shell = default_shell();
    let cmd = CommandBuilder::new(shell);
    let _child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    state.sessions.lock().map_err(|e| e.to_string())?
        .insert(session_id.clone(), PtySession { writer, master: pair.master });

    let app_handle = app.clone();
    let emit_session_id = session_id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_handle.emit("pty_output", PtyOutputPayload {
                        session_id: emit_session_id.clone(),
                        data,
                    });
                }
                Err(_) => break,
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn pty_write(
    state: tauri::State<'_, PtyState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get_mut(&session_id).ok_or("session not found")?;
    session.writer.write_all(data.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pty_resize(
    state: tauri::State<'_, PtyState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&session_id).ok_or("session not found")?;
    session.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pty_close(
    state: tauri::State<'_, PtyState>,
    session_id: String,
) -> Result<(), String> {
    state.sessions.lock().map_err(|e| e.to_string())?.remove(&session_id);
    Ok(())
}

fn default_shell() -> String {
    #[cfg(target_os = "windows")]
    { std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string()) }
    #[cfg(not(target_os = "windows"))]
    { std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()) }
}
