//! 全局 sysinfo 实例状态
//!
//! sysinfo 需要保留实例以计算差值（CPU 使用率等），且进程列表首次刷新开销大，
//! 因此所有采集都共享这些全局实例。

use std::sync::Mutex;

use sysinfo::{Components, Disks, Networks, System};

use crate::utils::errcode;

pub static SYSTEM: Mutex<Option<System>> = Mutex::new(None);
pub static DISKS: Mutex<Option<Disks>> = Mutex::new(None);
pub static NETWORKS: Mutex<Option<Networks>> = Mutex::new(None);
pub static COMPONENTS: Mutex<Option<Components>> = Mutex::new(None);

/// 在全局状态上执行闭包，必要时用 `init` 初始化。
fn with_state<T, F, R>(
    state: &Mutex<Option<T>>,
    name: &str,
    init: impl FnOnce() -> T,
    f: F,
) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut T) -> R,
{
    let mut guard = state
        .lock()
        .map_err(|e| (errcode::ERR_INTERNAL_LOCK, format!("{} lock: {}", name, e)))?;
    if guard.is_none() {
        *guard = Some(init());
    }
    let value = guard.as_mut().ok_or_else(|| {
        (
            errcode::ERR_INTERNAL_LOCK,
            format!("{} state unavailable", name),
        )
    })?;
    Ok(f(value))
}

/// 在全局 System 上执行闭包，必要时初始化
pub fn with_system<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut System) -> R,
{
    with_state(&SYSTEM, "system", System::new, f)
}

/// 在全局 Disks 上执行闭包，必要时初始化
pub fn with_disks<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Disks) -> R,
{
    with_state(&DISKS, "disks", Disks::new_with_refreshed_list, f)
}

/// 在全局 Networks 上执行闭包，必要时初始化
pub fn with_networks<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Networks) -> R,
{
    with_state(&NETWORKS, "networks", Networks::new_with_refreshed_list, f)
}

/// 在全局 Components 上执行闭包，必要时初始化
pub fn with_components<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Components) -> R,
{
    with_state(
        &COMPONENTS,
        "components",
        Components::new_with_refreshed_list,
        f,
    )
}
