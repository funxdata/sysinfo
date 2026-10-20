/// 系统基础信息（静态 + 负载）
pub fn get_basic_info() -> Result<serde_json::Value, (i32, String)> {
    let load = sysinfo::System::load_average();
    Ok(serde_json::json!({
        "name": sysinfo::System::name(),
        "kernel_version": sysinfo::System::kernel_version(),
        "os_version": sysinfo::System::os_version(),
        "host_name": sysinfo::System::host_name(),
        "uptime": sysinfo::System::uptime(),
        "boot_time": sysinfo::System::boot_time(),
        "load_average": {
            "one": load.one,
            "five": load.five,
            "fifteen": load.fifteen,
        },
    }))
}
