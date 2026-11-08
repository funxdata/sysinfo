use crate::utils::state;

/// 刷新并获取内存信息。
pub fn get_memory_info() -> Result<serde_json::Value, (i32, String)> {
    state::with_system(|sys| {
        sys.refresh_memory();
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
