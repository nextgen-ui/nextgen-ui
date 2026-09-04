use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, ProcessesToUpdate,
    RefreshKind, System,
};
use tauri::{AppHandle, Emitter};

/// How many processes the monitor panel shows.
const TOP_PROCESSES: usize = 12;

/// Walking every process is by far the most expensive thing the sampler does,
/// and the panel that consumes it is usually closed. The frontend flips this
/// flag as ProcessMonitor mounts and unmounts.
#[derive(Default)]
pub struct StatsState {
    pub processes_wanted: Arc<AtomicBool>,
}

#[tauri::command]
pub fn set_process_stream(state: tauri::State<'_, StatsState>, enabled: bool) {
    state.processes_wanted.store(enabled, Ordering::Relaxed);
}

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

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu: f32,
    mem: u64,
    status: String,
}

pub fn start_stats_emitter(app: AppHandle, interval_ms: u64, processes_wanted: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        // Only CPU and memory are ever read off a process, so skip the cmdline,
        // exe, cwd, environ and user lookups a full refresh would do per process.
        let proc_kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                .with_memory(MemoryRefreshKind::everything()),
        );
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

            let net_in: u64  = networks.values().map(|n| n.received()).sum();
            let net_out: u64 = networks.values().map(|n| n.transmitted()).sum();
            let disk_read: u64  = disks.iter().map(|d| d.usage().read_bytes).sum();
            let disk_write: u64 = disks.iter().map(|d| d.usage().written_bytes).sum();

            let _ = app.emit("system_stats", SystemStats {
                cpu_usage, cpu_per_core,
                mem_used: sys.used_memory(),
                mem_total: sys.total_memory(),
                swap_used: sys.used_swap(),
                swap_total: sys.total_swap(),
                net_in, net_out, disk_read, disk_write,
            });

            if processes_wanted.load(Ordering::Relaxed) {
                sys.refresh_processes_specifics(ProcessesToUpdate::All, true, proc_kind);

                // Rank by borrowing, then allocate strings only for the rows that
                // survive — the old code built two Strings for every process on
                // the system each second and discarded all but twelve.
                let mut ranked: Vec<&sysinfo::Process> = sys.processes().values().collect();
                ranked.sort_unstable_by(|a, b| {
                    b.cpu_usage()
                        .partial_cmp(&a.cpu_usage())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                ranked.truncate(TOP_PROCESSES);

                let procs: Vec<ProcessInfo> = ranked
                    .into_iter()
                    .map(|p| ProcessInfo {
                        pid: p.pid().as_u32(),
                        name: p.name().to_string_lossy().to_string(),
                        cpu: p.cpu_usage(),
                        mem: p.memory(),
                        status: format!("{:?}", p.status()),
                    })
                    .collect();
                let _ = app.emit("process_list", procs);
            }

            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        }
    });
}

#[tauri::command]
pub fn get_system_stats() -> SystemStats {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::everything()),
    );
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    let cpu_per_core: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
    let cpu_usage = if cpu_per_core.is_empty() { 0.0 }
        else { cpu_per_core.iter().sum::<f32>() / cpu_per_core.len() as f32 };
    SystemStats {
        cpu_usage, cpu_per_core,
        mem_used: sys.used_memory(),
        mem_total: sys.total_memory(),
        swap_used: sys.used_swap(),
        swap_total: sys.total_swap(),
        net_in: 0, net_out: 0,
        disk_read: 0, disk_write: 0,
    }
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    // Refresh just this pid rather than snapshotting the whole system.
    let target = sysinfo::Pid::from_u32(pid);
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[target]),
        true,
        ProcessRefreshKind::nothing(),
    );
    if let Some(process) = sys.process(target) {
        process.kill();
        Ok(())
    } else {
        Err(format!("Process {} not found", pid))
    }
}
