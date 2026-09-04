use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

pub struct PtySession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    /// Pid of the shell itself. When it is the foreground process group, the
    /// user is at a prompt rather than inside a program.
    shell_pid: Option<u32>,
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

    let child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
    let shell_pid = child.process_id();

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    state.sessions.lock().map_err(|e| e.to_string())?
        .insert(session_id.clone(), PtySession { writer, master: pair.master, shell_pid });

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

/// What the terminal is currently doing with typed input.
#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct TerminalInputState {
    /// Terminal is echoing typed characters itself.
    pub echo: bool,
    /// Canonical (line) mode — off for anything reading key by key.
    pub canonical: bool,
    /// The shell is the foreground process, i.e. the user is at a prompt.
    pub at_shell_prompt: bool,
    /// Name of the foreground program, when it can be determined.
    pub foreground: Option<String>,
    /// Best judgement that a secret is being typed.
    pub secret: bool,
}

/// Programs that read secrets. Needed because termios alone cannot always tell:
/// with sudo's `pwfeedback` enabled (the default on Linux Mint) sudo reads key
/// by key to print asterisks, leaving ICANON off and ECHO off — exactly the
/// state an ordinary shell prompt sits in, since readline does the same.
#[cfg(unix)]
const SECRET_PROGRAMS: &[&str] = &[
    "sudo", "sudoedit", "su", "doas", "ssh", "sshpass", "scp", "sftp", "ssh-add",
    "passwd", "chpasswd", "gpg", "gpg2", "openssl", "cryptsetup", "veracrypt",
    "smbclient", "mysql", "psql", "mongosh", "keyctl", "systemd-ask-password",
];

#[cfg(unix)]
fn is_secret_program(name: &str) -> bool {
    SECRET_PROGRAMS.contains(&name) || name.starts_with("pinentry")
}

/// Name of the process group currently owning the terminal.
#[cfg(unix)]
fn foreground_name(fd: std::os::unix::io::RawFd) -> (Option<i32>, Option<String>) {
    // SAFETY: fd is a live pty master owned by the caller.
    let pgrp = unsafe { libc::tcgetpgrp(fd) };
    if pgrp <= 0 {
        return (None, None);
    }
    let comm = std::fs::read_to_string(format!("/proc/{pgrp}/comm"))
        .ok()
        .map(|s| s.trim().to_string());
    (Some(pgrp), comm)
}

/// Inspect the terminal to decide whether typed input should be displayed.
///
/// Treated as secret when echo is off and either:
///   - canonical mode is on, which is what getpass and `read -s` leave; or
///   - a secret-reading program owns the terminal, which covers sudo with
///     pwfeedback, where termios looks exactly like a shell prompt.
///
/// Deliberately *not* secret when the shell itself is in the foreground: an
/// interactive shell always runs with echo off because readline echoes
/// characters itself. Programs like vim or less also run with echo off, but
/// they are not in the list, so the keyboard keeps working inside them.
#[tauri::command]
pub fn pty_input_state(
    state: tauri::State<'_, PtyState>,
    session_id: String,
) -> Result<TerminalInputState, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions.get(&session_id).ok_or("session not found")?;

    #[cfg(unix)]
    {
        match session.master.as_raw_fd() {
            Some(fd) => Ok(input_state_fd(fd, session.shell_pid)),
            // Cannot inspect: assume secret rather than risk displaying one.
            None => Ok(TerminalInputState { secret: true, ..Default::default() }),
        }
    }

    #[cfg(not(unix))]
    {
        let _ = session;
        Ok(TerminalInputState::default())
    }
}

#[cfg(unix)]
fn input_state_fd(
    fd: std::os::unix::io::RawFd,
    shell_pid: Option<u32>,
) -> TerminalInputState {
    // SAFETY: fd is a live pty master owned by the caller.
    let (echo, canonical) = unsafe {
        let mut t: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut t) != 0 {
            return TerminalInputState { secret: true, ..Default::default() };
        }
        (t.c_lflag & libc::ECHO != 0, t.c_lflag & libc::ICANON != 0)
    };

    let (pgrp, foreground) = foreground_name(fd);
    let at_shell_prompt = match (pgrp, shell_pid) {
        (Some(p), Some(shell)) => p as u32 == shell,
        _ => false,
    };

    // Canonical mode with echo off is a line-oriented secret read (getpass,
    // `read -s`) whoever owns the terminal — `read -s` is a shell builtin, so
    // the shell itself is the foreground process there.
    // Raw mode with echo off is ambiguous: it is equally an ordinary shell
    // prompt (readline), a full-screen program, or sudo with pwfeedback. Only
    // the foreground program separates them.
    let secret = !echo
        && (canonical
            || (!at_shell_prompt && foreground.as_deref().is_some_and(is_secret_program)));

    TerminalInputState { echo, canonical, at_shell_prompt, foreground, secret }
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use portable_pty::PtySize;
    use std::time::{Duration, Instant};

    struct Sh {
        _child: Box<dyn portable_pty::Child + Send + Sync>,
        writer: Box<dyn Write + Send>,
        fd: std::os::unix::io::RawFd,
        pid: Option<u32>,
        _master: Box<dyn portable_pty::MasterPty + Send>,
    }

    fn interactive_shell() -> Sh {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .expect("openpty");
        let mut cmd = CommandBuilder::new("/bin/bash");
        cmd.arg("-i");
        cmd.env("TERM", "xterm");
        cmd.env("PS1", "$ ");
        let child = pair.slave.spawn_command(cmd).expect("spawn");
        let pid = child.process_id();
        let fd = pair.master.as_raw_fd().expect("fd");

        let mut reader = pair.master.try_clone_reader().expect("reader");
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) { if n == 0 { break; } }
        });

        let writer = pair.master.take_writer().expect("writer");
        std::thread::sleep(Duration::from_millis(500));
        Sh { _child: child, writer, fd, pid, _master: pair.master }
    }

    fn wait_for(sh: &Sh, want: bool) -> TerminalInputState {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut last = input_state_fd(sh.fd, sh.pid);
        while Instant::now() < deadline {
            last = input_state_fd(sh.fd, sh.pid);
            if last.secret == want { return last; }
            std::thread::sleep(Duration::from_millis(25));
        }
        last
    }

    /// An interactive shell runs with echo off because readline echoes for
    /// itself. That must never be mistaken for a password prompt.
    #[test]
    fn shell_prompt_is_not_secret() {
        let sh = interactive_shell();
        let st = input_state_fd(sh.fd, sh.pid);
        assert!(!st.secret, "shell prompt must not be secret: {st:?}");
        assert!(st.at_shell_prompt, "shell should own the terminal: {st:?}");
    }

    /// getpass and `read -s` leave canonical mode on with echo off.
    #[test]
    fn canonical_password_prompt_is_secret() {
        let mut sh = interactive_shell();
        sh.writer.write_all(b"read -s -p 'Password: ' x\n").unwrap();
        sh.writer.flush().ok();
        let st = wait_for(&sh, true);
        assert!(st.secret, "read -s must be secret: {st:?}");
    }

    /// Regression: sudo with `pwfeedback` (the default on Linux Mint) reads key
    /// by key to print asterisks, so it leaves ICANON off and ECHO off — the
    /// same termios an ordinary shell prompt has. Only the foreground program
    /// distinguishes them. Simulated with a binary named `sudo` so the test
    /// needs no privileges.
    #[test]
    fn pwfeedback_style_sudo_is_secret() {
        let dir = std::env::temp_dir().join(format!("ngui-sudo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = dir.join("sudo");
        std::fs::copy("/bin/sleep", &fake).unwrap();

        let mut sh = interactive_shell();
        sh.writer
            .write_all(format!("stty -icanon -echo; {} 30\n", fake.display()).as_bytes())
            .unwrap();
        sh.writer.flush().ok();

        let st = wait_for(&sh, true);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            st.secret,
            "raw-mode sudo (pwfeedback) must be secret: {st:?}"
        );
        assert!(!st.canonical, "this case is specifically non-canonical: {st:?}");
    }

    /// Full-screen programs also run raw with echo off, but are not reading
    /// secrets — the keyboard should keep working inside them.
    #[test]
    fn full_screen_program_is_not_secret() {
        let mut sh = interactive_shell();
        sh.writer.write_all(b"stty -icanon -echo; sleep 30\n").unwrap();
        sh.writer.flush().ok();
        std::thread::sleep(Duration::from_millis(700));
        let st = input_state_fd(sh.fd, sh.pid);
        assert!(
            !st.secret,
            "an ordinary raw-mode program must not be secret: {st:?}"
        );
    }
}
