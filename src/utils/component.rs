use crate::utils::state::with_components;

/// 温度传感器组件
pub fn get_component_info() -> Result<serde_json::Value, (i32, String)> {
    with_components(|components| {
        components.refresh(true);

        let items: Vec<serde_json::Value> = components
            .list()
            .iter()
            .map(|c| {
                serde_json::json!({
                    "label": c.label(),
                    "temperature": c.temperature(),
                    "max": c.max(),
                    "critical": c.critical(),
                })
            })
            .collect();

        serde_json::json!({
            "count": items.len(),
            "components": items,
        })
    })
}


