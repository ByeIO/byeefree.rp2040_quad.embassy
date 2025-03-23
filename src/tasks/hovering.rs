#![allow(unused)]

//! 悬停任务

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

// 悬停参数配置
/// 基础推力（悬停时电机转速）
const BASE_THRUST: u16 = 40;
/// 最大允许倾斜角度(度)
const MAX_ANGLE: f32 = 5.0;
/// 高度PID控制参数
const KP: f32 = 0.6;
const KI: f32 = 0.03;
const KD: f32 = 0.15;

/// 悬停任务
#[embassy_executor::task]
pub async fn hovering_task(
    // 飞行参数
    mut in_flight_args: signals::FlightSub,
) {
    defmt::println!("hello from hovering_task");

    loop {
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;

        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;

        // 当前状态为READY且请求切换至悬停模式
        if matches!(flight_mode_now, FlightModeEnum::Ready)
            && matches!(flight_mode_wanted.mode, FlightModeEnum::Hovering)
        {
            defmt::println!("hovering: start hovering procedure");

            // 获取初始高度和四元数
            let imu_data = variables::Imu10DofDataEditor::get_data().await;
            let initial_altitude = imu_data.altitude[0] as f32;
            let mut integral = 0.0;
            let mut last_error = 0.0;
            let timer = Instant::now();

            // 进入悬停循环
            loop {
                // 获取传感器数据
                let imu_data = variables::Imu10DofDataEditor::get_data().await;
                let altitude_now = imu_data.altitude[0] as f32;

                // 计算高度误差
                let error = initial_altitude - altitude_now;

                // PID计算
                // 积分项 (dt=20ms)
                integral += error * 0.02; 
                // 微分项
                let derivative = (error - last_error) / 0.02; 
                let delta_z = KP * error + KI * integral + KD * derivative;
                last_error = error;

                // 计算总推力
                let thrust = (BASE_THRUST as f32 + delta_z).clamp(30.0, 70.0) as u16;

                // 姿态保持控制（LQR控制器）
                /// 水平姿态
                let target_quat = [1.0, 0.0, 0.0, 0.0]; 
                lqr::lqr_controller(target_quat).await;

                // 发布电机转速
                signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
                    (Some(thrust), Some(thrust), Some(thrust), Some(thrust))
                );

                // 检查模式切换
                let current_mode = variables::FlightModeEditor::get_data().await.mode;
                if !matches!(current_mode, FlightModeEnum::Hovering) {
                    defmt::println!("hovering: exit hovering mode");
                    break;
                }

                // 控制频率50Hz
                Timer::after(Duration::from_millis(20)).await;
            }
        }
    }
}
