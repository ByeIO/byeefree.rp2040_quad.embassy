#![allow(non_camel_case_types)]

//! 传感器数据结构

use num_traits::Num;
use serde::{Deserialize, Serialize};

// 10轴传感器数据结构
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[derive(defmt::Format)]
pub struct Imu10DofData<f32> {
    pub gyr: [f32; 3],
    pub acc: [f32; 3],
    pub mag: [f32; 3],
    pub pressure: [f32; 1],
}
