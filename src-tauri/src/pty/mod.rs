use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct PtySession {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    /// Pid of the shell itself. When it is the foreground process group, the
    /// user is at a prompt rather than inside a program.
    shell_pid: Option<u32>,
    /// Held so the shell can be reaped. Dropping it without `wait()` left a
    /// zombie in the process table for every tab closed during the session.
    child: Box<dyn portable_pty::Child + Send + Sync>,
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
    // Keeping the slave open would stop the reader ever seeing EOF when the
    // shell exits, since this process would still hold the other end.
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    state.sessions.lock().map_err(|e| e.to_string())?
        .insert(session_id.clone(), PtySession { writer, master: pair.master, shell_pid, child });

    let app_handle = app.clone();
    let emit_session_id = session_id.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut pending: Vec<u8> = Vec::new();
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    pending.extend_from_slice(&buf[..n]);
                    let data = take_utf8(&mut pending);
                    if data.is_empty() { continue; }
                    let _ = app_handle.emit("pty_output", PtyOutputPayload {
                        session_id: emit_session_id.clone(),
                        data,
                    });
                }
                Err(_) => break,
            }
        }
        // The shell exited on its own (`exit`, Ctrl+D). If pty_close has not
        // already taken the session, reap it here so it does not linger.
        let session = app_handle.state::<PtyState>().sessions.lock()
            .ok()
            .and_then(|mut s| s.remove(&emit_session_id));
        if let Some(session) = session { shutdown(session); }
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
    /// Line-oriented secret read: canonical mode with echo off, as getpass and
    /// `read -s` leave it. Unambiguous and true for exactly as long as the read
    /// lasts, so it can gate display directly.
    pub canonical_secret: bool,
    /// A program that reads secrets owns the terminal in raw mode with echo
    /// off — sudo with pwfeedback. Ambiguous: it stays true for the whole
    /// session after `sudo su` authenticates, because sudo remains the process
    /// group leader. The frontend therefore treats this as a trigger that a
    /// read has *started*, and releases on Enter, rather than as a live truth.
    pub program_secret: bool,
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
/// Reports two separate signals rather than one verdict, because they have
/// very different lifetimes — see the field docs on TerminalInputState.
///
/// Deliberately neither, when the shell itself is in the foreground: an
/// interactive shell always runs with echo off because readline echoes
/// characters itself. Programs like vim or less also run with echo off, but
/// are not in the list, so the keyboard keeps working inside them.
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
            None => Ok(TerminalInputState { canonical_secret: true, program_secret: true, ..Default::default() }),
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
            return TerminalInputState { canonical_secret: true, program_secret: true, ..Default::default() };
        }
        (t.c_lflag & libc::ECHO != 0, t.c_lflag & libc::ICANON != 0)
    };

    let (pgrp, foreground) = foreground_name(fd);
    let at_shell_prompt = match (pgrp, shell_pid) {
        (Some(p), Some(shell)) => p as u32 == shell,
        _ => false,
    };

    // Canonical mode with echo off is a line-oriented secret read whoever owns
    // the terminal — `read -s` is a shell builtin, so the shell itself is the
    // foreground process there.
    let canonical_secret = !echo && canonical;

    // Raw mode with echo off is ambiguous: equally an ordinary shell prompt
    // (readline), a full-screen program, or sudo with pwfeedback. Only the
    // foreground program separates them, and even then only at the moment the
    // program takes the terminal.
    let program_secret = !echo
        && !canonical
        && !at_shell_prompt
        && foreground.as_deref().is_some_and(is_secret_program);

    TerminalInputState { echo, canonical, at_shell_prompt, foreground, canonical_secret, program_secret }
}

#[tauri::command]
pub fn pty_close(
    state: tauri::State<'_, PtyState>,
    session_id: String,
) -> Result<(), String> {
    let session = state.sessions.lock().map_err(|e| e.to_string())?.remove(&session_id);
    // Off the IPC thread: kill() allows the shell a short grace period.
    if let Some(session) = session { std::thread::spawn(move || shutdown(session)); }
    Ok(())
}

/// Hang up the shell and reap it. `kill()` sends SIGHUP, then SIGKILL if the
/// shell is still alive after a grace period, so a shell that traps HUP cannot
/// keep this thread waiting.
fn shutdown(mut session: PtySession) {
    let _ = session.child.kill();
    let _ = session.child.wait();
}

/// Drain the longest valid UTF-8 prefix of `pending`, leaving a trailing
/// partial character for the next read. Decoding each 4 KB read on its own
/// turned any multi-byte character that straddled a read boundary into U+FFFD.
/// Bytes that are invalid outright, rather than incomplete, are replaced.
fn take_utf8(pending: &mut Vec<u8>) -> String {
    let mut out = String::new();
    loop {
        match std::str::from_utf8(pending) {
            Ok(s) => { out.push_str(s); pending.clear(); return out; }
            Err(e) => {
                let valid = e.valid_up_to();
                // SAFETY: from_utf8 just verified this prefix.
                out.push_str(unsafe { std::str::from_utf8_unchecked(&pending[..valid]) });
                match e.error_len() {
                    // Incomplete sequence at the end — wait for more bytes.
                    None => { pending.drain(..valid); return out; }
                    Some(bad) => { out.push('\u{FFFD}'); pending.drain(..valid + bad); }
                }
            }
        }
    }
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

#[cfg(test)]
mod utf8_tests {
    use super::take_utf8;

    #[test]
    fn split_multibyte_character_is_held_until_complete() {
        let bytes = "a€b".as_bytes(); // € is 3 bytes
        let mut pending = bytes[..2].to_vec();
        assert_eq!(take_utf8(&mut pending), "a");
        assert_eq!(pending.len(), 1, "partial character must be kept");
        pending.extend_from_slice(&bytes[2..]);
        assert_eq!(take_utf8(&mut pending), "€b");
        assert!(pending.is_empty());
    }

    #[test]
    fn invalid_bytes_are_replaced_not_held() {
        let mut pending = vec![b'x', 0xFF, b'y'];
        assert_eq!(take_utf8(&mut pending), "x\u{FFFD}y");
        assert!(pending.is_empty(), "invalid bytes must not stall the stream");
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

    fn wait_for(sh: &Sh, pred: impl Fn(&TerminalInputState) -> bool) -> TerminalInputState {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut last = input_state_fd(sh.fd, sh.pid);
        while Instant::now() < deadline {
            last = input_state_fd(sh.fd, sh.pid);
            if pred(&last) { return last; }
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
        assert!(!st.canonical_secret && !st.program_secret, "shell prompt is not a read: {st:?}");
        assert!(st.at_shell_prompt, "shell should own the terminal: {st:?}");
    }

    /// getpass and `read -s` leave canonical mode on with echo off.
    #[test]
    fn canonical_password_prompt_is_secret() {
        let mut sh = interactive_shell();
        sh.writer.write_all(b"read -s -p 'Password: ' x\n").unwrap();
        sh.writer.flush().ok();
        let st = wait_for(&sh, |s| s.canonical_secret);
        assert!(st.canonical_secret, "read -s must be a canonical secret read: {st:?}");
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

        let st = wait_for(&sh, |s| s.program_secret);
        std::fs::remove_dir_all(&dir).ok();
        assert!(st.program_secret, "raw-mode sudo (pwfeedback) must trigger: {st:?}");
        assert!(!st.canonical_secret, "and must not be reported as a canonical read: {st:?}");
    }

    /// Full-screen programs also run raw with echo off, but are not reading
    /// secrets — the keyboard should keep working inside them.
    #[test]
    fn full_screen_program_is_not_secret() {
        let mut sh = interactive_shell();
        sh.writer.write_all(b"stty -icanon -echo; sleep 30\n").unwrap();
        sh.writer.flush().ok();

        // Wait for the mode to actually take effect, rather than sleeping a
        // fixed interval — on a slow runner the assertion would otherwise pass
        // vacuously, before echo was ever turned off.
        let st = wait_for(&sh, |s| !s.echo && !s.canonical && !s.at_shell_prompt);
        assert!(!st.echo && !st.canonical, "raw mode never took effect: {st:?}");
        assert!(
            !st.canonical_secret && !st.program_secret,
            "an ordinary raw-mode program must not be secret: {st:?}"
        );
    }

    #[test]
    fn closed_session_is_reaped_not_left_a_zombie() {
        let pair = native_pty_system()
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .expect("openpty");
        let mut cmd = CommandBuilder::new("/bin/bash");
        cmd.arg("-i");
        let child = pair.slave.spawn_command(cmd).expect("spawn");
        let pid = child.process_id().expect("pid");
        let session = PtySession {
            writer: pair.master.take_writer().expect("writer"),
            master: pair.master,
            shell_pid: Some(pid),
            child,
        };
        std::thread::sleep(Duration::from_millis(300));

        shutdown(session);

        // Reaped means gone from /proc entirely; a zombie would still be listed
        // with state Z.
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
        assert!(stat.is_err(), "shell {pid} still in the process table: {stat:?}");
    }
}
