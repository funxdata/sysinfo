use serde::Deserialize;

use crate::utils::errcode;
use crate::utils::state::with_system;

/// 分页参数
#[derive(Deserialize)]
pub struct PageInput {
    /// 每页条数，默认 100，上限 1000
    pub limit: Option<usize>,
    /// 偏移量，默认 0
    pub offset: Option<usize>,
}

impl PageInput {
    pub fn normalized(&self) -> (usize, usize) {
        let limit = self.limit.unwrap_or(100).clamp(1, 1000);
        let offset = self.offset.unwrap_or(0);
        (limit, offset)
    }
}

/// 进程列表（分页）
pub fn get_process_list(page: PageInput) -> Result<serde_json::Value, (i32, String)> {
    let (limit, offset) = page.normalized();

    with_system(|sys| {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let all = sys.processes();
        let total = all.len();

        // 按 pid 排序，保证分页稳定
        let mut entries: Vec<_> = all.iter().collect();
        entries.sort_by_key(|(pid, _)| pid.as_u32());

        let page_items: Vec<serde_json::Value> = entries
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(pid, p)| {
                let mut name = p.name().to_string_lossy().into_owned();
                if name.len() > 128 {
                    name.truncate(128);
                }
                serde_json::json!({
                    "pid": pid.as_u32(),
                    "name": name,
                    "memory": p.memory(),
                    "cpu_usage": p.cpu_usage(),
                    "status": format!("{:?}", p.status()),
                })
            })
            .collect();

        let returned = page_items.len();
        let has_more = offset + returned < total;

        serde_json::json!({
            "total": total,
            "offset": offset,
            "limit": limit,
            "count": returned,
            "has_more": has_more,
            "processes": page_items,
        })
    })
    .map_err(|e| {
        if e.0 == errcode::ERR_INTERNAL_LOCK {
            e
        } else {
            (errcode::ERR_PROCESS_INFO, e.1)
        }
    })
}
