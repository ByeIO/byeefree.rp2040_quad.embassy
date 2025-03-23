#![allow(unused)]

//! 起飞任务

/// 多任务相关
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
use crate::utils::types::flight::{ FlightModeEnum, FlightType };
/// 控制器
use crate::modules::controllers::lqr;

// 起飞参数配置
/// 基础推力（悬停时电机转速）
const BASE_THRUST: u16 = 40;    
/// 最大允许倾斜角度(度)
const MAX_ANGLE: f32 = 5.0;     
/// 上升速率(m/s)
const ASCEND_RATE: f32 = 0.5;   
/// 起飞超时时间(ms)
const TIMEOUT: u64 = 15000;

/// 起飞过程状态
#[derive(Clone, Copy)]
pub enum TakeoffState {
    // 预检测
    PreCheck,      
    // 动力建立
    ThrustBuildUp, 
    // 离地确认
    LiftOff,       
    // 稳定爬升
    Ascending      
}

/// 起飞任务
#[embassy_executor::task]
pub async fn take_off_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from take_off_task");
    
    loop{
        // 尝试获取消息
        let mut flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态为READY状态才可起飞
        if matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::TakeOff){
            // 获取消息中的参数: 期望高度(cm)
            let altitude_wanted = flight_mode_wanted.altitude;
            
            // 1. 预起飞检查
            let mut imu_data = variables::Imu10DofDataEditor::get_data().await;
            let mut last_altitude = imu_data.altitude[0] as f32;
            let timer = Instant::now();
            let time_before = timer.as_millis();
            
            // 2. 离地阶段控制
            let mut thrust = BASE_THRUST;
            
            // 循环直到大于等于期望高度
            loop{
                // 获取当前高度(cm)
                let altitude_now = variables::Imu10DofDataEditor::get_data().await.altitude[0] as i16;
                
                // 2.1 TODO 获取融合后的传感器数据
                // imu_data = variables::Imu10DofDataEditor::get_data().await;
                
                if altitude_now < altitude_wanted {
                    // 2.2 姿态保持控制（LQR控制器）
                    let target_quat = [1.0, 0.0, 0.0, 0.0]; // 期望水平姿态
                    lqr::lqr_controller(
                        target_quat,
                    ).await;
                    
                    // TODO 跟踪传感器高度数据和加速度数据, 继续调用控制器任务使X型四旋翼无人机离地起飞
                    // 2.3 高度PID控制
                    /// 计算误差
                    let altitude_error = altitude_wanted - altitude_now;
                    let delta_z = pid_height_control(altitude_error.into(), ASCEND_RATE).await;
                    
                    // 2.4 推力分配
                    thrust = (BASE_THRUST as f32 + delta_z).clamp(30.0, 70.0) as u16;
                    signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
                        (Some(thrust), Some(thrust), Some(thrust), Some(thrust))
                    );
                    
                    // 超时检测
                    if timer.as_millis() - time_before > TIMEOUT {
                        defmt::println!("take_off : timeout err");
                        break;
                    }
                    
                    // 50Hz控制频率
                    Timer::after(Duration::from_millis(20)).await;
                    
                }// end if
                else{
                    break;
                }
            }// end loop
            
            // 起飞成功, 切换到悬停状态
            flight_mode_wanted.mode = FlightModeEnum::Hovering;
            signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
            
        }// end if matches
        else{
            defmt::println!("take_off : flight_mode_now or flight_wanted err");
        }
    }// end loop
    
}

/// 高度PID控制器
pub async fn pid_height_control(error: f32, rate: f32) -> f32 {
    const KP: f32 = 0.8;
    const KI: f32 = 0.05;
    const KD: f32 = 0.2;
    
    static mut INTEGRAL: f32 = 0.0;
    static mut LAST_ERROR: f32 = 0.0;
    
    unsafe {
        // 积分项（dt=20ms）
        INTEGRAL += error * 0.02;
        let derivative = (error - LAST_ERROR) / 0.02;
        LAST_ERROR = error;
        
        KP * error + KI * INTEGRAL + KD * derivative
    }
}

/// 紧急降落程序
pub async fn emergency_land() {
    defmt::error!("[紧急降落] 检测到异常姿态");
    signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
        (Some(0), Some(0), Some(0), Some(0))
    );
}
