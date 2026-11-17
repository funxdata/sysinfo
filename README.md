# sys_info

Rust 编写的系统信息采集与原生文件对话框库，编译为动态库（C ABI），供 Deno / Node.js / C 调用。

- 包名 `sys_info`，发布名 `sysinfo`
- 平台：Windows / Linux / macOS
- 版本：`0.1.3`

## 快速开始

```bash
cargo build --release
cargo test
cd ts && deno task check   # Deno FFI 测试
```

产物位于 `target/release/`：

| 平台 | 本地产物 | 发布名 |
| --- | --- | --- |
| Windows | `sys_info.dll` | `sysinfo.dll` |
| Linux | `libsys_info.so` | `sysinfo.so` |
| macOS | `libsys_info.dylib` | `sysinfo.dylib` |

## 文档

- [项目文档](docs/README.md)
- [FFI API](docs/ffi-api.md)
- [开发规范](skills/README.md)
- [待优化项](issues/README.md)

## License

见 [LICENSE](LICENSE)。
