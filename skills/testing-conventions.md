# 测试规范

## 组织方式

- 集成测试放在 `tests/`，按 FFI 模块划分：
  - `ffi_system.rs`：系统类接口
  - `ffi_sysinfo.rs`：内存、磁盘等数值字段
  - `ffi_file.rs`：文件类接口
  - `ffi_errcode.rs`：输入校验与返回码
  - `ffi_dialog.rs`：对话框（默认 `#[ignore]`）
- 公共辅助放在 `tests/common/mod.rs`，各测试文件通过 `mod common;` 引入。

## 调用方式

- 必须按真实 C ABI 调用导出函数，不要绕过 FFI 直接测 `utils`，以覆盖指针、缓冲区与序列化。
- 使用 `common::call` / `call_no_input` 传输入并解析 JSON 输出。
- 需要检查返回码时使用 `common::call_raw`。
- 输出缓冲区统一使用 `common::new_buf()`，大小为 `MAX_OUTPUT_LEN`。
- 无输入场景显式传 `NULL`，同时保留“传任意合法 JSON”的契约测试。

## 断言要求

- 先断言函数返回码为 `0`，再断言 JSON 的 `code`。
- 成功响应断言 `code == 200`，错误响应断言**具体错误码常量值**（如 `400102`），不要只判断 `code != 200`。
- 数值字段校验合理范围：
  - `used <= total`、`available <= total`；
  - 百分比位于 `0.0..=100.0`；
  - 计数字段与数组长度一致。
- 断言字段类型（`is_u64`、`is_string`、`is_boolean`、`is_array`），避免下游拿到 `null` 或类型漂移。
- 不依赖本机具体值（主机名、进程数量、磁盘名称），只断言结构与范围。

## 临时资源

- 创建目录使用 `common::unique_name` 生成唯一名称，避免并行测试冲突。
- 测试结束必须清理临时目录 / 文件；清理失败可忽略（`let _ = remove_dir_all(...)`）。
- 不要在仓库内写入持久文件，不要依赖用户目录中的既有文件。
- 验证 `json_out == NULL` 等返回码时优先选择无副作用的接口；必须调用 `post_create_dir` 这类有副作用的接口时，要使用唯一名称并在测试结束清理，避免在仓库根留下目录。
- 路径拼接与 JSON 转义使用 `common::path_input` / `dir_name_input`，不要手写字符串。

## 对话框测试

- 需要人工交互的测试必须标记 `#[ignore = "需要人工交互：..."]`，避免 CI 阻塞。
- 手动运行：

```bash
cargo test --test ffi_dialog -- --ignored --nocapture
```

- 断言响应结构合法即可，不要强制用户必须选择文件（取消也是合法路径）。

## dytes 包自检

发布前除 `cargo test` 外，还需按 dytes 的测试环境规范验证包清单与安装布局：

- 运行 `cd ts && deno task check`（构建 + `dytes test --keep` + Deno FFI 测试）；
- 测试产物只写入 `.dytes/`，该目录已加入 `.gitignore`，禁止提交；
- 当前包没有 dytes 依赖，不会访问 S3；若以后新增 `dependencies-*` / `test-dependency`，依赖会在线下载到 `.dytes/env/`，失败即中止；
- 使用 `--json` 时，机器可读结果在 stdout 最后一行；
- 退出码：`0` 通过、`1` 依赖/IO 失败、`2` 参数错误、`3` 清单/导出/校验失败。

## 提交前验证

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- 新增功能必须带测试；修复缺陷应先补能复现的用例。
- 测试名称使用 `snake_case` 描述行为，例如 `create_dir_traversal`、`memory_info_fields`。
- 测试中允许 `unwrap`（失败即断言失败），生产代码不允许。
