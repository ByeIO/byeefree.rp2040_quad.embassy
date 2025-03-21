#![allow(non_camel_case_types)]

//! 传感器数据结构

use num_traits::Num;
use serde::{Deserialize, Serialize};

// 10轴传感器数据结构
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
#[derive(defmt::Format)]
pub struct Imu10DofData<f32> {
    /// 10轴数据
    // 陀螺仪
    pub gyr: [f32; 3],
    // 加速度
    pub acc: [f32; 3],
    // 磁力计
    pub mag: [f32; 3],
    // 气压计
    pub pressure: [f32; 1],
    
    /// 10轴+2轴数据
    // 四元数
    pub quat: [f32; 4],
    // 海拔高度
    pub attitude: [f32; 1],
    // 温度
    pub temperature: [f32; 1],
}
