use std::fs;
use std::path::Path;

/// 获取当前工作目录绝对路径
pub fn current_dir() -> Result<String, String> {
    let dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?;
    Ok(dir.to_string_lossy().into_owned())
}

/// 在当前目录下创建新目录（递归创建，支持多级）
/// 返回新目录的绝对路径
pub fn create_dir(dir_name: &str) -> Result<String, String> {
    if dir_name.trim().is_empty() {
        return Err("目录名不能为空".to_string());
    }

    let mut new_path = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?;
    new_path.push(dir_name);

    fs::create_dir_all(&new_path)
        .map_err(|e| format!("创建目录失败: {}", e))?;

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