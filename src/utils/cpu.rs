use crate::utils::state::with_system;

/// CPU 信息
pub fn get_cpu_info() -> Result<serde_json::Value, (i32, String)> {
    with_system(|sys| {
        sys.refresh_cpu_all();

        let cpus: Vec<serde_json::Value> = sys
            .cpus()
            .iter()
            .map(|cpu| {
                serde_json::json!({
                    "name": cpu.name(),
                    "brand": cpu.brand(),
                    "vendor_id": cpu.vendor_id(),
                    "frequency": cpu.frequency(),
                    "usage": cpu.cpu_usage(),
                })
            })
            .collect();

        serde_json::json!({
            "global_usage": sys.global_cpu_usage(),
            "physical_core_count": sysinfo::System::physical_core_count(),
            "logical_core_count": cpus.len(),
            "cpus": cpus,
        })
    })
}

