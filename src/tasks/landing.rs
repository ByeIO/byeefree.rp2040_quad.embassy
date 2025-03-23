#![allow(unused)]

//! 降落任务

// 多任务相关
use embassy_time::{Duration, Timer, Instant};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

/// 数学计算
use nalgebra::{Quaternion, Vector3};
use libm::{acos, sqrtf};

// 内部库
/// 信号
use crate::utils::signals;
use crate::utils::types::sensor::Imu10DofData;
/// 全局变量
use crate::utils::variables;
/// 滤波器
use crate::filters::baro_altitude_fusion_filter::baro_altitude_fusion_filter;
/// 类型
use crate::utils::types::flight::{ FlightModeEnum, FlightType};
/// 控制器
use crate::modules::controllers::lqr;

// 降落参数配置
/// 基础推力（悬停时电机转速）
const BASE_THRUST: u16 = 40;
/// 最大允许倾斜角度(度)
const MAX_ANGLE: f32 = 5.0;
/// 下降速率(m/s)
const DESCEND_RATE: f32 = -0.3;
/// 着陆高度阈值(cm)
const LANDING_ALTITUDE_THRESHOLD: f32 = 10.0;
/// 降落超时时间(ms)
const LANDING_TIMEOUT: u64 = 20000;

/// 降落任务
#[embassy_executor::task]
pub async fn landing_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from landing_task");
    
    loop{
        // 尝试获取消息
        let mut flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态才执行降落
        if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Landing){
            defmt::println!("landing: start landing procedure");
            
            // 获取初始高度和时间戳
            let imu_data = variables::Imu10DofDataEditor::get_data().await;
            let initial_altitude = imu_data.altitude[0] as f32;
            let timer = Instant::now();
            let time_before = timer.as_millis();
            let mut last_altitude = initial_altitude;
            
            // 进入降落循环
            loop {
                // 获取当前传感器数据
                let imu_data = variables::Imu10DofDataEditor::get_data().await;
                let altitude_now = imu_data.altitude[0] as f32;
                // 计算垂直速度(m/s)
                let vertical_velocity = (last_altitude - altitude_now) / 0.02; 
                last_altitude = altitude_now;
                
                // 着陆条件检测：高度低于阈值且速度接近0
                if altitude_now <= LANDING_ALTITUDE_THRESHOLD && abs(vertical_velocity) < 0.1 {
                    defmt::println!("landing: touchdown detected");
                    signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
                        (Some(0), Some(0), Some(0), Some(0))
                    );
                    // 切换模式到Ready
                    flight_mode_wanted.mode = FlightModeEnum::Ready;
                    signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                    break;
                }
                
                // 超时检测
                if timer.as_millis() - time_before > LANDING_TIMEOUT {
                    defmt::error!("landing: timeout, triggering emergency");
                    emergency_land().await;
                    break;
                }
                
                // 姿态保持控制（LQR控制器）
                /// 期望水平姿态
                let target_quat = [1.0, 0.0, 0.0, 0.0]; 
                lqr::lqr_controller(target_quat).await;
                
                // 高度PID控制
                /// 目标高度为0
                let altitude_error = -altitude_now; 
                let delta_z = pid_height_control(altitude_error, DESCEND_RATE).await;
                
                // 推力分配（逐渐降低）
                let thrust = (BASE_THRUST as f32 + delta_z).clamp(30.0, 70.0) as u16;
                signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
                    (Some(thrust), Some(thrust), Some(thrust), Some(thrust))
                );
                
                // 控制频率50Hz
                Timer::after(Duration::from_millis(20)).await;
            }
        } else {
            defmt::println!("landing: invalid state for landing");
        }
    }// end loop
}

/// 高度PID控制器（复用起飞逻辑，参数可调整）
pub async fn pid_height_control(error: f32, rate: f32) -> f32 {
    const KP: f32 = 0.6;
    const KI: f32 = 0.03;
    const KD: f32 = 0.15;
    
    static mut INTEGRAL: f32 = 0.0;
    static mut LAST_ERROR: f32 = 0.0;
    
    // TODO 修改unsafe为安全操作
    unsafe {
        // 积分项（dt=20ms）
        INTEGRAL += error * 0.02;
        let derivative = (error - LAST_ERROR) / 0.02;
        LAST_ERROR = error;
        
        KP * error + KI * INTEGRAL + KD * derivative
    }
}

/// 计算当前倾斜角度（单位：度）
pub async fn calculate_tilt_angle(quat: &Quaternion<f32>) -> f32 {
    // 计算俯仰角或横滚角的最大倾斜
    let sin_theta = 2.0 * (quat.w * quat.coords.y - quat.coords.z * quat.coords.x);
    let angle = libm::asinf(sin_theta).to_degrees();
    abs(angle)
}

/// 紧急降落程序（复用起飞逻辑）
pub async fn emergency_land() {
    defmt::error!("[紧急降落] 检测到异常姿态");
    signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
        (Some(0), Some(0), Some(0), Some(0))
    );
}

/// 绝对值
pub fn abs(x: f32) -> f32 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}
