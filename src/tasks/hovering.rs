#![allow(unused)]

//! 悬停任务

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

/// 悬停任务
#[embassy_executor::task]
pub async fn hovering_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from hovering_task");
    
    loop{
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态才悬停
        if matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Hovering){
            // TODO 调整位姿保持水平
            
            
            // TODO 跟踪传感器高度数据和加速度数据, 适度调整电机转速
            
            
        }
    }// end loop
    
}
