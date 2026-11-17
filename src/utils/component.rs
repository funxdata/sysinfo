use crate::utils::state::with_components;

/// 温度传感器组件
pub fn get_component_info() -> Result<serde_json::Value, (i32, String)> {
    with_components(|components| {
        components.refresh(true);

        let items: Vec<serde_json::Value> = components
            .list()
            .iter()
            .map(|c| {
                // sysinfo 用 -273.15 表示阈值不可用，统一转为 null
                let max = c.max().filter(|v| *v > -273.0);
                let critical = c.critical().filter(|v| *v > -273.0);
                serde_json::json!({
                    "label": c.label(),
                    "temperature": c.temperature(),
                    "max": max,
                    "critical": critical,
                })
            })
            .collect();

        serde_json::json!({
            "count": items.len(),
            "components": items,
        })
    })
}
