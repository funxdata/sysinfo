use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::utils::errcode;

/// 获取当前工作目录绝对路径
pub fn current_dir() -> Result<String, (i32, String)> {
    let dir = std::env::current_dir()
        .map_err(|e| (errcode::ERR_CURRENT_DIR, format!("获取当前目录失败: {}", e)))?;
    Ok(dir.to_string_lossy().into_owned())
}

/// 在当前目录下创建新目录（递归创建，支持多级）
/// 返回新目录的绝对路径
///
/// 只允许当前目录下的相对路径：拒绝绝对路径、盘符/UNC 前缀、根目录、`..`
/// 以及 Windows 保留名，避免路径穿越或写到预期之外的位置。
pub fn create_dir(dir_name: &str) -> Result<String, (i32, String)> {
    let name = dir_name.trim();
    if name.is_empty() {
        return Err((errcode::ERR_EMPTY_FIELD, "目录名不能为空".to_string()));
    }

    let rel = Path::new(name);
    let only_relative = rel
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if rel.is_absolute() || !only_relative || has_windows_reserved_component(rel) {
        return Err((
            errcode::ERR_INVALID_PATH,
            "dir_name 必须是当前目录下的相对路径，且不能包含 '..'、盘符或保留名".to_string(),
        ));
    }

    let mut new_path: PathBuf = std::env::current_dir()
        .map_err(|e| (errcode::ERR_CURRENT_DIR, format!("获取当前目录失败: {}", e)))?;
    new_path.push(name);

    fs::create_dir_all(&new_path)
        .map_err(|e| (errcode::ERR_CREATE_DIR, format!("创建目录失败: {}", e)))?;

    Ok(new_path.to_string_lossy().into_owned())
}

/// 判断文件或目录是否存在（不跟随符号链接）
pub fn exists(path: &str) -> Result<bool, (i32, String)> {
    match symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err((errcode::ERR_INVALID_PATH, format!("检查路径失败: {}", e))),
    }
}

/// 获取符号链接自身的元数据（不跟随链接）
pub fn symlink_metadata(path: &str) -> std::io::Result<fs::Metadata> {
    fs::symlink_metadata(path)
}

#[cfg(windows)]
fn has_windows_reserved_component(rel: &Path) -> bool {
    rel.components().any(|c| {
        let Component::Normal(name) = c else {
            return false;
        };
        let name = name.to_string_lossy();
        if name.ends_with('.') || name.ends_with(' ') {
            return true;
        }
        let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
        matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
    })
}

#[cfg(not(windows))]
fn has_windows_reserved_component(_rel: &Path) -> bool {
    false
}
