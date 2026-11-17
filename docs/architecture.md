# 架构说明

## 目录结构

```text
.
├── Cargo.toml              # 包定义：cdylib + rlib
├── dytes.json              # dytes 发布元数据与平台产物映射
├── src/
│   ├── lib.rs              # 模块声明 + 导出函数 re-export
│   ├── ffi/                # C ABI 边界：解析输入、序列化输出
│   │   ├── file.rs         # 文件类 FFI
│   │   ├── sys.rs          # 系统类 FFI
│   │   └── dialog.rs       # 对话框类 FFI
│   └── utils/              # 纯 Rust 实现
│       ├── cjson.rs        # JSON 输入解析、输出写入、统一响应
│       ├── errcode.rs      # 错误码常量
│       ├── state.rs        # 全局 sysinfo 实例（Mutex）
│       ├── system.rs       # 系统基础信息
│       ├── cpu.rs          # CPU 信息
│       ├── memory.rs       # 内存信息
│       ├── disk.rs         # 磁盘信息
│       ├── network.rs      # 网络接口信息
│       ├── process.rs      # 进程列表（分页）
│       ├── component.rs    # 温度传感器组件
│       ├── file.rs         # 文件系统操作
│       └── dialog.rs       # rfd 原生对话框
└── tests/                  # FFI 集成测试
    ├── common/mod.rs       # 调用辅助（输入 CString、输出缓冲区）
    ├── ffi_dialog.rs       # 对话框（默认 ignore，需人工交互）
    ├── ffi_errcode.rs      # 错误码与非法输入
    ├── ffi_file.rs         # 文件类接口
    ├── ffi_sysinfo.rs      # 内存与磁盘字段校验
    └── ffi_system.rs       # 系统类接口冒烟测试
```

## 分层设计

```text
调用方（Deno / Node / C ...）
        │  dlopen + C ABI
        ▼
src/ffi/*        —— 只负责：解析 json_in、调用 utils、把结果写入 json_out
        ▼
src/utils/*      —— 纯 Rust 业务实现，返回 Result<serde_json::Value, (i32, String)>
        ▼
sysinfo / rfd / std::fs
```

- **边界层 `ffi`**：每个函数签名统一为 `extern "C" fn(json_in: *const c_char, json_out: *mut c_char) -> c_int`。无输入的函数允许 `json_in == null`；有输入的函数通过 `serde` 反序列化，失败时返回统一错误 JSON。
- **实现层 `utils`**：不接触裸指针，方便单元测试与复用；采集函数统一返回 `Result<serde_json::Value, (错误码, 错误信息)>`。
- **输出层 `cjson`**：`write_success` / `write_error` 负责构造 `{code, data, msg}` 并写入调用方缓冲区。
- `lib.rs` 将全部 FFI 函数 `pub use` 到 crate 根，测试与调用方都以 `post_*` 命名访问。

## 全局状态与线程安全

`src/utils/state.rs` 维护四个 `static Mutex<Option<...>>`：

| 静态变量 | 类型 | 初始化方式 |
| --- | --- | --- |
| `SYSTEM` | `Mutex<Option<System>>` | 首次使用时 `System::new()` |
| `DISKS` | `Mutex<Option<Disks>>` | `Disks::new_with_refreshed_list()` |
| `NETWORKS` | `Mutex<Option<Networks>>` | `Networks::new_with_refreshed_list()` |
| `COMPONENTS` | `Mutex<Option<Components>>` | `Components::new_with_refreshed_list()` |

设计原因：

1. **CPU 使用率等指标依赖两次采样差值**，必须保留同一个 `sysinfo` 实例，不能每次调用重新创建。
2. **进程列表首次刷新开销大**，复用实例可降低后续调用的成本。
3. 每个实例由独立互斥锁保护，`with_system` / `with_disks` / `with_networks` / `with_components` 在锁内执行闭包；锁中毒时返回业务错误码 `400999`。

注意：

- 这些接口是**进程内全局单例**，同一类采集在多个线程间串行执行（不同类别互不阻塞）。
- 每次采集都会在锁内调用 `refresh*`，因此重复调用可能带来一定耗时；调用方应避免高频轮询。
- 导出函数通过 `#[unsafe(no_mangle)]` 保持 C 符号名，编译为 `cdylib` 后即为动态库导出符号。

## 兼容性

- Rust edition 2021，需支持 `unsafe(no_mangle)` 语法的较新工具链（仓库使用 stable 1.96）。
- 依赖：`sysinfo 0.39`、`serde` / `serde_json`、`rfd 0.17.2`、`libc`。
- `rfd` 在 Linux 下需要 GTK/Wayland/X11 及 D-Bus 相关系统库，详见 [构建与发布](build-and-release.md)。
