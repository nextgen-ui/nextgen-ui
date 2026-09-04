use serde::Serialize;
use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};

#[derive(Serialize, Clone, Debug, Default)]
pub struct LocalNetInfo {
    pub local_ip: Option<String>,
    pub interface_name: Option<String>,
    pub ping_ms: Option<u32>,
}

#[tauri::command]
pub fn get_local_net_info() -> LocalNetInfo {
    let local_ip = get_local_ip();
    let interface_name = get_default_interface();
    let ping_ms = measure_ping();

    LocalNetInfo {
        local_ip,
        interface_name,
        ping_ms,
    }
}

#[tauri::command]
pub fn measure_ping() -> Option<u32> {
    let targets = ["1.1.1.1:53", "8.8.8.8:53", "9.9.9.9:53"];
    for target in targets {
        let start = Instant::now();
        if let Ok(addr) = target.parse() {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(1500)).is_ok() {
                return Some(start.elapsed().as_millis() as u32);
            }
        }
    }
    None
}

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("1.1.1.1:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

fn get_default_interface() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/net/route") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "00000000" {
                    return Some(parts[0].to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_net_info_does_not_panic() {
        let info = get_local_net_info();
        // Just verify it returns without panicking
        let _ = info.local_ip;
        let _ = info.interface_name;
    }

    #[test]
    fn ping_measurement_does_not_panic() {
        let _ = measure_ping();
    }
}
