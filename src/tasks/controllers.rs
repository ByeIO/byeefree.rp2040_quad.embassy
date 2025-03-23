#![allow(unused)]

//! 姿态控制器任务

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
/// 姿态控制器
use crate::modules::controllers::lqr::lqr_controller;

/// 姿态控制任务调度器
#[embassy_executor::task]
pub async fn controller_task(
   // 订阅期望达到的位姿(四元数)
   mut in_quat_wanted : signals::ControllerSub,
){
    defmt::println!("hello from controller_task");
    loop{
        // 1. 获取期望姿态和当前状态
        /// 获取期望位姿
        let quat_wanted = in_quat_wanted.next_message_pure().await.quat;
        /// 获取当前位姿, TODO 获取滤波后当前位姿
        let quat_now = variables::Imu10DofDataEditor::get_data().await.quat;
        
        // 期望位姿和当前位姿不同
        if quat_wanted != quat_now {
            //  2. 调用位姿控制算法
            lqr_controller(quat_wanted).await;
        }
        
        // 留出CPU资源给其他任务
        Timer::after(Duration::from_millis(1)).await;
    }
}
