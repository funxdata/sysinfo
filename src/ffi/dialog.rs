use std::ffi::{c_char, c_int};

use crate::utils::cjson::write_success;
use crate::utils::dialog;

/// 弹出系统文件选择对话框
///
/// 输入：可传 null。
/// 输出：选择文件时 `data.path` 为绝对路径；取消时 `data.path` 为 null、`msg` 为 "cancelled"。
#[unsafe(no_mangle)]
pub extern "C" fn post_choose_file(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match dialog::choose_file() {
        Some(path) => {
            let p = path.to_string_lossy().into_owned();
            write_success(json_out, "success", serde_json::json!({ "path": p }))
        }
        None => write_success(json_out, "cancelled", serde_json::json!({ "path": null })),
    }
}

/// 弹出系统目录选择对话框
///
/// 输入：可传 null。
/// 输出：选择目录时 `data.path` 为绝对路径；取消时 `data.path` 为 null、`msg` 为 "cancelled"。
#[unsafe(no_mangle)]
pub extern "C" fn post_choose_dir(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match dialog::choose_dir() {
        Some(path) => {
            let p = path.to_string_lossy().into_owned();
            write_success(json_out, "success", serde_json::json!({ "path": p }))
        }
        None => write_success(json_out, "cancelled", serde_json::json!({ "path": null })),
    }
}
