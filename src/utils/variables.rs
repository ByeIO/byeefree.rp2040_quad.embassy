#![allow(dead_code)]

//! 全局变量

// 用静态变量互斥访问来对冲rust的生命周期、借用难, 
// 所有的操作接口都是：&'static self。

// 静态变量互斥访问相关
use embassy_sync::blocking_mutex::{
    CriticalSectionMutex, 
    raw::{
        CriticalSectionRawMutex, ThreadModeRawMutex
    },
};
use embassy_sync::mutex::Mutex;
// use core::sync::atomic::{AtomicU16, Ordering};
use static_cell::{StaticCell, ConstStaticCell};
use core::{
    //内部可变性类型RefCell
    cell::RefCell, 
    fmt::Write, 
    //引用借用
    borrow::BorrowMut,
    borrow::Borrow,
};

/// 初始化全局变量
pub fn init_variables(){
    // unimplemented!()
}

/* start 电机速度 */
/// 全局电机速度，使用异步Mutex保护
static MOTOR_SPEEDS: Mutex<ThreadModeRawMutex, [u16; 4]> = Mutex::new([0u16; 4]);

/// 电机速度操作接口
pub struct MotorSpeedEditor;

impl MotorSpeedEditor {
    /// 异步获取电机速度（拷贝值返回）
    pub async fn get_speeds() -> [u16; 4] {
        let guard = MOTOR_SPEEDS.lock().await;
        *guard
    }

    /// 异步更新全部电机速度
    pub async fn write_speeds(speeds: [u16; 4]) {
        let mut guard = MOTOR_SPEEDS.lock().await;
        *guard = speeds;
    }

    /// 异步设置单个电机速度
    pub async fn set_speed(index: usize, value: u16) {
        let mut guard = MOTOR_SPEEDS.lock().await;
        guard[index] = value;
    }
}
/* end 电机速度 */

/* start 电机方向 */
// 1: 右转, 0: 左转
/// 全局电机方向，使用异步Mutex保护
static MOTOR_DIRECTIONS: Mutex<ThreadModeRawMutex, [bool; 4]> = Mutex::new([true; 4]);

/// 电机方向操作接口
pub struct MotorDirectionEditor;

impl MotorDirectionEditor {
    /// 异步获取电机方向（拷贝值返回）
    pub async fn get_directions() -> [bool; 4] {
        let guard = MOTOR_DIRECTIONS.lock().await;
        *guard
    }

    /// 异步更新全部电机方向
    pub async fn write_directions(directions: [bool; 4]) {
        let mut guard = MOTOR_DIRECTIONS.lock().await;
        *guard = directions;
    }

    /// 异步设置单个电机方向
    pub async fn set_direction(index: usize, value: bool) {
        let mut guard = MOTOR_DIRECTIONS.lock().await;
        guard[index] = value;
    }
}
/* end 电机方向 */
