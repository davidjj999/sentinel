use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

pub struct SystemMonitor {
    sys: System,
    components: Components,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        let components = Components::new_with_refreshed_list();
        Self { sys, components }
    }

    async fn get_bitcoind_logs(&self) -> Option<String> {
        use tokio::process::Command;
        let output = Command::new("docker")
            .args(["logs", "--tail", "15", "bitcoind"])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if stdout.trim().is_empty() {
                    Some("Bitcoind Logs: <empty>".to_string())
                } else {
                    Some(format!("Bitcoind Logs:\n{}", stdout.trim()))
                }
            }
            _ => None, // Gracefully handle absence or error
        }
    }

    pub async fn get_stats(&mut self) -> String {
        // Refresh triggers data collection
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        self.components.refresh(true);

        let cpu_count = self.sys.cpus().len();
        let global_cpu_usage = self.sys.global_cpu_usage();

        let total_memory = self.sys.total_memory() / 1024 / 1024; // MB
        let used_memory = self.sys.used_memory() / 1024 / 1024; // MB
        let total_swap = self.sys.total_swap() / 1024 / 1024; // MB
        let used_swap = self.sys.used_swap() / 1024 / 1024; // MB

        let uptime = System::uptime();
        let uptime_hours = uptime / 3600;
        let uptime_minutes = (uptime % 3600) / 60;

        // Temperature information
        let mut temp_info = String::from("Temperatures:\n");
        for component in &self.components {
            let temp = component.temperature().unwrap_or(0.0);
            temp_info.push_str(&format!("  {}: {:.1}°C\n", component.label(), temp));
        }
        if self.components.is_empty() {
             temp_info.push_str("  No temperature sensors found.\n");
        }

        // Top process by CPU
        let mut top_process_info = String::from("Top Process: None");
        if let Some((pid, proc)) = self.sys.processes().iter().max_by(|(_, a), (_, b)| {
            a.cpu_usage().partial_cmp(&b.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            top_process_info = format!("Top Process: {} (PID: {}) - {:.1}% CPU", proc.name().to_string_lossy(), pid, proc.cpu_usage());
        }

        let mut report = format!(
            "System Report:\n\
            Uptime: {}h {}m\n\
            CPU Usage: {:.2}%\n\
            Active Cores: {}\n\
            Memory: {} / {} MB (Used/Total)\n\
            Swap: {} / {} MB\n\
            Hostname: {}\n\
            OS: {} {}\n\
            {}\n\
            {}",
            uptime_hours, uptime_minutes,
            global_cpu_usage,
            cpu_count,
            used_memory, total_memory,
            used_swap, total_swap,
            System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "".to_string()),
            temp_info.trim_end(), // Remove trailing newline from temp loop
            top_process_info
        );

        if let Some(logs) = self.get_bitcoind_logs().await {
            report.push_str("\n\n");
            report.push_str(&logs);
        }

        report
    }
}

