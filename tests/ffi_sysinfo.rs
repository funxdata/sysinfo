mod common;

use common::call_no_input;

use sys_info::{post_disk_info, post_memory_info};

#[test]
fn memory_info_fields() {
    let v = call_no_input(post_memory_info);
    assert_eq!(v["code"], 200);

    let total = v["data"]["total_bytes"].as_u64().unwrap();
    let used = v["data"]["used_bytes"].as_u64().unwrap();
    let free = v["data"]["free_bytes"].as_u64().unwrap();
    let pct = v["data"]["usage_percent"].as_f64().unwrap();

    assert!(total > 0);
    assert!(used <= total);
    assert!(free <= total);
    assert!((0.0..=100.0).contains(&pct));
    assert!(v["data"]["total_swap_bytes"].is_u64());
    assert!(v["data"]["used_swap_bytes"].is_u64());
}

#[test]
fn memory_info_repeatable() {
    let a = call_no_input(post_memory_info);
    let b = call_no_input(post_memory_info);
    assert_eq!(a["data"]["total_bytes"], b["data"]["total_bytes"]);
}

#[test]
fn disk_info_basic() {
    let v = call_no_input(post_disk_info);
    assert_eq!(v["code"], 200);

    let count = v["data"]["count"].as_u64().unwrap();
    assert!(count > 0, "至少应检测到一个磁盘");
    let disks = v["data"]["disks"].as_array().unwrap();
    assert_eq!(disks.len() as u64, count);
    assert!(v["data"]["total_bytes"].as_u64().is_some());
}

#[test]
fn disk_info_each_disk_fields() {
    let v = call_no_input(post_disk_info);
    for d in v["data"]["disks"].as_array().unwrap() {
        assert!(d["name"].is_string());
        assert!(d["mount_point"].is_string());
        assert!(d["file_system"].is_string());

        let total = d["total_bytes"].as_u64().unwrap();
        let used = d["used_bytes"].as_u64().unwrap();
        let avail = d["available_bytes"].as_u64().unwrap();
        assert!(used <= total);
        assert!(avail <= total);

        let pct = d["usage_percent"].as_f64().unwrap();
        assert!((0.0..=100.0).contains(&pct));

        assert!(d["is_removable"].is_boolean());
        assert!(d["is_read_only"].is_boolean());
        assert!(d["kind"].is_string());
    }
}
