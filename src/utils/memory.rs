use std::sync::Mutex;
use sysinfo::System;

use crate::utils::errcode;

static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

/// 获取或初始化全局 System 实例（sysinfo 需要保留历史值计算差值）
fn with_system<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&System) -> R,
{
    let mut guard = SYSTEM
        .lock()
        .map_err(|e| (errcode::ERR_MEMORY_INFO, format!("lock failed: {}", e)))?;

    if guard.is_none() {
        *guard = Some(System::new());
    }

    let sys = guard.as_ref().unwrap();
    Ok(f(sys))
}

/// 刷新并获取内存信息
pub fn get_memory_info() -> Result<serde_json::Value, (i32, String)> {
    // 先刷新
    {
        let mut guard = SYSTEM
            .lock()
            .map_err(|e| (errcode::ERR_MEMORY_INFO, format!("lock failed: {}", e)))?;
        if guard.is_none() {
            *guard = Some(System::new());
        }
        guard.as_mut().unwrap().refresh_memory();
    }

    with_system(|sys| {
        let total = sys.total_memory();
        let used = sys.used_memory();
        let free = sys.free_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();

        let usage_percent = if total > 0 {
            (used as f64 / total as f64 * 100.0 * 100.0).round() / 100.0
        } else {
            0.0
        };

        serde_json::json!({
            "total_bytes": total,
            "used_bytes": used,
            "free_bytes": free,
            "usage_percent": usage_percent,
            "total_swap_bytes": total_swap,
            "used_swap_bytes": used_swap,
        })
    })
}
