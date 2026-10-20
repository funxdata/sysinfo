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

/// 在全局 System 上执行闭包，必要时初始化
pub fn with_system<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut System) -> R,
{
    let mut guard = SYSTEM
        .lock()
        .map_err(|e| (errcode::ERR_INTERNAL_LOCK, format!("system lock: {}", e)))?;
    if guard.is_none() {
        *guard = Some(System::new());
    }
    Ok(f(guard.as_mut().unwrap()))
}

/// 在全局 Disks 上执行闭包，必要时初始化
pub fn with_disks<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Disks) -> R,
{
    let mut guard = DISKS
        .lock()
        .map_err(|e| (errcode::ERR_INTERNAL_LOCK, format!("disks lock: {}", e)))?;
    if guard.is_none() {
        *guard = Some(Disks::new_with_refreshed_list());
    }
    Ok(f(guard.as_mut().unwrap()))
}

/// 在全局 Networks 上执行闭包，必要时初始化
pub fn with_networks<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Networks) -> R,
{
    let mut guard = NETWORKS
        .lock()
        .map_err(|e| (errcode::ERR_INTERNAL_LOCK, format!("networks lock: {}", e)))?;
    if guard.is_none() {
        *guard = Some(Networks::new_with_refreshed_list());
    }
    Ok(f(guard.as_mut().unwrap()))
}

/// 在全局 Components 上执行闭包，必要时初始化
pub fn with_components<F, R>(f: F) -> Result<R, (i32, String)>
where
    F: FnOnce(&mut Components) -> R,
{
    let mut guard = COMPONENTS
        .lock()
        .map_err(|e| (errcode::ERR_INTERNAL_LOCK, format!("components lock: {}", e)))?;
    if guard.is_none() {
        *guard = Some(Components::new_with_refreshed_list());
    }
    Ok(f(guard.as_mut().unwrap()))
}


