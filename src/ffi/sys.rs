use std::ffi::{c_char, c_int};

use crate::utils::cjson::{parse_json_input, write_error, write_success};
use crate::utils::process::{self, PageInput};
use crate::utils::{component, cpu, disk, memory, network, system};

/// 系统基础信息：名称、内核版本、OS 版本、主机名、运行时长、启动时间、负载
#[unsafe(no_mangle)]
pub extern "C" fn post_system_basic_info(
    _json_in: *const c_char,
    json_out: *mut c_char,
) -> c_int {
    match system::get_basic_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// 内存信息
#[unsafe(no_mangle)]
pub extern "C" fn post_memory_info(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match memory::get_memory_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// CPU 信息
#[unsafe(no_mangle)]
pub extern "C" fn post_cpu_info(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match cpu::get_cpu_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// 磁盘信息
#[unsafe(no_mangle)]
pub extern "C" fn post_disk_info(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match disk::get_disk_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// 网络接口信息
#[unsafe(no_mangle)]
pub extern "C" fn post_network_info(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match network::get_network_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// 进程列表（分页）
///
/// 输入（可选，json_in 可为 null）：
/// `{ "limit": 100, "offset": 0 }`
///
/// 输出：
/// ```json
/// {
///   "total": 342,
///   "offset": 0,
///   "limit": 100,
///   "count": 100,
///   "has_more": true,
///   "processes": [ { "pid": 1, "name": "systemd", "memory": ..., "cpu_usage": ..., "status": "..." } ]
/// }
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn post_process_list(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let page: PageInput = if json_in.is_null() {
        PageInput {
            limit: None,
            offset: None,
        }
    } else {
        match unsafe { parse_json_input(json_in) } {
            Ok(v) => v,
            Err((code, msg)) => return write_error(json_out, code, &msg),
        }
    };

    match process::get_process_list(page) {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}

/// 温度传感器组件信息
#[unsafe(no_mangle)]
pub extern "C" fn post_component_info(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    match component::get_component_info() {
        Ok(data) => write_success(json_out, "success", data),
        Err((code, msg)) => write_error(json_out, code, &msg),
    }
}
