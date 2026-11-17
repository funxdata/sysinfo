# 错误码

响应体中的 `code` 为业务状态码；FFI 函数本身的返回值只表示输出是否写入成功（`0` / `-1` / `-2`）。常量定义见 `src/utils/errcode.rs`。

## 成功

| 常量 | 值 | 说明 |
| --- | --- | --- |
| `CODE_SUCCESS` | `200` | 调用成功（对话框取消也属于成功） |

## 输入 / 参数错误（4001xx）

| 常量 | 值 | 触发场景 |
| --- | --- | --- |
| `ERR_JSON_IN_NULL` | `400100` | 必填输入为 `NULL`（如 `post_create_dir` 未传 `json_in`） |
| `ERR_INVALID_UTF8` | `400101` | 输入不是合法 UTF-8 |
| `ERR_INVALID_JSON` | `400102` | 输入不是合法 JSON，或缺少必填字段 |
| `ERR_EMPTY_FIELD` | `400103` | `dir_name` 为空或纯空白 |
| `ERR_INVALID_PATH` | `400104` | 路径为绝对路径/盘符/UNC/根目录、包含 `..`、Windows 保留名，或路径无法访问 |

## 业务执行错误（4009xx）

| 常量 | 值 | 触发场景 |
| --- | --- | --- |
| `ERR_CURRENT_DIR` | `400900` | 获取当前工作目录失败 |
| `ERR_CREATE_DIR` | `400901` | 创建目录失败（权限、磁盘、已存在文件等） |
| `ERR_PROCESS_INFO` | `400906` | 进程列表采集失败 |
| `ERR_FILE_INFO` | `400909` | 读取文件元数据失败（非 `NotFound`，如权限错误） |
| `ERR_INTERNAL_LOCK` | `400999` | 全局状态互斥锁中毒（内部错误） |

## FFI 边界错误

| 常量 | 值 | 触发场景 |
| --- | --- | --- |
| `ERR_PANIC` | `500000` | FFI 边界捕获到 panic，返回 `{ "code": 500000, "data": null, "msg": "internal panic" }` |

> 说明：`post_*` 在 FFI 边界使用 `catch_unwind`；捕获 panic 后不会让 unwind 跨过 `extern "C"`，宿主进程可以继续运行。若 panic 发生在持有全局锁期间，锁可能中毒，后续调用返回 `400999`。

## FFI 返回值

| 返回值 | 说明 |
| --- | --- |
| `0` | 已写入 `json_out`，需解析 JSON 里的 `code` |
| `-1` | `json_out` 为 `NULL` |
| `-2` | 序列化结果超过 `MAX_OUTPUT_LEN`（65536 字节，含 `\0`），未写入 |

`-1` / `-2` 时不会写入任何 JSON。`-2` 属于可恢复错误：调用方应减小分页 `limit` 或重试；当前没有查询所需长度的接口。
