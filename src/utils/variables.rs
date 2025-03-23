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

/* start 10轴传感器 */
use crate::utils::types::sensor::Imu10DofData;
// 10轴传感器数据，使用异步Mutex保护
static IMU_10DOF_DATA: Mutex<ThreadModeRawMutex, Imu10DofData<f32>> = Mutex::new(Imu10DofData {
    gyr: [0.0; 3],
    acc: [0.0; 3],
    mag: [0.0; 3],
    pressure: [0.0; 1],
    quat: [0.0; 4],
    altitude: [0.0; 1],
    temperature: [0.0; 1],
});

/// 10轴传感器数据操作接口
pub struct Imu10DofDataEditor;

impl Imu10DofDataEditor {
    /// 异步获取10轴传感器数据（拷贝值返回）
    pub async fn get_data() -> Imu10DofData<f32> {
        let guard = IMU_10DOF_DATA.lock().await;
        *guard
    }

    /// 异步更新全部10轴传感器数据
    pub async fn write_data(data: Imu10DofData<f32>) {
        let mut guard = IMU_10DOF_DATA.lock().await;
        *guard = data;
    }

    /// 异步更新单个传感器数据字段
    pub async fn update_field<F>(update_fn: F)
    where
        F: FnOnce(&mut Imu10DofData<f32>),
    {
        let mut guard = IMU_10DOF_DATA.lock().await;
        update_fn(&mut *guard);
    }
}
/* end 10轴传感器 */

/* start 当前飞行状态 */
use crate::utils::types::flight::{ FlightModeEnum, FlightType};

// 当前飞行状态, 使用异步Mutex保护
static FLIGHT_MODE: Mutex<ThreadModeRawMutex, FlightType> = Mutex::new(
    FlightType {
        // 默认为Ready状态
        mode: FlightModeEnum::Ready,
        altitude: 0,
        angle: 0,
        time: 0,
        radius: 0,
    }
);

///当前飞行状态操作接口
pub struct FlightModeEditor;

impl FlightModeEditor {
    /// 异步获取当前飞行状态(拷贝值返回)
    pub async fn get_data() -> FlightType {
        let guard = FLIGHT_MODE.lock().await;
        *guard
    }

    /// 异步更新当前飞行状态
    pub async fn write_data(data: FlightType) {
        let mut guard = FLIGHT_MODE.lock().await;
        *guard = data;
    }
}
/* end 当前飞行状态 */
