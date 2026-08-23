
use std::ffi::{c_char, c_int};
use serde::Deserialize;
use serde_json::json;

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::file;

/// 获取当前工作目录
#[unsafe(no_mangle)]
pub extern "C" fn post_cur_dir(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    // 此接口不需要输入参数，但为了统一，可接受空 JSON 或任意 JSON
    // 这里不解析输入，直接忽略，但为了保持一致性，可简单解析
    let _: serde_json::Value = match parse_json_input(json_in) {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    match file::current_dir() {
        Ok(path) => write_success(json_out, "success", json!({ "path": path })),
        Err(e) => write_error(json_out, 400900, &e),
    }
}

/// 在当前目录下创建新目录
#[unsafe(no_mangle)]
pub extern "C" fn post_create_dir(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    #[derive(Deserialize)]
    struct Input {
        dir_name: String,
    }

    let req: Input = match parse_json_input(json_in) {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    if req.dir_name.trim().is_empty() {
        return write_error(json_out, 400103, "dir_name is empty");
    }

    match file::create_dir(&req.dir_name) {
        Ok(path) => write_success(json_out, "success", json!({ "path": path })),
        Err(e) => write_error(json_out, 400901, &e),
    }
}

// src/ffi/file.rs

/// 检查文件或目录是否存在
#[unsafe(no_mangle)]
pub extern "C" fn post_file_exists(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    #[derive(Deserialize)]
    struct Input {
        path: String,
    }

    let req: Input = match parse_json_input(json_in) {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    let exists = file::exists(&req.path);
    write_success(json_out, "success", json!({ "exists": exists }))
}

/// 获取文件或目录的详细信息（大小、类型、修改/创建时间等）
#[unsafe(no_mangle)]
pub extern "C" fn post_file_info(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    #[derive(Deserialize)]
    struct Input {
        path: String,
    }

    let req: Input = match parse_json_input(json_in) {
        Ok(v) => v,
        Err((code, msg)) => return write_error(json_out, code, &msg),
    };

    match file::stat(&req.path) {
        Ok(metadata) => {
            let info = json!({
                "exists": true,
                "is_file": metadata.is_file(),
                "is_dir": metadata.is_dir(),
                "is_symlink": metadata.is_symlink(),
                "size": metadata.len(),
                "modified": metadata.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                "created": metadata.created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
            write_success(json_out, "success", info)
        }
        Err(e) => {
            // 文件不存在或无法读取元数据，返回 exists: false
            write_success(json_out, "success", json!({ "exists": false, "error": e }))
        }
    }
}