mod common;

use std::ffi::CString;

use common::{call_raw, new_buf};

use sys_info::post_create_dir;

#[test]
fn null_input_returns_error_json() {
    let mut buf = new_buf();
    let (ret, out) = call_raw(post_create_dir, std::ptr::null(), &mut buf);
    assert_eq!(ret, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["code"], 400100);
    assert!(v["data"].is_null());
}

#[test]
fn invalid_utf8_input() {
    // c_char 在多数 Unix 平台为 i8；构造 [-1, 0] 表示非法 UTF-8 字节 + null 结尾
    let bad: [std::os::raw::c_char; 2] = [-1, 0];
    let mut buf = new_buf();
    let (ret, out) = call_raw(post_create_dir, bad.as_ptr(), &mut buf);
    assert_eq!(ret, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["code"], 400101);
}

#[test]
fn missing_field_json() {
    let c = CString::new(r#"{"other":"x"}"#).unwrap();
    let mut buf = new_buf();
    let (ret, out) = call_raw(post_create_dir, c.as_ptr(), &mut buf);
    assert_eq!(ret, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["code"], 400102);
}

#[test]
fn null_output_buffer_returns_minus_one() {
    let c = CString::new(r#"{"dir_name":"x"}"#).unwrap();
    let ret = post_create_dir(c.as_ptr(), std::ptr::null_mut());
    assert_eq!(ret, -1);
}


