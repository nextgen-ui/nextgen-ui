use serde::Serialize;
use sysinfo::System;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct SystemStats {
    cpu_usage: f32,
    cpu_per_core: Vec<f32>,
    mem_used: u64,
    mem_total: u64,
    swap_used: u64,
    swap_total: u64,
}

pub fn start_stats_emitter(app: AppHandle, interval_ms: u64) {
    std::thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let cpu_per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
            let cpu_usage = if cpu_per_core.is_empty() { 0.0 }
                else { cpu_per_core.iter().sum::<f32>() / cpu_per_core.len() as f32 };
            let _ = app.emit("system_stats", SystemStats {
                cpu_usage,
                cpu_per_core,
                mem_used: sys.used_memory(),
                mem_total: sys.total_memory(),
                swap_used: sys.used_swap(),
                swap_total: sys.total_swap(),
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
    }
}
