use crate::utils::state::with_disks;

/// 磁盘信息（0.39 支持 is_read_only 与 usage 读写字节）
pub fn get_disk_info() -> Result<serde_json::Value, (i32, String)> {
    with_disks(|disks| {
        disks.refresh(true);

        let mut total_bytes: u64 = 0;
        let mut total_used_bytes: u64 = 0;
        let mut total_available_bytes: u64 = 0;
        let mut items = Vec::with_capacity(disks.len());

        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                (used as f64 / total as f64 * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            };

            total_bytes += total;
            total_used_bytes += used;
            total_available_bytes += available;

            let usage = disk.usage();

            items.push(serde_json::json!({
                "name": disk.name().to_string_lossy(),
                "mount_point": disk.mount_point().to_string_lossy(),
                "file_system": disk.file_system().to_string_lossy(),
                "total_bytes": total,
                "used_bytes": used,
                "available_bytes": available,
                "usage_percent": usage_percent,
                "is_removable": disk.is_removable(),
                "is_read_only": disk.is_read_only(),
                "kind": format!("{:?}", disk.kind()),
                "read_bytes": usage.total_read_bytes,
                "written_bytes": usage.total_written_bytes,
            }));
        }

        serde_json::json!({
            "count": items.len(),
            "total_bytes": total_bytes,
            "total_used_bytes": total_used_bytes,
            "total_available_bytes": total_available_bytes,
            "disks": items,
        })
    })
}

