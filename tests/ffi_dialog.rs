//! 文件/目录选择对话框的 FFI 测试
//!
//! 对话框需要人工交互，默认标记为 `#[ignore]`，避免 CI 中阻塞。
//! 手动运行方式：
//! ```bash
//! cargo test --test ffi_dialog -- --ignored --nocapture
//! ```

mod common;

use common::{call_no_input, new_buf, read_buf};

use sys_info::{post_choose_dir, post_choose_file};

/// 断言响应体是合法的统一结构：code == 200，data.path 为 string 或 null
fn assert_valid_dialog_response(v: &serde_json::Value) {
    assert_eq!(v["code"], 200, "code 应为 200");
    assert!(
        v["msg"].is_string(),
        "msg 应为字符串，实际：{}",
        v["msg"]
    );
    let path = &v["data"]["path"];
    assert!(
        path.is_string() || path.is_null(),
        "data.path 应为 string 或 null，实际：{}",
        path
    );
    // 若为字符串，必须是绝对路径
    if let Some(p) = path.as_str() {
        assert!(
            std::path::Path::new(p).is_absolute(),
            "选择结果应为绝对路径，实际：{}",
            p
        );
    }
}

#[test]
#[ignore = "需要人工交互：弹出文件选择对话框"]
fn manual_choose_file() {
    println!("请在弹出的对话框中选择一个文件（或取消）...");
    let v = call_no_input(post_choose_file);
    println!("响应：{}", serde_json::to_string_pretty(&v).unwrap());
    assert_valid_dialog_response(&v);

    if v["data"]["path"].is_string() {
        let p = v["data"]["path"].as_str().unwrap();
        assert!(std::path::Path::new(p).exists(), "选择的文件应存在");
        println!("已选择文件：{}", p);
    } else {
        println!("用户取消了文件选择");
    }
}

#[test]
#[ignore = "需要人工交互：弹出目录选择对话框"]
fn manual_choose_dir() {
    println!("请在弹出的对话框中选择一个目录（或取消）...");
    let v = call_no_input(post_choose_dir);
    println!("响应：{}", serde_json::to_string_pretty(&v).unwrap());
    assert_valid_dialog_response(&v);

    if v["data"]["path"].is_string() {
        let p = v["data"]["path"].as_str().unwrap();
        assert!(std::path::Path::new(p).is_dir(), "选择的路径应为目录");
        println!("已选择目录：{}", p);
    } else {
        println!("用户取消了目录选择");
    }
}

/// 契约测试：即使调用方传入任意 JSON，函数也应正常返回合法结构
/// （对话框仍会弹出，因此同样标记 ignore）
#[test]
#[ignore = "需要人工交互：会弹出对话框"]
fn choose_file_with_arbitrary_json_input() {
    let v = common::call(post_choose_file, Some(r#"{"whatever":"x"}"#));
    assert_valid_dialog_response(&v);
}

#[test]
#[ignore = "需要人工交互：会弹出对话框"]
fn choose_dir_with_arbitrary_json_input() {
    let v = common::call(post_choose_dir, Some(r#"{"foo":123}"#));
    assert_valid_dialog_response(&v);
}

/// 取消场景的显式验证（需要人工点击取消）
#[test]
#[ignore = "需要人工交互：请在对话框弹出时点击取消"]
fn choose_file_cancelled_returns_null_path() {
    let v = call_no_input(post_choose_file);
    assert_eq!(v["code"], 200);
    if v["data"]["path"].is_null() {
        assert_eq!(v["msg"], "cancelled", "取消时 msg 应为 cancelled");
    } else {
        println!("未取消，而是选择了：{}", v["data"]["path"]);
    }
}
