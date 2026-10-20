#![allow(dead_code)]

use std::ffi::{c_char, CStr, CString};

use sys_info::utils::cjson::MAX_OUTPUT_LEN;

pub type FfiFn = unsafe extern "C" fn(*const c_char, *mut c_char) -> i32;

pub fn new_buf() -> Vec<i8> {
    vec![0i8; MAX_OUTPUT_LEN]
}

pub fn read_buf(buf: &[i8]) -> String {
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_str()
        .unwrap_or("")
        .to_string()
}

pub fn call(f: FfiFn, input: Option<&str>) -> serde_json::Value {
    let mut buf = new_buf();
    let c_in = input.map(|s| CString::new(s).unwrap());
    let in_ptr = c_in
        .as_ref()
        .map(|c| c.as_ptr())
        .unwrap_or(std::ptr::null());
    let ret = unsafe { f(in_ptr, buf.as_mut_ptr()) };
    assert_eq!(ret, 0, "FFI 返回码应为 0");
    let out = read_buf(&buf);
    serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("输出不是合法 JSON: {e}\n原始输出: {out}"))
}

pub fn call_no_input(f: FfiFn) -> serde_json::Value {
    call(f, None)
}

pub fn call_raw(f: FfiFn, input: *const c_char, buf: &mut [i8]) -> (i32, String) {
    let ret = unsafe { f(input, buf.as_mut_ptr()) };
    (ret, read_buf(buf))
}

pub fn unique_name(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}_{}", prefix, std::process::id(), nanos)
}

pub fn json_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn path_input(path: &str) -> String {
    format!(r#"{{"path":"{}"}}"#, json_path(path))
}

pub fn dir_name_input(name: &str) -> String {
    format!(r#"{{"dir_name":"{}"}}"#, name.replace('"', "\\\""))
}
