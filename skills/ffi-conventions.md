# FFI 接口规范

## 函数签名

所有导出函数必须使用同一签名，并通过 `#[unsafe(no_mangle)]` 导出：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn post_xxx(json_in: *const c_char, json_out: *mut c_char) -> c_int
```

- 不需要输入的函数命名为 `_json_in` 表示忽略；需要输入的函数按需解析。
- 函数必须 `pub`，并在 `src/lib.rs` 中 `pub use`，保证测试和调用方都能访问。
- 不允许返回 Rust 引用、`String` 或 `Result`，跨边界只传 C 类型。

## 输入约定

- 输入是 UTF-8、以 `\0` 结尾的 JSON 字符串。
- 必填输入使用 `serde` 反序列化到专用结构体：

```rust
#[derive(Deserialize)]
struct Input {
    path: String,
}
```

- `json_in` 为 `NULL` 且接口需要输入时，返回 `ERR_JSON_IN_NULL`（`400100`）。
- JSON 非法、缺少字段、类型不匹配统一返回 `ERR_INVALID_JSON`（`400102`），错误信息带上解析原因。
- 可选输入（如分页）显式判断 `json_in.is_null()`，不要用空字符串代替。
- 字段级校验失败使用专门的错误码：空值 `ERR_EMPTY_FIELD`，非法路径 `ERR_INVALID_PATH`。
- 路径类输入必须校验安全边界：只允许当前目录下的相对路径，拒绝绝对路径、盘符/UNC 前缀、根目录、`..` 与 Windows 保留名；不能直接拼接后使用。

## 输出约定

- 只能通过 `write_success` / `write_error` 写输出，禁止手工拼接 JSON 字符串。
- 成功响应固定 `{ "code": 200, "data": <object>, "msg": "success" }`。
- 失败响应固定 `{ "code": <errcode>, "data": null, "msg": <信息> }`。
- 业务上的“无数据”仍使用成功响应，例如空数组、`exists: false`、对话框取消。
- 缓冲区大小固定为 `MAX_OUTPUT_LEN`（65536 字节，含结尾 `\0`）；超长时 `write_json_out` 返回 `-2`。
- `data` 必须是对象或数组，字段命名遵循 [Rust 编码规范](rust-conventions.md)。

## 返回值

| 返回值 | 含义 |
| --- | --- |
| `0` | 已写入 JSON，业务结果看 `code` |
| `-1` | `json_out` 为 `NULL`，未写入 |
| `-2` | 输出超过缓冲区，未写入 |

不要把业务错误码作为函数返回值。

## 新增 FFI 接口清单

按顺序完成，缺一不可：

1. **错误码**：如有新错误场景，先在 `src/utils/errcode.rs` 增加常量。
2. **实现**：在 `src/utils/<模块>.rs` 写纯 Rust 逻辑，返回 `Result<serde_json::Value, (i32, String)>`。
3. **边界**：在 `src/ffi/<模块>.rs` 写 `post_*` 包装，解析输入、调用实现、写输出。
4. **模块声明**：必要时更新 `src/ffi/mod.rs` / `src/utils/mod.rs`。
5. **导出**：在 `src/lib.rs` 增加 `pub use`，命名与 `post_*` 一致。
6. **测试**：在 `tests/ffi_<模块>.rs` 增加成功路径与错误路径用例；资源需清理。
7. **文档**：更新 `docs/ffi-api.md`（输入、输出、单位、特殊行为）和 `docs/error-codes.md`（如新增错误码）。
8. **验证**：`cargo test` 全绿；确认无新增编译警告。

## 兼容性要求

- 已发布字段只增不删、不改类型、不改单位。
- 新增可选字段必须保证旧调用方忽略后仍能正常工作。
- 修改语义（例如时间戳单位从秒改为毫秒）属于破坏性变更，必须提升次版本并同步文档。
- 对话框等阻塞接口应保持“取消返回 `code: 200` + `path: null`”的既有语义。
