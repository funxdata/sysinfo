# Rust 编码规范

## 模块分层

| 目录 | 职责 | 禁止 |
| --- | --- | --- |
| `src/ffi/` | C ABI 函数、JSON 输入解析、调用 `utils`、写输出 | 承载业务逻辑、直接操作 `sysinfo` / `std::fs` |
| `src/utils/` | 纯 Rust 业务实现 | 裸指针、`CStr`、直接写 `json_out` |
| `src/utils/state.rs` | 全局采集实例的唯一入口 | 在其他模块直接 `lock()` 或创建新的 `System` |
| `src/lib.rs` | 模块声明与 FFI 符号 re-export | 业务实现 |

新增模块时同步更新 `src/ffi/mod.rs` 或 `src/utils/mod.rs`，并在 `src/lib.rs` 中 `pub use` 新的导出函数。

## 文件与函数组织

- 一个文件聚焦一类能力（CPU、内存、磁盘……），不要混入无关功能。
- 采集函数命名为 `get_*_info`，文件操作使用动词短语（`current_dir`、`create_dir`、`exists`）。
- FFI 导出函数统一以 `post_` 开头，与 `src/lib.rs` 的 re-export 和文档保持一致。
- 常量放在使用处或 `errcode.rs`；不要散落魔法数字。

## 错误处理

- `utils` 层统一返回 `Result<T, (i32, String)>`，第一项是 `errcode.rs` 中的常量，第二项是可读的中文/英文错误信息。
- 禁止在业务代码中 `unwrap()` / `expect()` / `panic!()`；可恢复错误向上返回。
- 锁中毒映射为 `ERR_INTERNAL_LOCK`（`400999`），不要静默忽略。
- 错误信息保留底层原因（如 `format!("创建目录失败: {e}")`），便于调用方排查。
- 新增错误场景时先在 `src/utils/errcode.rs` 定义常量，再在代码和 `docs/error-codes.md` 中使用。

## 命名与数据格式

- Rust 标识符使用 `snake_case`；结构体与枚举使用 `PascalCase`。
- 输出 JSON 字段统一 `snake_case`，避免缩写。
- 容量字段必须以 `_bytes` 结尾；时间戳区分 `_time`（Unix 秒）与毫秒字段（如 `modified`），并在文档中写明单位。
- 百分比字段以 `_percent` 结尾，范围 `0.0..=100.0`，保留两位小数。
- 可选/不可用值优先用 `null`；底层库使用哨兵值时（例如温度 `-273.15`），应在采集层归一化为 `null`，避免调用方自行过滤。

## 注释与文档

- 对外函数写 `///` 文档注释，说明输入、输出和特殊行为（取消、空结果、平台差异）。
- 复杂逻辑（分页、锁、路径安全校验）写简短注释解释“为什么”，不要复述代码。
- 修改接口行为时同步更新 `docs/ffi-api.md` 与 `docs/error-codes.md`，不要只改代码。

## unsafe 与内存安全

- `unsafe` 范围尽可能小，并在注释中说明前置条件（指针非空、以 `\0` 结尾、缓冲区长度等）。
- FFI 函数使用 `#[unsafe(no_mangle)]` 保持导出符号名，不要改用旧式 `#[no_mangle]`。
- 处理调用方字符串时必须用 `CStr` 转换并校验 UTF-8；不要假设指针有效。
- 截断用户可控字符串（如进程名）时按字符边界处理，不能直接按字节 `truncate`，以免在多字节字符中间 panic。
- 输出写入前必须检查长度（`MAX_OUTPUT_LEN`，含结尾 `\0`）。

## 全局状态

- 只能通过 `state::with_system` / `with_disks` / `with_networks` / `with_components` 访问采集实例。
- 闭包内完成刷新与数据提取，避免把锁守卫或引用带到锁外。
- 保持实例复用：CPU 使用率、网络速率等依赖两次采样差值；不要每次调用重新 `new()`。
- 已知限制（采集在锁内串行、首次 CPU 采样不可靠）应通过文档告知调用方，而不是隐藏。

## 依赖与工具链

- 当前依赖：`sysinfo 0.39`、`serde`、`serde_json`、`rfd 0.17.2`、`libc`。
- 新增依赖前先评估体积、平台支持（Windows/Linux/macOS）与系统库要求；优先复用已有依赖。
- 使用 stable 工具链，保持 edition 2021 与现有 `#[unsafe(no_mangle)]` 语法兼容。
- 提交前至少执行 `cargo fmt` 与 `cargo test`；建议再跑 `cargo clippy --all-targets`。
