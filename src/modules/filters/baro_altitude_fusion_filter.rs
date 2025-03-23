#![allow(dead_code)]

//! 气压融合互补滤波器

// no_std库
use core::f32::consts::PI;
// 数学运算
use libm::powf; 
use nalgebra::{UnitQuaternion, Quaternion};

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

// 内部库
use crate::utils::types::sensor::Imu10DofData;
use crate::utils::variables::Imu10DofDataEditor;

// 滤波器参数配置
/// 互补滤波截止频率（rad/s）
const COMPLEMENTARY_OMEGA: f32 = 3.0;   
/// 气压速度增益
const BARO_VEL_GAIN: f32 = 0.1;         
/// 重力加速度（m/s²）
const GRAVITY: f32 = 9.80665;           
/// 采样周期（秒）
const DT: f32 = 0.005;                  

// 定义静态变量存储滤波器状态（高度、速度、加速度偏差）
#[derive(Copy, Default, Clone)]
pub struct FilterState {
    pub z_est: [f32; 3],      // 估计状态：[高度, 速度, 加速度]
    pub z_bias: f32,          // 加速度计零偏
    pub last_update: u64,     // 最后更新时间戳（毫秒）
}

/* start 全局静态变量 */
/// 使用Mutex保护滤波器状态（线程安全）
static FILTER_STATE: Mutex<ThreadModeRawMutex, FilterState> = Mutex::new(FilterState{
    z_est: [0.0, 0.0, 0.0],
    z_bias: 0.0,
    last_update: 0u64,
});

/// 滤波器状态机操作接口
pub struct FilterStateEditor;

impl FilterStateEditor {
    /// 异步获取10轴传感器数据（拷贝值返回）
    pub async fn get_data() -> FilterState {
        let guard = FILTER_STATE.lock().await;
        *guard
    }

    /// 异步更新全部10轴传感器数据
    pub async fn write_data(data: FilterState) {
        let mut guard = FILTER_STATE.lock().await;
        *guard = data;
    }

    /// 异步更新单个传感器数据字段
    pub async fn update_field<F>(update_fn: F)
    where
        F: FnOnce(&mut FilterState),
    {
        let mut guard = FILTER_STATE.lock().await;
        update_fn(&mut *guard);
    }
}
/* end 全局静态变量 */

/// 获取系统时间
fn get_system_time() -> u64 {
    let instant = embassy_time::Instant::now();
    instant.as_millis()
}

/// 气压融合互补滤波器
pub async fn baro_altitude_fusion_filter() -> Imu10DofData<f32>{
    
    // 获取全局变量的值 IMU_10DOF_DATA
    let imu_data = Imu10DofDataEditor::get_data().await;
    
    // 获取当前时间戳
    let current_time = get_system_time();

    // 计算时间差(秒)
    let mut dt = (current_time - FILTER_STATE.lock().await.last_update) as f32 / 1000.0;
    
    // 限制时间差范围
    dt = dt.max(0.001).min(0.1);
    
    // 更新滤波器状态时间戳
    FilterStateEditor::update_field(|state| state.last_update = current_time).await;
    
    // 从加速度计获取原始数据（需单位转换为m/s²）
    let accel_z = imu_data.acc[2] * GRAVITY;
    
    // 计算垂直方向加速度（去除重力影响）
    let vertical_accel = accel_z - FILTER_STATE.lock().await.z_bias;
    
    // 预测步骤
    FilterStateEditor::update_field(|state| {
        state.z_est[0] += state.z_est[1] * dt + vertical_accel * powf(dt, 2.0) / 2.0;
        state.z_est[1] += vertical_accel * dt;
    }).await;
    
    // 获取气压计高度（需校准和单位转换, 单位m)
    let baro_altitude = imu_data.pressure[0] / 100.0; 
    
    // 计算高度误差
    let height_error = baro_altitude - FILTER_STATE.lock().await.z_est[0];
    
    // 互补滤波校正（参考三阶互补滤波算法）
    let omega_sq = COMPLEMENTARY_OMEGA * COMPLEMENTARY_OMEGA;
    let integ1 = height_error * omega_sq * COMPLEMENTARY_OMEGA * dt;
    // 更新加速度估计
    FilterStateEditor::update_field(|state| state.z_est[2] += integ1).await; 
    
    let integ2 = (FILTER_STATE.lock().await.z_est[2] + vertical_accel 
        + height_error * omega_sq * 3.0) * dt;
    // 更新速度估计
    FilterStateEditor::update_field(|state| state.z_est[1] += integ2).await; 
    
    let integ3 = (height_error * COMPLEMENTARY_OMEGA * 3.0 
        + FILTER_STATE.lock().await.z_est[1]) * dt;
    // 更新高度估计
    FilterStateEditor::update_field(|state| state.z_est[0] += integ3).await; 
    
    // 更新加速度计零偏
    FilterStateEditor::update_field(|state| state.z_bias -= height_error * 0.05 * dt).await;
    
    // 将融合后的高度更新到IMU数据结构
    let mut updated_imu_data = imu_data;
    updated_imu_data.altitude[0] = FILTER_STATE.lock().await.z_est[0];
    
    // 返回滤波后的值
    updated_imu_data
}
