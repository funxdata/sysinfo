mod common;

use common::{call, call_no_input, dir_name_input, path_input, unique_name};

use sys_info::{post_create_dir, post_cur_dir, post_file_exists, post_file_info};

#[test]
fn cur_dir_null_input() {
    let v = call_no_input(post_cur_dir);
    assert_eq!(v["code"], 200);
    assert!(std::path::Path::new(v["data"]["path"].as_str().unwrap()).is_absolute());
}

#[test]
fn cur_dir_invalid_json() {
    let v = call(post_cur_dir, Some("{bad}"));
    assert_eq!(v["code"], 400102);
}

#[test]
fn create_dir_ok_and_cleanup() {
    let name = unique_name("ffi_dir");
    let v = call(post_create_dir, Some(&dir_name_input(&name)));
    assert_eq!(v["code"], 200);
    let path = v["data"]["path"].as_str().unwrap().to_string();
    assert!(std::path::Path::new(&path).is_dir());
    let _ = std::fs::remove_dir_all(&path);
}

#[test]
fn create_dir_empty() {
    let v = call(post_create_dir, Some(&dir_name_input("   ")));
    assert_eq!(v["code"], 400103);
}

#[test]
fn create_dir_traversal() {
    let v = call(post_create_dir, Some(&dir_name_input("../evil")));
    assert_eq!(v["code"], 400104);
}

#[test]
fn file_exists_true_then_false() {
    let name = unique_name("ffi_exists");
    let created = call(post_create_dir, Some(&dir_name_input(&name)));
    let path = created["data"]["path"].as_str().unwrap().to_string();

    let v = call(post_file_exists, Some(&path_input(&path)));
    assert_eq!(v["data"]["exists"], true);

    let _ = std::fs::remove_dir_all(&path);

    let v2 = call(post_file_exists, Some(&path_input(&path)));
    assert_eq!(v2["data"]["exists"], false);
}

#[test]
fn file_info_dir() {
    let name = unique_name("ffi_info");
    let created = call(post_create_dir, Some(&dir_name_input(&name)));
    let path = created["data"]["path"].as_str().unwrap().to_string();

    let v = call(post_file_info, Some(&path_input(&path)));
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["is_dir"], true);
    assert_eq!(v["data"]["is_file"], false);

    let _ = std::fs::remove_dir_all(&path);
}

#[test]
fn file_info_missing() {
    let v = call(post_file_info, Some(r#"{"path":"/no/such/path/xyz_123"}"#));
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["exists"], false);
}
