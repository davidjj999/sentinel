use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        Self { sys }
    }

    pub fn get_stats(&mut self) -> String {
        // Refresh triggers data collection
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();

        let cpu_count = self.sys.cpus().len();
        let global_cpu_usage = self.sys.global_cpu_usage();
        
        // Calculate average CPU usage if global isn't sufficient or for more detail
        // But global_cpu_info() is usually good.
        // Actually, refresh_cpu() updates all cpus. global_cpu_info might not be populated in all versions automatically?
        // Let's use the average of all CPUs for safety or just the global one if available.
        // In sysinfo 0.30+, global_cpu_info() works well.

        let total_memory = self.sys.total_memory() / 1024 / 1024; // MB
        let used_memory = self.sys.used_memory() / 1024 / 1024; // MB
        let total_swap = self.sys.total_swap() / 1024 / 1024; // MB
        let used_swap = self.sys.used_swap() / 1024 / 1024; // MB

        let uptime = System::uptime();
        let uptime_hours = uptime / 3600;
        let uptime_minutes = (uptime % 3600) / 60;

        format!(
            "System Report:\n\
            Uptime: {}h {}m\n\
            CPU Usage: {:.2}%\n\
            Active Cores: {}\n\
            Memory: {} / {} MB (Used/Total)\n\
            Swap: {} / {} MB\n\
            Hostname: {}\n\
            OS: {} {}",
            uptime_hours, uptime_minutes,
            global_cpu_usage,
            cpu_count,
            used_memory, total_memory,
            used_swap, total_swap,
            System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            System::name().unwrap_or_else(|| "Unknown".to_string()),
            System::os_version().unwrap_or_else(|| "".to_string())
        )
    }
}
