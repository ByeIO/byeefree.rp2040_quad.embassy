#![allow(unused)]

//! 滤波器任务

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

/// 滤波器任务调度器
#[embassy_executor::task]
pub async fn filter_task(
    mut in_raw_data : signals::ImuReadingSub,
    out_filtered_data : signals::ImuFilterPub,
){
    defmt::println!("hello from tasks::filter_task");
    loop{
        // 滤波
        let imu_data = baro_altitude_fusion_filter().await;
        
        // 发布
        signals::IMU_FILTER.publisher().unwrap().publish_immediate(imu_data);
        
        // 2Hz
        Timer::after(Duration::from_millis(500)).await;
    }
}
