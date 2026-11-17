# 性能与资源

## ISSUE-018 进程分页每次全量刷新并排序

- 状态：待修 ｜ 优先级：P3
- 位置：`src/utils/process.rs`
- 类型：性能

**现象**

每次调用 `post_process_list` 都会：

```rust
sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
let all = sys.processes();
let total = all.len();
let mut entries: Vec<_> = all.iter().collect();
entries.sort_by_key(|(pid, _)| pid.as_u32());
```

即使只请求 `limit: 5, offset: 0`，也要刷新并排序全部进程。分页本身是输出层逻辑，不能减少底层采集成本。

**影响**

- 高频轮询（如前端每秒刷新进程列表）会持续占用 CPU；
- 进程数多（数百到数千）时，排序和 JSON 序列化成本明显；
- 与 `state.rs` 的全局锁叠加，会阻塞同一时刻的其他 `System` 采集（内存、CPU）。

**修复建议**

可选方案：

1. 增加进程列表缓存：保存 `(刷新时间, 排序后的 Vec<pid>)`，在 TTL（如 1 秒）内复用；调用方显式传 `refresh: true` 时强制刷新；
2. 为 `post_process_list` 增加 `refresh` 参数（默认 `true` 保持兼容），允许分页翻页时复用上一页的刷新结果；
3. 若只需要“稳定的分页”，可在刷新后按 pid 排序并缓存，`offset` 只影响切片；
4. 评估 `sysinfo` 是否支持只刷新指定 PID / 增量刷新，减少全量扫描。

注意：缓存会引入数据陈旧问题，必须在 `docs/ffi-api.md` 说明 TTL 与刷新语义。

**验收**

- 连续分页请求（同一刷新周期）不再重复全量刷新；
- 进程列表内容与现有测试一致；
- 文档说明缓存/TTL/刷新参数；
- 用基准或日志验证刷新次数下降。

---

## ISSUE-019 CPU 首次采样不可靠

- 状态：待修 ｜ 优先级：P3
- 位置：`src/utils/state.rs`、`src/utils/cpu.rs`
- 类型：数据准确性 / API 语义

**现象**

`with_system` 首次使用 `System::new()` 初始化，随后 `post_cpu_info` 调用 `refresh_cpu_all()`。由于没有更早的采样基准，首次 `cpu_usage()` / `global_cpu_usage()` 可能返回 `100.0`（Windows 实测）或 `0.0`，不能反映真实占用。

文档已在 `docs/ffi-api.md` 说明“调用方应忽略首次结果”，但 API 本身没有区分“有效采样”和“首次采样”。

**影响**

- 调用方如果直接展示首次结果，会看到 100% 的错误读数；
- 不同平台首次值不同，难以统一处理。

**修复建议**

1. 在 `System` 初始化时立即执行一次 `refresh_cpu_all()` 并记录时间戳；后续调用距离首次刷新超过最小间隔（如 200ms）再返回 usage；
2. 为 `post_cpu_info` 的 `usage` / `global_usage` 返回 `null`（或增加 `sampled: false`），直到完成第二次采样；
3. 至少保留现有文档说明，并在 Deno/Rust 测试中忽略首次值。

方案 1 不改变 JSON 结构，推荐优先考虑；方案 2 更明确但涉及字段类型变更。

**验收**

- 首次调用不再返回误导性的 `100.0`（或明确标记未采样）；
- 连续调用能返回稳定 usage；
- 文档与测试同步更新。
