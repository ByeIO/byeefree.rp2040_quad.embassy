#![allow(unused)]

//! 飞行模式管理任务

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

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

/// 飞行任务启动器
pub async fn flight_task_launcher(
    // 任务调度器
    spawner: &Spawner,
){
    defmt::println!("hello from flight_task_launcher");
    // // 启动飞行模式管理任务
    // spawner.must_spawn(flight_task(
    //     signals::FLIGHT_CHANNEL.subscriber().unwrap(),
    // ));
    
    // 启动姿态控制器任务
    use crate::tasks::controllers::controller_task;
    spawner.must_spawn(controller_task(
        signals::CONTROLLER_CHANNEL.subscriber().unwrap(),
    ));
    
    // 启动各个移动任务
    use crate::tasks::moving::moving_task_launcher;
    moving_task_launcher(&spawner).await;
    
    // 启动起飞任务并监听
    use crate::tasks::take_off::take_off_task;
    spawner.must_spawn(take_off_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // 启动降落任务并监听
    use crate::tasks::landing::landing_task;
    spawner.must_spawn(landing_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // 启动悬停任务并监听
    use crate::tasks::hovering::hovering_task;
    spawner.must_spawn(hovering_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // 启动绕圈任务并监听
    use crate::tasks::circling::circling_task;
    spawner.must_spawn(circling_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
}

// /// FIXME: 飞行模式管理任务(好像不需要用到🤔)
// #[embassy_executor::task]
// pub async fn flight_task(
//     // 期望达到的飞行模式及对应参数
//     mut in_flight_mode_wanted : signals::FlightSub,
// ){
//     defmt::println!("hello from flight_task");
//     loop{
//         // 尝试获取消息
//         let flight_mode_wanted = in_flight_mode_wanted.next_message_pure().await;
        
//         // 从全局变量获取当前飞行模式
//         let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
//         match flight_mode_wanted.mode{
//             // 期望起飞
//             FlightModeEnum::TakeOff =>{
//                 // 当前状态为READY状态才可起飞
//                 if matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 期望高度
//                     let altitude_wanted = flight_mode_wanted.altitude;
//                     // TODO 调用起飞任务
                    
//                 }
//             },
//             // 期望降落
//             FlightModeEnum::Landing => {
//                 // 当前状态不为READY状态才执行降落
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // TODO 调用降落任务
//                 }
//             },
//             // 期望前进
//             FlightModeEnum::Forward => {
//                 // 当前状态不为READY状态(在空中)才前进
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 移动持续时间(ms)
//                     let duration = flight_mode_wanted.time;
                    
//                     // TODO 调用前进任务
                    
                    
//                 }
//             },
//             // 期望后退
//             FlightModeEnum::Backward => {
//                 // 当前状态不为READY状态(在空中)才后退
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 移动持续时间(ms)
//                     let duration = flight_mode_wanted.time;
                    
//                     // TODO 调用后退任务
                    
                    
//                 }
//             },
//             // 期望左移
//             FlightModeEnum::Left => {
//                 // 当前状态不为READY状态(在空中)才左移
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 移动持续时间(ms)
//                     let duration = flight_mode_wanted.time;
                    
//                     // TODO 调用左移任务
                    
                    
//                 }
//             },
//             // 期望右移
//             FlightModeEnum::Right => {
//                 // 当前状态不为READY状态(在空中)才右移
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 移动持续时间(ms)
//                     let duration = flight_mode_wanted.time;
                    
//                     // TODO 调用右移任务
                    
                    
//                 }
//             },
//             // 期望绕圈
//             FlightModeEnum::Circling => {
//                 // 当前状态不为READY状态(在空中)才绕圈
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // 获取消息中的参数: 绕圈半径和角度
//                     let radius = flight_mode_wanted.radius;
//                     let angle = flight_mode_wanted.angle;
                    
//                     // TODO 调用绕圈任务
                    
//                 }
//             },
//             // 期望悬停
//             FlightModeEnum::Hovering => {
//                 // 当前状态不为READY状态(在空中)才悬停
//                 if !matches!(flight_mode_now, FlightModeEnum::Ready){
//                     // TODO 调用悬停任务
                    
//                 }
//             },
//             // 其余情况(没有了😂)
//             _ =>{
                
//             },
//         }// end match
        
//         // 留出CPU资源给其他任务
//         Timer::after(Duration::from_millis(1)).await;
//     }
// }
