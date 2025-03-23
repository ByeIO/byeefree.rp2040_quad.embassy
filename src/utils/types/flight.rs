#![allow(dead_code)]

//! 飞行状态类型

use crate::types::sensor::SensorData;

// 当前飞行状态枚举
#[derive(Clone,Copy)]
pub enum FlightModeEnum {
    // 待命状态
    Ready,
    // 起飞中
    TakeOff,
    // 悬停中
    Hovering,
    // 降落中
    Landing,
    // 空中移动中
    // Moving,
    // 前进中,
    Forward,
    // 后退中,
    Backward,
    // 左移中,
    Left,
    // 右移中
    Right,
    // 旋转中
    Circling,
}

// 当前飞行状态及其参数
#[derive(Clone,Copy)]
pub struct FlightType {
    /// 飞行模式
    pub mode : FlightModeEnum,
    // 传入的参数
    /// 期望移动时长
    pub time: i16,
    /// 期望高度
    pub altitude: i16,
    /// 期望旋转角度
    pub angle: i16,
    /// 期望绕圈半径
    pub radius: i16,
}
