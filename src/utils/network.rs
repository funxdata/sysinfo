use crate::utils::state::with_networks;

/// 网络接口信息
pub fn get_network_info() -> Result<serde_json::Value, (i32, String)> {
    with_networks(|networks| {
        networks.refresh(true);

        let interfaces: Vec<serde_json::Value> = networks
            .list()
            .iter()
            .map(|(name, data)| {
                serde_json::json!({
                    "interface": name,
                    "total_received": data.total_received(),
                    "total_transmitted": data.total_transmitted(),
                    "total_packets_received": data.total_packets_received(),
                    "total_packets_transmitted": data.total_packets_transmitted(),
                    "total_errors_on_received": data.total_errors_on_received(),
                    "total_errors_on_transmitted": data.total_errors_on_transmitted(),
                    "mac_address": format!("{:?}", data.mac_address()),
                })
            })
            .collect();

        serde_json::json!({
            "count": interfaces.len(),
            "interfaces": interfaces,
        })
    })
}


