# 版本与发布规范

## 版本同步

发布前以下位置必须一致：

1. `Cargo.toml` 的 `[package].version`
2. `dytes.json` 的 `version`
3. `docs/README.md` 中标注的版本
4. `Cargo.lock` 中 `sys_info` 的版本（执行 `cargo build` 自动更新）

CI 会在 tag 触发时校验 `v<tag>` 与 `dytes.json` 的 `version`，不一致直接失败。

## 提交信息

沿用仓库现有的 Conventional Commits 风格（中文描述）：

```text
feat: 增加磁盘 IO 统计
fix: 修复进程名截断越界
refactor: 优化状态锁与内存采集
docs: 补充 FFI 字段说明
test: 补充路径穿越用例
chore: 发布 0.1.3
ci: 升级 dytes Action
```

- 一次提交只做一件事；格式化、重构、功能不要混在一起。
- 涉及接口行为变化时，在提交正文说明影响与迁移方式。
- 不提交 `target/`、临时探针、编辑器配置等无关文件。

## 发版流程

1. 确认 `cargo test` 全绿，工作区无未提交改动。
2. 同步更新版本号（见上）并提交：`chore: 发布 x.y.z`。
3. 打标签并推送：

```bash
git tag vX.Y.Z
git push origin master --tags
```

4. GitHub Actions（`.github/workflows/dytes-release.yml`）自动：
   - 校验 tag 与 `dytes.json` 版本；
   - 在 Windows / Linux / macOS 上执行 `cargo build --release`；
   - 调用 `dytes publish` 上传产物。

5. 发布后在 `docs/README.md` 记录版本变更要点（如行为变化、平台限制）。

## 产物命名

| 平台 | 本地产物 | 发布名 |
| --- | --- | --- |
| Windows | `target/release/sys_info.dll` | `sysinfo.dll` |
| Linux | `target/release/libsys_info.so` | `sysinfo.so` |
| macOS | `target/release/libsys_info.dylib` | `sysinfo.dylib` |

- 修改 `Cargo.toml` 的 `[lib].name` 或 `platform-exports` 时，必须同步 `dytes.json` 与本文档。
- 不要手动重命名产物后发布，发布映射由 `dytes.json` 负责。

## 发布检查清单

- [ ] `Cargo.toml`、`dytes.json`、`docs/README.md` 版本一致
- [ ] `cargo fmt --check` 通过
- [ ] `cargo clippy --all-targets -- -D warnings` 通过
- [ ] `cargo test` 全绿（对话框用例保持 `ignored`）
- [ ] `cd ts && deno task check` 通过，`.dytes/` 未提交
- [ ] `docs/ffi-api.md` 与实际输出字段一致
- [ ] Linux 系统依赖无新增（`rfd` 相关库已在 CI 安装）
- [ ] tag 名符合 `vX.Y.Z`
