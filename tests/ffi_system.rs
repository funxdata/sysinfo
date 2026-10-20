mod common;

use common::call_no_input;

use sys_info::{
    post_component_info, post_cpu_info, post_disk_info, post_memory_info, post_network_info,
    post_process_list, post_system_basic_info,
};

#[test]
fn system_basic_info() {
    let v = call_no_input(post_system_basic_info);
    assert_eq!(v["code"], 200);
    assert!(v["data"]["uptime"].is_u64());
    assert!(v["data"]["boot_time"].is_u64());
}

#[test]
fn memory_info() {
    let v = call_no_input(post_memory_info);
    assert_eq!(v["code"], 200);
    let total = v["data"]["total_bytes"].as_u64().unwrap();
    let used = v["data"]["used_bytes"].as_u64().unwrap();
    assert!(total > 0);
    assert!(used <= total);
}

#[test]
fn cpu_info() {
    let v = call_no_input(post_cpu_info);
    assert_eq!(v["code"], 200);
    let n = v["data"]["logical_core_count"].as_u64().unwrap();
    assert!(n > 0);
    assert_eq!(v["data"]["cpus"].as_array().unwrap().len() as u64, n);
}

#[test]
fn disk_info() {
    let v = call_no_input(post_disk_info);
    assert_eq!(v["code"], 200);
    let count = v["data"]["count"].as_u64().unwrap();
    assert_eq!(v["data"]["disks"].as_array().unwrap().len() as u64, count);
}

#[test]
fn network_info() {
    let v = call_no_input(post_network_info);
    assert_eq!(v["code"], 200);
    assert!(v["data"]["interfaces"].is_array());
}

#[test]
fn process_list() {
    let v = call_no_input(post_process_list);
    assert_eq!(v["code"], 200);
    assert!(v["data"]["count"].as_u64().unwrap() > 0);
}

#[test]
fn component_info_may_be_empty() {
    let v = call_no_input(post_component_info);
    assert_eq!(v["code"], 200);
    assert!(v["data"]["components"].is_array());
}
