use std::ffi::{c_char, c_int};
use std::io::ErrorKind;

use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{guard, parse_json_input, write_error, write_success};
use crate::utils::errcode;
use crate::utils::file;

/// 统一读取路径输入
#[derive(Deserialize)]
struct PathInput {
    path: String,
}

/// 获取当前工作目录
///
/// 不需要输入参数：`json_in` 允许为 null 指针，此时跳过输入解析。
///
/// # Safety
/// `json_in` 必须为 null 或指向以 null 结尾的有效 UTF-8 C 字符串；
/// `json_out` 必须至少有 `MAX_OUTPUT_LEN` 字节可用空间。
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn post_cur_dir(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    guard(json_out, || {
        // json_in 为 null 时视为无输入；否则要求是合法 JSON（容忍任意结构）
        if !json_in.is_null() {
            if let Err((code, msg)) = unsafe { parse_json_input::<serde_json::Value>(json_in) } {
                return write_error(json_out, code, &msg);
            }
        }

        match file::current_dir() {
            Ok(path) => write_success(json_out, "success", json!({ "path": path })),
            Err((code, msg)) => write_error(json_out, code, &msg),
        }
    })
}

/// 在当前目录下创建新目录
///
/// # Safety
/// 同 [`post_cur_dir`]。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn post_create_dir(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    guard(json_out, || {
        #[derive(Deserialize)]
        struct Input {
            dir_name: String,
        }

        let req: Input = match unsafe { parse_json_input(json_in) } {
            Ok(v) => v,
            Err((code, msg)) => return write_error(json_out, code, &msg),
        };

        if req.dir_name.trim().is_empty() {
            return write_error(json_out, errcode::ERR_EMPTY_FIELD, "dir_name is empty");
        }

        match file::create_dir(&req.dir_name) {
            Ok(path) => write_success(json_out, "success", json!({ "path": path })),
            Err((code, msg)) => write_error(json_out, code, &msg),
        }
    })
}

/// 检查文件或目录是否存在（不跟随符号链接）
///
/// # Safety
/// 同 [`post_cur_dir`]。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn post_file_exists(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    guard(json_out, || {
        let req: PathInput = match unsafe { parse_json_input(json_in) } {
            Ok(v) => v,
            Err((code, msg)) => return write_error(json_out, code, &msg),
        };

        match file::exists(&req.path) {
            Ok(exists) => write_success(json_out, "success", json!({ "exists": exists })),
            Err((code, msg)) => write_error(json_out, code, &msg),
        }
    })
}

/// 获取文件或目录的详细信息（大小、类型、修改/创建时间等）
///
/// # Safety
/// 同 [`post_cur_dir`]。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn post_file_info(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    guard(json_out, || {
        let req: PathInput = match unsafe { parse_json_input(json_in) } {
            Ok(v) => v,
            Err((code, msg)) => return write_error(json_out, code, &msg),
        };

        // 使用 symlink_metadata 以正确识别符号链接（不跟随链接）
        match file::symlink_metadata(&req.path) {
            Ok(metadata) => {
                let info = json!({
                    "exists": true,
                    "is_file": metadata.is_file(),
                    "is_dir": metadata.is_dir(),
                    "is_symlink": metadata.is_symlink(),
                    "size": metadata.len(),
                    "modified": timestamp_millis(metadata.modified()),
                    "created": timestamp_millis(metadata.created()),
                });
                write_success(json_out, "success", info)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                write_success(json_out, "success", json!({ "exists": false }))
            }
            Err(e) => write_error(
                json_out,
                errcode::ERR_FILE_INFO,
                &format!("读取文件元数据失败: {}", e),
            ),
        }
    })
}

/// 将 `SystemTime` 转为 Unix 毫秒时间戳；不可用时返回 null
fn timestamp_millis(t: std::io::Result<std::time::SystemTime>) -> Option<u64> {
    t.ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
}
