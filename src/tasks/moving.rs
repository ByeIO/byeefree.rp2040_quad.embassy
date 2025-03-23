#![allow(unused)]

//! 前进后退左右移动任务

// 多任务相关
use embassy_time::{Duration, Timer, Instant};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// 数学计算
use nalgebra::{Quaternion, Vector3, UnitQuaternion};
use libm::{acos, sqrtf, powf, acosf};

// 宏编程
use paste::paste;

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

/// 空中移动任务启动器
pub async fn moving_task_launcher(
    spawner: &Spawner,
){
    defmt::println!("hello from moving_task_launcher");
    
    // // 启动前进任务并监听消息
    // spawner.must_spawn(moving_forward_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动后退任务并监听消息
    // spawner.must_spawn(moving_backward_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动左移消息并监听消息
    // spawner.must_spawn(moving_left_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动右移任务并监听消息
    // spawner.must_spawn(moving_right_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // 使用宏生成任务启动代码
    macro_rules! spawn_task {
        ($direction:ident) => {
            spawner.must_spawn(
                paste! {[<moving_ $direction _task>]}(
                    signals::FLIGHT_CHANNEL.subscriber().unwrap()
                )
            );
        };
    }

    spawn_task!(forward);
    spawn_task!(backward);
    spawn_task!(left);
    spawn_task!(right);
}

// /// 空中前进任务
// #[embassy_executor::task]
// pub async fn moving_forward_task(
//     // 飞行参数
//     mut in_flight_args : signals::FlightSub,
// ){
//     defmt::println!("hello from moving_forward_task");
//     loop{
//         // 尝试获取消息
//         let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
//         // 从全局变量获取当前飞行模式
//         let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
//         // 当前状态不为READY状态(在空中)才前进
//         if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Forward){
//             // 获取消息中的参数: 移动持续时间(ms)
//             let duration = flight_mode_wanted.time;
            
//             // 获取当前位姿, TODO 获取滤波后当前位姿
//             let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
//             let quat_now = attitude_original.quat;
            
//             // TODO 构造前进姿态的四元数
//             let mut attitude_wanted = Imu10DofData::<f32>::default();
//             let mut quat_wanted = attitude_wanted.quat;
            
//             // 调用姿态控制器
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
//             // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
//             Timer::after(Duration::from_millis(duration as u64)).await;
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
//         }// end if
        
//     }// end loop
    
// }

// /// 空中后退任务
// #[embassy_executor::task]
// pub async fn moving_backward_task(
//     // 飞行参数
//     mut in_flight_args : signals::FlightSub,
// ){
//     defmt::println!("hello from moving_backward_task");
    
//     loop{
//         // 尝试获取消息
//         let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
//         // 从全局变量获取当前飞行模式
//         let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
//         // 当前状态不为READY状态(在空中)才后退
//         if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Backward){
//             // 获取消息中的参数: 移动持续时间(ms)
//             let duration = flight_mode_wanted.time;
            
//             // 获取当前位姿, TODO 获取滤波后当前位姿
//             let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
//             let quat_now = attitude_original.quat;
            
//             // TODO 构造后退姿态的四元数
//             let mut attitude_wanted = Imu10DofData::<f32>::default();
//             let mut quat_wanted = attitude_wanted.quat;
            
//             // 调用姿态控制器
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
//             // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
//             Timer::after(Duration::from_millis(duration as u64)).await;
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
//         }// end if
        
//     }// end loop
    
// }

// /// 空中左移任务
// #[embassy_executor::task]
// pub async fn moving_left_task(
//     // 飞行参数
//     mut in_flight_args : signals::FlightSub,
// ){
//     defmt::println!("hello from moving_left_task");
    
//     loop{
//         // 尝试获取消息
//         let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
//         // 从全局变量获取当前飞行模式
//         let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
//         // 当前状态不为READY状态(在空中)才左移
//         if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Left){
//             // 获取消息中的参数: 移动持续时间(ms)
//             let duration = flight_mode_wanted.time;
            
//             // 获取当前位姿, TODO 获取滤波后当前位姿
//             let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
//             let quat_now = attitude_original.quat;
            
//             // TODO 构造左移姿态的四元数
//             let mut attitude_wanted = Imu10DofData::<f32>::default();
//             let mut quat_wanted = attitude_wanted.quat;
            
//             // 调用姿态控制器
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
//             // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
//             Timer::after(Duration::from_millis(duration as u64)).await;
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
//         }// end if
        
//     }// end loop
    
// }

// /// 空中右移任务
// #[embassy_executor::task]
// pub async fn moving_right_task(
//     // 飞行参数
//     mut in_flight_args : signals::FlightSub,
// ){
//     defmt::println!("hello from moving_right_task");
    
//     loop{
//         // 尝试获取消息
//         let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
//         // 从全局变量获取当前飞行模式
//         let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
//         // 当前状态不为READY状态(在空中)才右移
//         if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Right){
//             // 获取消息中的参数: 移动持续时间(ms)
//             let duration = flight_mode_wanted.time;
            
//             // 获取当前位姿, TODO 获取滤波后当前位姿
//             let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
//             let quat_now = attitude_original.quat;
            
//             // TODO 构造右移姿态的四元数
//             let mut attitude_wanted = Imu10DofData::<f32>::default();
//             let mut quat_wanted = attitude_wanted.quat;
            
//             // 调用姿态控制器
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
//             // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
//             Timer::after(Duration::from_millis(duration as u64)).await;
//             signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
//         }// end if
        
//     }// end loop
    
// }

/// 生成移动任务的宏
macro_rules! generate_moving_task {
    ($direction:ident, $axis:expr, $angle_sign:expr) => {
        paste! {
            #[embassy_executor::task]
            pub async fn [<moving_ $direction _task>](
                mut in_flight_args: signals::FlightSub,
            ) {
                defmt::println!("hello from moving_{}_task", stringify!($direction));
                
                // 姿态倾斜角度（5度）
                const TILT_ANGLE: f32 = 5.0;
                
                loop {
                    let flight_mode_wanted = in_flight_args.next_message_pure().await;
                    let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
                    
                    // 使用camel转为驼峰命名法
                    if !matches!(flight_mode_now, FlightModeEnum::Ready) 
                        && matches!(flight_mode_wanted.mode, paste!(FlightModeEnum::[<$direction:camel>]) ) 
                    {
                        // 获取持续时间参数
                        let duration = flight_mode_wanted.time;
                        
                        // 获取当前姿态
                        let imu_data = variables::Imu10DofDataEditor::get_data().await;
                        let q_current = Quaternion::from_parts(
                            imu_data.quat[0],
                            Vector3::new(
                                imu_data.quat[1],
                                imu_data.quat[2],
                                imu_data.quat[3],
                            )
                        );
                        
                        // 构造目标旋转四元数（绕对应轴旋转）
                        let theta = TILT_ANGLE.to_radians() * $angle_sign;
                        let q_rot = UnitQuaternion::from_axis_angle(&$axis, theta).into_inner();
                        
                        // 计算目标四元数（当前姿态叠加旋转）
                        let q_target = q_current * q_rot;
                        let target_quat = [
                            q_target.scalar(),
                            q_target.vector().x,
                            q_target.vector().y,
                            q_target.vector().z,
                        ];
                        
                        // 调用姿态控制器
                        lqr::lqr_controller(target_quat).await;
                        
                        // 保持倾斜姿态
                        Timer::after(Duration::from_millis(duration as u64)).await;
                        
                        // 恢复原姿态
                        lqr::lqr_controller(imu_data.quat).await;
                        
                        // 异常检测：当前倾斜角度
                        let tilt = calculate_tilt_angle(&q_current).await;
                        if tilt > MAX_ANGLE {
                            emergency_hover().await;
                            break;
                        }
                    }
                }
            }
        }
    };
}

// 生成四个方向的任务
generate_moving_task!(forward, Vector3::y_axis(), 1.0);
generate_moving_task!(backward, Vector3::y_axis(), -1.0);
generate_moving_task!(left, Vector3::x_axis(), 1.0);
generate_moving_task!(right, Vector3::x_axis(), -1.0);

/// 最大允许倾斜角度（度）
const MAX_ANGLE: f32 = 15.0;

/// 计算当前倾斜角度（单位：度）
pub async fn calculate_tilt_angle(quat: &Quaternion<f32>) -> f32 {
    let projected = (libm::powf(2.0, quat.w) - abs(quat.vector().norm_squared()));
    libm::acosf(libm::sqrtf(projected) / quat.norm()).to_degrees()
}

/// 紧急悬停程序
pub async fn emergency_hover() {
    defmt::error!("[紧急悬停] 检测到异常姿态");
    // 保持水平姿态
    lqr::lqr_controller([1.0, 0.0, 0.0, 0.0]).await;
    // 设置基础推力
    signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
        (Some(30), Some(30), Some(30), Some(30))
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
