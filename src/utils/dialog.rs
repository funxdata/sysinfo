use std::path::PathBuf;

/// 弹出系统原生文件选择对话框，返回用户选择的文件路径。
/// 用户取消时返回 `None`。
pub fn choose_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择文件")
        .pick_file()
}

/// 弹出系统原生目录选择对话框，返回用户选择的目录路径。
/// 用户取消时返回 `None`。
pub fn choose_dir() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("选择目录")
        .pick_folder()
}

