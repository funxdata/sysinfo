use std::fs;
use std::path::{Path, PathBuf};

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
/// 仅允许相对路径，且不允许包含 `..`，避免路径穿越。
pub fn create_dir(dir_name: &str) -> Result<String, (i32, String)> {
    let name = dir_name.trim();
    if name.is_empty() {
        return Err((errcode::ERR_EMPTY_FIELD, "目录名不能为空".to_string()));
    }

    let rel = Path::new(name);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err((
            errcode::ERR_INVALID_PATH,
            "dir_name 必须是相对路径且不能包含 '..'".to_string(),
        ));
    }

    let mut new_path: PathBuf = std::env::current_dir()
        .map_err(|e| (errcode::ERR_CURRENT_DIR, format!("获取当前目录失败: {}", e)))?;
    new_path.push(name);

    fs::create_dir_all(&new_path)
        .map_err(|e| (errcode::ERR_CREATE_DIR, format!("创建目录失败: {}", e)))?;

    Ok(new_path.to_string_lossy().into_owned())
}

/// 判断文件或目录是否存在
pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// 获取文件或目录的元数据（大小、类型、时间等）
pub fn stat(path: &str) -> Result<fs::Metadata, String> {
    fs::metadata(path).map_err(|e| e.to_string())
}

/// 获取符号链接自身的元数据（不跟随链接）
pub fn symlink_metadata(path: &str) -> Result<fs::Metadata, String> {
    fs::symlink_metadata(path).map_err(|e| e.to_string())
}
