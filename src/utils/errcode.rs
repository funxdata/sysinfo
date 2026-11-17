//! 统一错误码定义

// ---- 输入/参数错误 ----
pub const ERR_JSON_IN_NULL: i32 = 400100;
pub const ERR_INVALID_UTF8: i32 = 400101;
pub const ERR_INVALID_JSON: i32 = 400102;
pub const ERR_EMPTY_FIELD: i32 = 400103;
pub const ERR_INVALID_PATH: i32 = 400104;

// ---- 业务执行错误 ----
pub const ERR_CURRENT_DIR: i32 = 400900;
pub const ERR_CREATE_DIR: i32 = 400901;
pub const ERR_PROCESS_INFO: i32 = 400906;
pub const ERR_FILE_INFO: i32 = 400909;
pub const ERR_INTERNAL_LOCK: i32 = 400999;

// ---- FFI 边界错误 ----
pub const ERR_PANIC: i32 = 500000;

// ---- 成功 ----
pub const CODE_SUCCESS: i32 = 200;
