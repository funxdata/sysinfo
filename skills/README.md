# 开发规范

本目录记录 `sys_info` 的开发约定，供新增功能、修改代码和发版时对照使用。

> 说明：这里是普通文档目录，不会被 Reasonix 自动加载为 skill。若希望规范可被自动发现并调用，应放到 `.reasonix/skills/<name>/SKILL.md`（项目级 skill 约定）。

## 规范索引

| 文件 | 内容 |
| --- | --- |
| [rust-conventions.md](rust-conventions.md) | 模块分层、命名、错误处理、注释、unsafe 与全局状态 |
| [ffi-conventions.md](ffi-conventions.md) | FFI 函数契约、输入解析、统一响应、错误码与新增接口清单 |
| [testing-conventions.md](testing-conventions.md) | 测试组织、断言方式、临时资源清理、对话框测试 |
| [release-conventions.md](release-conventions.md) | 版本同步、提交信息、tag 与 dytes 发布 |

## 基本原则

1. **边界与实现分离**：`src/ffi/` 只处理 C ABI、JSON 解析与输出；业务逻辑放在 `src/utils/`，不接触裸指针。
2. **统一契约**：所有导出函数返回 `{ code, data, msg }`，成功码固定 `200`，错误码集中定义在 `src/utils/errcode.rs`。
3. **不破坏调用方**：已发布的字段只增不改、不改语义；确需变更时提升版本并在 `docs/` 说明。
4. **可测试**：新增 FFI 接口必须同时补集成测试；测试不得依赖本机特定数据。
5. **最小改动**：一次提交只做一件事，避免顺手重构无关代码。

## 项目入口

- 功能与接口说明：[docs/README.md](../docs/README.md)
- 架构与全局状态：[docs/architecture.md](../docs/architecture.md)
- FFI API 字段：[docs/ffi-api.md](../docs/ffi-api.md)
- 错误码表：[docs/error-codes.md](../docs/error-codes.md)
- 构建与发布：[docs/build-and-release.md](../docs/build-and-release.md)
