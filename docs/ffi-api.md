# FFI API

本文档描述 `sys_info` 动态库导出的全部函数。所有函数位于 crate 根，符号名即函数名。

## 调用约定

```c
int32_t post_xxx(const char *json_in, char *json_out);
```

| 项目 | 约定 |
| --- | --- |
| 输入 | UTF-8 编码、以 `\0` 结尾的 JSON 字符串；不需要输入时可传 `NULL` |
| 输出 | 调用方提供的缓冲区，**至少 `65536` 字节**（`MAX_OUTPUT_LEN`，含结尾 `\0`） |
| 返回码 | `0`：已向 `json_out` 写入 JSON（业务成功或失败均如此）；`-1`：`json_out` 为 `NULL`；`-2`：输出超过缓冲区长度，未写入 |
| 输出超长 | 返回 `-2`，不会写入任何内容；调用方应减小 `limit` 或重试 |
| 输入解析失败 | 返回 `0`，`json_out` 为错误 JSON |
| 并发 | 同一类采集在多线程间串行，详见 [架构说明](architecture.md) |

> 重要：返回码只表示“是否成功写入输出”，**业务结果必须看 JSON 中的 `code` 字段**。

### 统一响应格式

成功：

```json
{ "code": 200, "data": { }, "msg": "success" }
```

失败：

```json
{ "code": 400102, "data": null, "msg": "invalid json: ..." }
```

`code`、`msg` 含义见 [错误码](error-codes.md)。下文只列 `data` 结构与输入字段。

---

## 系统类

### `post_system_basic_info`

输入：无（忽略 `json_in`）。

```json
{
  "name": "Windows 11 Pro",
  "kernel_version": "10.0.22631",
  "os_version": "23H2",
  "host_name": "DESKTOP-XXX",
  "uptime": 123456,
  "boot_time": 1700000000,
  "load_average": { "one": 0.5, "five": 0.4, "fifteen": 0.3 }
}
```

- `uptime`：秒。
- `boot_time`：Unix 时间戳（秒）。
- 名称/版本等字段在平台不支持时为 `null`。

### `post_memory_info`

输入：无。

```json
{
  "total_bytes": 34359738368,
  "used_bytes": 17179869184,
  "free_bytes": 17179869184,
  "usage_percent": 50.0,
  "total_swap_bytes": 8589934592,
  "used_swap_bytes": 0
}
```

- 所有容量单位为字节。
- `usage_percent` 保留两位小数；总内存为 0 时为 `0.0`。

### `post_cpu_info`

输入：无。

```json
{
  "global_usage": 12.5,
  "physical_core_count": 8,
  "logical_core_count": 16,
  "cpus": [
    {
      "name": "cpu0",
      "brand": "AMD Ryzen 7 ...",
      "vendor_id": "AuthenticAMD",
      "frequency": 3200,
      "usage": 10.0
    }
  ]
}
```

- `frequency`：MHz。
- `usage` / `global_usage`：百分比，来自两次刷新差值。**实例创建后的首次刷新没有历史采样**，此时 `usage` 可能返回 `100.0`（Windows 实测）或 `0.0`，调用方应忽略首次结果。
- `physical_core_count`：平台无法获取时为 `null`。
- `logical_core_count` 等于 `cpus` 数组长度。
- `name` 为 `sysinfo` 给出的逻辑核名称（如 `CPU 1`）。

### `post_disk_info`

输入：无。

```json
{
  "count": 2,
  "total_bytes": 1000204886016,
  "total_used_bytes": 500000000000,
  "total_available_bytes": 500204886016,
  "disks": [
    {
      "name": "C:\\",
      "mount_point": "C:\\",
      "file_system": "NTFS",
      "total_bytes": 1000204886016,
      "used_bytes": 500000000000,
      "available_bytes": 500204886016,
      "usage_percent": 49.99,
      "is_removable": false,
      "is_read_only": false,
      "kind": "SSD",
      "read_bytes": 123456789,
      "written_bytes": 987654321
    }
  ]
}
```

- `used_bytes = total_bytes - available_bytes`（饱和减法）。
- `name` 可能为空字符串，平台无卷名时只能依赖 `mount_point`。
- `kind` 为 `sysinfo` 磁盘类型的 `Debug` 字符串（如 `SSD`、`HDD`、`Unknown`）。
- `read_bytes` / `written_bytes` 为累计读写字节数。
- 顶层合计字段是各磁盘的累加。

### `post_network_info`

输入：无。

```json
{
  "count": 1,
  "interfaces": [
    {
      "interface": "eth0",
      "total_received": 123456,
      "total_transmitted": 654321,
      "total_packets_received": 1000,
      "total_packets_transmitted": 900,
      "total_errors_on_received": 0,
      "total_errors_on_transmitted": 0,
      "mac_address": "44:e5:17:98:0a:99"
    }
  ]
}
```

- 字节数与包数均为累计值。
- `mac_address` 为标准冒号分隔的十六进制格式（小写），如 `"44:e5:17:98:0a:99"`；无 MAC 时通常为 `"00:00:00:00:00:00"`。
- 无网络接口时 `count` 为 `0`、`interfaces` 为空数组。

### `post_process_list`

输入（可选；`json_in` 可为 `NULL`）：

```json
{ "limit": 100, "offset": 0 }
```

| 字段 | 类型 | 默认值 | 约束 |
| --- | --- | --- | --- |
| `limit` | number | `100` | 自动收敛到 `1..=1000` |
| `offset` | number | `0` | 从 0 开始，按 `pid` 升序分页 |

输出：

```json
{
  "total": 342,
  "offset": 0,
  "limit": 100,
  "count": 100,
  "has_more": true,
  "processes": [
    {
      "pid": 1,
      "name": "systemd",
      "memory": 10485760,
      "cpu_usage": 0.1,
      "status": "Sleep"
    }
  ]
}
```

- `memory`：字节。
- `name` 超过 128 个字符时按字符边界截断，不会在多字节字符中间切断。
- `status` 为 `sysinfo` 进程状态的 `Debug` 字符串（如 `Run`、`Sleep`、`Zombie`）。
- 进程按 `pid` 升序排序，保证分页稳定。
- `has_more = offset + count < total`。

### `post_component_info`

输入：无。

```json
{
  "count": 1,
  "components": [
    {
      "label": "Tctl",
      "temperature": 45.0,
      "max": 95.0,
      "critical": null
    }
  ]
}
```

- 温度单位为摄氏度。
- 无传感器（如多数虚拟机）时 `count` 为 `0`、`components` 为空数组。
- `max` / `critical` 为传感器阈值；传感器不提供该阈值时为 `null`（底层 `-273.15` 哨兵值已归一化）。

---

## 文件类

### `post_cur_dir`

输入：无，或任意合法 JSON（内容被忽略）。

输出：

```json
{ "path": "D:\\codes\\project" }
```

- `json_in` 非 `NULL` 但不是合法 JSON 时返回 `400102`。

### `post_create_dir`

输入：

```json
{ "dir_name": "logs/2026" }
```

- `dir_name` 必须是当前目录下的**相对路径**；拒绝绝对路径、盘符/UNC 前缀（如 `C:foo`、`\\server\share`）、根目录、`..` 和 Windows 保留名（`CON`、`NUL`、`COM1` 等）。
- 支持多级目录（`create_dir_all`）。

输出：

```json
{ "path": "D:\\codes\\project\\logs\\2026" }
```

- 空字符串或纯空白 → `400103`。
- 绝对路径、盘符/UNC 前缀、`..` 或 Windows 保留名 → `400104`。
- 创建失败（权限、磁盘等）→ `400901`。

### `post_file_exists`

输入：

```json
{ "path": "D:\\codes\\project\\README.md" }
```

输出：

```json
{ "exists": true }
```

- 不跟随符号链接：判断的是链接自身是否存在；悬空链接也会返回 `true`。
- 路径无法访问（权限、I/O 错误）时返回 `400104`，不再静默当作不存在。

### `post_file_info`

输入：

```json
{ "path": "D:\\codes\\project\\README.md" }
```

输出（使用 `symlink_metadata`，**不跟随符号链接**）：

```json
{
  "exists": true,
  "is_file": true,
  "is_dir": false,
  "is_symlink": false,
  "size": 1024,
  "modified": 1730000000000,
  "created": 1720000000000
}
```

- `size`：字节。
- `modified` / `created`：Unix 毫秒时间戳；平台不支持时为 `null`。
- 路径不存在时返回成功包裹：

```json
{ "exists": false }
```

- 元数据读取失败（非 `NotFound`，如权限错误）返回 `400909`。

---

## 对话框类

### `post_choose_file`

输入：无（忽略 `json_in`），弹出系统原生文件选择对话框。

输出：

- 选择文件：`{ "path": "C:\\Users\\...\\file.txt" }`，`msg` 为 `"success"`。
- 取消：`{ "path": null }`，`msg` 为 `"cancelled"`，`code` 仍为 `200`。

### `post_choose_dir`

输入：无，弹出系统原生目录选择对话框，输出格式同上。

> 两个对话框函数会阻塞调用线程直到用户操作；在无图形界面或 CI 环境中不可用。
> 对应测试默认标记 `#[ignore]`，需手动运行。
