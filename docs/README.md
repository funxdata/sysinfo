# sys_info 文档

`sys_info` 是一个使用 Rust 编写的系统信息采集与原生对话框库，编译产物以 C ABI（FFI）形式导出，供 Deno、Node.js（`Deno.dlopen` / `ffi-napi` 等）、C/C++ 或其他语言通过动态库直接调用。

- 包名（Cargo）：`sys_info`
- 发布名（dytes）：`sysinfo`
- 版本：`0.1.3`
- 目标平台：Windows / Linux / macOS
- 许可证：见 [LICENSE](../LICENSE)

## 功能概览

| 分类 | 能力 |
| --- | --- |
| 系统 | 系统名称、内核版本、OS 版本、主机名、运行时长、启动时间、负载 |
| 硬件 | CPU、内存与交换分区、磁盘、温度传感器组件 |
| 网络 | 网络接口累计收发字节/包数、错误计数、MAC 地址 |
| 进程 | 分页进程列表（PID、名称、内存、CPU 占用、状态） |
| 文件 | 当前工作目录、创建目录、路径存在性、文件/目录元数据 |
| 交互 | 原生文件选择对话框、原生目录选择对话框 |

## 文档索引

| 文档 | 内容 |
| --- | --- |
| [架构说明](architecture.md) | 目录结构、模块分层、全局状态与线程安全设计 |
| [FFI API](ffi-api.md) | 调用约定、统一响应格式、全部 13 个导出函数及输入输出 |
| [错误码](error-codes.md) | 成功码、输入错误、业务错误与返回码含义 |
| [构建与发布](build-and-release.md) | 本地构建、测试、产物命名、dytes 发布流程与 CI |

## 快速开始

```bash
cargo build --release
cargo test
```

产物位于 `target/release/`：

- Windows：`sys_info.dll`
- Linux：`libsys_info.so`
- macOS：`libsys_info.dylib`

宿主语言加载动态库后，即可按 [FFI API](ffi-api.md) 的约定调用导出函数。例如（伪代码）：

```c
char out[65536];
int rc = post_memory_info(NULL, out);   // rc == 0，out 为 UTF-8 JSON
```

```json
{
  "code": 200,
  "data": {
    "total_bytes": 34359738368,
    "used_bytes": 17179869184,
    "free_bytes": 17179869184,
    "usage_percent": 50.0,
    "total_swap_bytes": 0,
    "used_swap_bytes": 0
  },
  "msg": "success"
}
```
