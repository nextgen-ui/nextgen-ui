use serde::Serialize;
use sysinfo::{Networks, Disks, System};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct SystemStats {
    cpu_usage: f32,
    cpu_per_core: Vec<f32>,
    mem_used: u64,
    mem_total: u64,
    swap_used: u64,
    swap_total: u64,
    net_in: u64,
    net_out: u64,
    disk_read: u64,
    disk_write: u64,
}

pub fn start_stats_emitter(app: AppHandle, interval_ms: u64) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new_with_refreshed_list();
        let mut disks = Disks::new_with_refreshed_list();

        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            networks.refresh(true);
            disks.refresh(true);

            let cpu_per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
            let cpu_usage = if cpu_per_core.is_empty() { 0.0 }
                else { cpu_per_core.iter().sum::<f32>() / cpu_per_core.len() as f32 };

            let net_in: u64 = networks.iter().map(|(_, n)| n.received()).sum();
            let net_out: u64 = networks.iter().map(|(_, n)| n.transmitted()).sum();
            let disk_read: u64 = disks.iter().map(|d| d.usage().read_bytes).sum();
            let disk_write: u64 = disks.iter().map(|d| d.usage().written_bytes).sum();

            let _ = app.emit("system_stats", SystemStats {
                cpu_usage,
                cpu_per_core,
                mem_used: sys.used_memory(),
                mem_total: sys.total_memory(),
                swap_used: sys.used_swap(),
                swap_total: sys.total_swap(),
                net_in,
                net_out,
                disk_read,
                disk_write,
            });

            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        }
    });
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    let mut sys = System::new_all();
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    let cpu_per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let cpu_usage = if cpu_per_core.is_empty() { 0.0 }
        else { cpu_per_core.iter().sum::<f32>() / cpu_per_core.len() as f32 };
    SystemStats {
        cpu_usage,
        cpu_per_core,
        mem_used: sys.used_memory(),
        mem_total: sys.total_memory(),
        swap_used: sys.used_swap(),
        swap_total: sys.total_swap(),
        net_in: 0,
        net_out: 0,
        disk_read: 0,
        disk_write: 0,
    }
}
