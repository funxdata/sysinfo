# 构建与发布

## 环境要求

- Rust stable（仓库当前使用 1.96，需支持 `unsafe(no_mangle)` 语法）
- Cargo
- Linux 额外系统依赖（`rfd` 需要 GTK/Wayland/X11 与 D-Bus）：

```bash
sudo apt-get update
sudo apt-get install -y cmake ninja-build pkg-config clang libclang-dev \
  libwayland-dev libxkbcommon-dev libdbus-1-dev
```

- macOS 额外依赖：

```bash
brew install cmake ninja pkg-config
```

## 本地构建

```bash
cargo build            # debug
cargo build --release  # release
```

`Cargo.toml` 将 crate 同时编译为 `cdylib` 与 `rlib`：

| 平台 | 产物 |
| --- | --- |
| Windows | `target/release/sys_info.dll` |
| Linux | `target/release/libsys_info.so` |
| macOS | `target/release/libsys_info.dylib` |

## 测试

```bash
cargo test
```

测试按文件划分：

| 测试文件 | 覆盖内容 |
| --- | --- |
| `tests/ffi_system.rs` | 系统类接口冒烟测试 |
| `tests/ffi_sysinfo.rs` | 内存 / 磁盘字段与数值范围 |
| `tests/ffi_file.rs` | 目录创建、存在性、文件元数据 |
| `tests/ffi_errcode.rs` | 空输入、非法 UTF-8、缺字段、空输出缓冲区 |
| `tests/ffi_dialog.rs` | 文件/目录对话框（默认 `#[ignore]`） |

对话框测试需要人工点击，手动运行：

```bash
cargo test --test ffi_dialog -- --ignored --nocapture
```

测试通过 `tests/common/mod.rs` 直接按 C ABI 调用导出函数，并使用 65536 字节输出缓冲区，等同于真实调用方的行为。

### Deno FFI 测试

`ts/` 目录提供 Deno 版本的 FFI 测试：通过 `Deno.dlopen` 加载动态库，覆盖系统、进程、文件与错误码接口。动态库优先取 `dytes test --keep` 的演练产物（`.dytes/run-<pid>/dytes-test-<ts>/libary/`），其次取 `SYSINFO_LIB`，最后回退 `target/release/`。

```bash
cd ts
deno task check
```

`check` 依次执行构建、`dytes test --keep` 和 `deno test`；完整流程需要 Deno 2.x，测试权限为 `--allow-ffi --allow-read --allow-write --allow-env`。

## dytes test 环境

发布前用 `dytes test` 校验包清单、平台导出与安装落盘布局，遵循 dytes 的测试环境规范：

- 所有测试产物放在 `.dytes/`，该目录已加入 `.gitignore`，不提交；
- `dytes test` 自动创建 `.dytes/env/` 与 `.dytes/run-<pid>/` 演练目录，演练目录默认在结束后删除；
- 当前包使用 `target/release/` 下的本地构建产物，不下载自身；
- 当前包没有 `dependencies-*` / `test-dependency`，因此不会访问 S3；
- 环境隔离：不读写用户真实配置目录（`%APPDATA%\dytes`、`~/.config/dytes`），只使用 `.dytes/`。

在 `ts/` 目录下用 Deno task 一键执行（要求 dytes >= 0.2.2，`dytes test` 从该版本开始提供）：

```bash
cd ts
deno task check
```

`check` 依次执行：

1. `deno task build`：`cargo build --release`；
2. `deno task dytes:test`：`dytes test --keep --json`，保留演练目录供 Deno 测试加载；
3. `deno task test`：Deno FFI 测试。

只做包自检：`deno task dytes:test`；只跑 FFI 测试：`deno task test`。`dytes test` 常用参数：

| 参数 | 说明 |
| --- | --- |
| `--json` | 机器可读输出；有依赖下载时结果在 stdout 最后一行 |
| `--keep` | 保留演练目录，便于排查和加载动态库 |
| `--platform <platform>` | 指定 `windows` / `linux` / `macos` 的导出 |

退出码：`0` 通过；`1` 依赖下载/IO 失败；`2` 参数错误；`3` 清单、导出、路径安全或锁文件校验失败。

`.dytes/` 是生成目录，可随时删除，下次 `dytes test` 自动重建。

## dytes 发布

`dytes.json` 定义发布名、包 UUID、版本与各平台产物映射：

```json
{
  "name": "sysinfo",
  "uuid": "86a6db59-026d-4dfd-a138-a5708cfa141b",
  "version": "0.1.3",
  "type": "sys",
  "platform-exports": {
    "windows": { "./target/release/sys_info.dll": "sysinfo.dll" },
    "linux": { "./target/release/libsys_info.so": "sysinfo.so" },
    "macos": { "./target/release/libsys_info.dylib": "sysinfo.dylib" }
  }
}
```

发布后宿主侧看到的文件名统一为 `sysinfo.dll` / `sysinfo.so` / `sysinfo.dylib`。

### CI 流程

工作流：`.github/workflows/dytes-release.yml`

- 触发条件：推送 `v*` 标签，或手动 `workflow_dispatch`。
- `prepare` 任务：
  - 读取 `dytes.json` 的 `version`；
  - 若由标签触发，校验 `v<tag>` 与版本一致，不一致直接失败；
  - 从 `platform-exports` 的键生成构建矩阵；
  - 依据仓库文件自动判断构建方式（`CMakeLists.txt` → cmake，`Cargo.toml` → cargo，否则 deno）。
- `publish` 任务按平台串行执行（`max-parallel: 1`）：
  - 安装 Rust 与平台依赖，执行 `cargo build --release`；
  - 执行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`（Windows）与 `cargo test`（三平台）；
  - 下载指定版本（当前 `0.2.3`）的 `dytes` 可执行文件：优先从 `packages.funxdata.com` CDN 下载，失败后回退 GitHub Releases；
  - 发布前执行 `dytes test --json` 作为门禁，校验失败即中止；
  - 执行 `dytes publish` 上传产物。

### 发版步骤

1. 修改 `Cargo.toml` 与 `dytes.json` 中的版本号，保持一致。
2. 本地执行 `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`，再执行 `cd ts && deno task check` 确认包自检与 FFI 测试通过。
3. 提交变更并打标签：`git tag v0.1.3 && git push origin v0.1.3`。
4. CI 自动执行 fmt/clippy/test、构建三平台产物、运行 `dytes test` 门禁并发布。

> 标签版本与 `dytes.json` 不一致会导致 `prepare` 失败。
