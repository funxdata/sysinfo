# 代码优化问题清单

本目录只保留**尚未修复**的代码优化项。已在 2026-11-17 修复的条目已从目录中删除，不再保留详情。

- 扫描/修复日期：2026-11-17
- 工作区：`D:\codes\github\funxdata\sys\sysinfo`
- 当前版本：`sys_info 0.1.3`
- 本次已修复并验证：FFI panic 防护、裸指针 `unsafe` 契约、路径校验（盘符/UNC/`..`/Windows 保留名）、进程名截断、`exists`/`file_info` 语义、MAC/温度字段、fmt/clippy、死代码与依赖清理、Cargo 元数据、CI 质量门禁、缓冲区边界测试

## 优先级

| 级别 | 判定 |
| --- | --- |
| P1 | 可能导致宿主进程崩溃或绕过路径安全边界，应优先处理 |
| P2 | 接口语义不一致、兼容性风险或质量门禁缺失 |
| P3 | 可维护性、性能、死代码与元数据清理 |

状态说明：本目录全部条目初始为“待修”。修复后请删除对应条目，而不是长期保留已完成内容。

## 未修复项

| 编号 | 标题 | 优先级 | 位置 | 状态 |
| --- | --- | --- | --- | --- |
| [018](performance.md) | 进程分页每次全量刷新并排序 | P3 | `src/utils/process.rs` | 待修 |
| [019](performance.md) | CPU 首次采样不可靠（可能 100%） | P3 | `src/utils/state.rs`、`src/utils/cpu.rs` | 待修 |

## 复现扫描

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ts && deno task check
```

当前上述命令均通过；018/019 属于行为与性能优化，不阻塞发布。
