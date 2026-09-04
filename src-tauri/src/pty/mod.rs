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
    // `cmd` is only mutated in the Windows-only branch below
    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut cmd = CommandBuilder::new(&shell);

    // PowerShell: disable the banner and use a clean prompt
    #[cfg(target_os = "windows")]
    if shell.to_lowercase().contains("powershell") {
        cmd.args(["-NoLogo", "-NoExit"]);
    }

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

/// Whether the terminal looks like it is reading a secret.
///
/// A password prompt (sudo, ssh, getpass, `read -s`) clears ECHO while leaving
/// canonical mode on. An interactive shell also runs with ECHO clear — readline
/// echoes characters itself — but turns ICANON off to do so. So "ICANON set and
/// ECHO clear" is what distinguishes a password prompt from an ordinary prompt.
///
/// The on-screen keyboard uses this to stop lighting up keys while a password
/// is typed, which would otherwise display it to anyone watching the screen.
///
/// On Linux, TCGETS against the master returns the slave's termios, so this
/// reads the state the foreground program actually set.
#[tauri::command]
pub fn pty_secure_input(
    state: tauri::State<'_, PtyState>,
    session_id: String,
) -> Result<bool, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&session_id).ok_or("session not found")?;

    #[cfg(unix)]
    {
        match session.master.as_raw_fd() {
            Some(fd) => Ok(secure_input_fd(fd)),
            None => Ok(false),
        }
    }

    #[cfg(not(unix))]
    {
        let _ = session;
        Ok(false)
    }
}

/// True when the pty is in canonical mode with echo disabled.
///
/// Fails closed: an unreadable state reports "secret", so the caller suppresses
/// display rather than risking showing a password.
#[cfg(unix)]
fn secure_input_fd(fd: std::os::unix::io::RawFd) -> bool {
    // SAFETY: the caller holds the live master pty that owns this fd.
    unsafe {
        let mut termios: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut termios) != 0 {
            return true;
        }
        let echo = termios.c_lflag & libc::ECHO != 0;
        let canonical = termios.c_lflag & libc::ICANON != 0;
        canonical && !echo
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use portable_pty::PtySize;
    use std::time::{Duration, Instant};

    struct Fixture {
        _child: Box<dyn portable_pty::Child + Send + Sync>,
        writer: Box<dyn Write + Send>,
        fd: std::os::unix::io::RawFd,
        _master: Box<dyn portable_pty::MasterPty + Send>,
    }

    fn shell() -> Fixture {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .expect("openpty");
        let mut cmd = CommandBuilder::new("/bin/sh");
        cmd.env("PS1", "");
        let child = pair.slave.spawn_command(cmd).expect("spawn");
        let fd = pair.master.as_raw_fd().expect("master fd");

        // Drain output so the shell never blocks on a full buffer.
        let mut reader = pair.master.try_clone_reader().expect("reader");
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 { break; }
            }
        });

        let writer = pair.master.take_writer().expect("writer");
        Fixture { _child: child, writer, fd, _master: pair.master }
    }

    fn wait_for(cond: impl Fn() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if cond() { return true; }
            std::thread::sleep(Duration::from_millis(25));
        }
        false
    }

    /// Echo off with canonical mode on is what a password prompt looks like.
    #[test]
    fn password_prompt_is_detected() {
        let mut f = shell();
        assert!(!secure_input_fd(f.fd), "an ordinary prompt is not secret");

        f.writer.write_all(b"stty -echo\n").unwrap();
        f.writer.flush().ok();
        assert!(wait_for(|| secure_input_fd(f.fd)), "echo off + icanon must read as secret");

        f.writer.write_all(b"stty echo\n").unwrap();
        f.writer.flush().ok();
        assert!(wait_for(|| !secure_input_fd(f.fd)), "restoring echo must clear it");
    }

    /// Regression: an interactive shell's readline also clears ECHO, but turns
    /// ICANON off to do its own echoing. Treating that as a password prompt
    /// suppressed the keyboard permanently at an ordinary prompt.
    #[test]
    fn readline_raw_mode_is_not_secret() {
        let mut f = shell();
        f.writer.write_all(b"stty -icanon -echo\n").unwrap();
        f.writer.flush().ok();

        // Give the setting time to land, then confirm it never reads as secret.
        std::thread::sleep(Duration::from_millis(300));
        assert!(
            !secure_input_fd(f.fd),
            "raw mode (readline) must not be mistaken for a password prompt"
        );
    }
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
    {
        // Prefer PowerShell Core (pwsh), fall back to Windows PowerShell
        if std::path::Path::new("C:\\Program Files\\PowerShell\\7\\pwsh.exe").exists() {
            return "C:\\Program Files\\PowerShell\\7\\pwsh.exe".to_string();
        }
        "powershell.exe".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}
