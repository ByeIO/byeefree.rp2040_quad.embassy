#![allow(unused)]

//! 起飞任务

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
            
            // 循环直到大于等于期望高度
            loop{
                // 获取当前高度(cm)
                let altitude_now = variables::Imu10DofDataEditor::get_data().await.altitude[0] as i16;
                
                if altitude_now < altitude_wanted {
                    // TODO 调用控制器任务使位姿保持水平
                    
                    
                    // TODO 跟踪传感器高度数据和加速度数据, 继续调用控制器任务使X型四旋翼无人机离地起飞
                    
                    
                }// end if
                else{
                    break;
                }
            }// end loop
            
            // 起飞成功, 切换到悬停状态
            flight_mode_wanted.mode = FlightModeEnum::Hovering;
            signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
            
        }// end if matches
    }// end loop
    
}
