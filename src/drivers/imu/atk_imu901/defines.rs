#![allow(dead_code)]
#![allow(non_snake_case)]

//! 正点原子ATK-IMU901十轴传感器驱动常量及结构体定义

/* start 常量 */
/// UART接收FIFO缓冲大小
pub const ATK_MS901M_UART_RX_FIFO_BUF_SIZE : usize = 128;
/// UART通讯帧数据最大长度
pub const ATK_MS901M_FRAME_DAT_MAX_SIZE : usize = 28;
/// 帧类型: 主动上传帧
pub const ATK_MS901M_FRAME_ID_TYPE_UPLOAD : u8 = 0;
/// 帧类型: 应答帧
pub const ATK_MS901M_FRAME_ID_TYPE_ACK : u8 = 1;

/// 读取寄存器ID
pub const fn ATK_MS901M_READ_REG_ID(id: u8) -> u8 {
    id | 0x80
}

/// 写入寄存器ID
pub const fn ATK_MS901M_WRITE_REG_ID(id: u8) -> u8 {
    id
}

/// 陀螺仪、加速度计满量程表
pub static G_ATK_MS901M_GYRO_FSR_TABLE: [u16; 4] = [250, 500, 1000, 2000];
pub static G_ATK_MS901M_ACCELEROMETER_FSR_TABLE: [u8; 4] = [2, 4, 8, 16];

/// 通讯帧头
pub const ATK_MS901M_FRAME_HEAD_L : u8 = 0x55;
// 高位主动上传帧头
pub const ATK_MS901M_FRAME_HEAD_UPLOAD_H : u8 = 0x55;
// 高位应答帧头
pub const ATK_MS901M_FRAME_HEAD_ACK_H : u8 = 0xAF;
/* end 常量 */

/* start 枚举 */
/// 主动上传帧ID枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mFrameUploadId {
    /// 姿态角
    Attitude = 0x01,
    /// 四元数
    Quat = 0x02,
    /// 陀螺仪、加速度计
    GyroAcce = 0x03,
    /// 磁力计
    Mag = 0x04,
    /// 气压计
    Baro = 0x05,
    /// 端口
    Port = 0x06,
}

impl From<AtkMs901mFrameUploadId> for u8 {
    fn from(val: AtkMs901mFrameUploadId) -> Self {
        val as u8
    }
}

impl TryFrom<u8> for AtkMs901mFrameUploadId {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(AtkMs901mFrameUploadId::Attitude),
            0x02 => Ok(AtkMs901mFrameUploadId::Quat),
            0x03 => Ok(AtkMs901mFrameUploadId::GyroAcce),
            0x04 => Ok(AtkMs901mFrameUploadId::Mag),
            0x05 => Ok(AtkMs901mFrameUploadId::Baro),
            0x06 => Ok(AtkMs901mFrameUploadId::Port),
            _ => Err(()),
        }
    }
}

/// 应答帧ID枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mFrameAckId {
    /// 保存当前配置到Flash
    RegSave = 0x00,
    /// 设置传感器校准
    RegSenCal = 0x01,
    /// 读取传感器校准状态
    RegSenSta = 0x02,
    /// 设置陀螺仪量程
    RegGyroFsr = 0x03,
    /// 设置加速度计量程
    RegAccFsr = 0x04,
    /// 设置陀螺仪带宽
    RegGyroBw = 0x05,
    /// 设置加速度计带宽
    RegAccBw = 0x06,
    /// 设置UART通讯波特率
    RegBaud = 0x07,
    /// 设置回传内容
    RegReturnSet = 0x08,
    /// 设置回传内容2（保留）
    RegReturnSet2 = 0x09,
    /// 设置回传速率
    RegReturnRate = 0x0A,
    /// 设置算法
    RegAlg = 0x0B,
    /// 设置安装方向
    RegAsm = 0x0C,
    /// 设置陀螺仪自校准开关
    RegGauCal = 0x0D,
    /// 设置气压计自校准开关
    RegBauCal = 0x0E,
    /// 设置LED开关
    RegLedOff = 0x0F,
    /// 设置端口D0模式
    RegD0Mode = 0x10,
    /// 设置端口D1模式
    RegD1Mode = 0x11,
    /// 设置端口D2模式
    RegD2Mode = 0x12,
    /// 设置端口D3模式
    RegD3Mode = 0x13,
    /// 设置端口D1 PWM高电平脉宽
    RegD1Pulse = 0x16,
    /// 设置端口D3 PWM高电平脉宽
    RegD3Pulse = 0x1A,
    /// 设置端口D1 PWM周期
    RegD1Period = 0x1F,
    /// 设置端口D3 PWM周期
    RegD3Period = 0x23,
    /// 恢复默认设置
    RegReset = 0x7F,
}

/// uart通讯帧头枚举, 
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mFrameHeader {
    /// 低位
    // L = 0x55,
    
    // 主动上传帧头: 0x55 0x55
    /// 高位主动上传帧头
    UploadH = 0x55,
    
    // 应答帧头: 0x55 0xAF
    /// 高位应答帧头
    AckH = 0xAF,
}

/// LED状态枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mLedState {
    /// LED灯关闭
    On = 0x00,
    /// LED灯打开
    Off = 0x01,
}

/// 端口枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mPort {
    /// 端口D0
    D0 = 0x00,
    /// 端口D1
    D1 = 0x01,
    /// 端口D2
    D2 = 0x02,
    /// 端口D3
    D3 = 0x03,
}

/// 端口模式枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mPortMode {
    /// 模拟输入
    AnalogInput = 0x00,
    /// 数字输入
    Input = 0x01,
    /// 输出数字高电平
    OutputHigh = 0x02,
    /// 输出数字低电平
    OutputLow = 0x03,
    /// 输出PWM
    OutputPwm = 0x04,
}

/// 错误代码枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mError {
    /// 没有错误
    Ok = 0,
    /// 错误
    Error = 1,
    /// 错误函数参数
    Invalid = 2,
    /// 超时错误
    Timeout = 3,
    /// 校验和错误类型
    Checksum = 4,  
    /// UART错误类型
    UartError = 5, 
}

impl From<embassy_rp::uart::Error> for AtkMs901mError {
    fn from(_: embassy_rp::uart::Error) -> Self {
        AtkMs901mError::UartError
    }
}
/* end 枚举 */

/* start 结构体 */
/// 姿态角数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901maltitudeData {
    /// 横滚角，单位：°
    pub roll: f32,
    /// 俯仰角，单位：°
    pub pitch: f32,
    /// 航向角，单位：°
    pub yaw: f32,
}

/// 四元数数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mQuaternionData {
    /// Q0
    pub q0: f32,
    /// Q1
    pub q1: f32,
    /// Q2
    pub q2: f32,
    /// Q3
    pub q3: f32,
}

/// 陀螺仪数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mGyroData {
    /// 原始数据
    pub raw: AtkMs901mRawData,
    /// X轴旋转速率，单位：dps
    pub x: f32,
    /// Y轴旋转速率，单位：dps
    pub y: f32,
    /// Z轴旋转速率，单位：dps
    pub z: f32,
}

/// 加速度计数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mAccelerometerData {
    /// 原始数据
    pub raw: AtkMs901mRawData,
    /// X轴加速度，单位：G
    pub x: f32,
    /// Y轴加速度，单位：G
    pub y: f32,
    /// Z轴加速度，单位：G
    pub z: f32,
}

/// 磁力计数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mMagnetometerData {
    /// X轴磁场强度
    pub x: i16,
    /// Y轴磁场强度
    pub y: i16,
    /// Z轴磁场强度
    pub z: i16,
    /// 温度，单位：℃
    pub temperature: f32,
}

/// 气压计数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mBarometerData {
    /// 气压，单位：Pa
    pub pressure: i32,
    /// 海拔，单位：cm
    pub altitude: i32,
    /// 温度，单位：℃
    pub temperature: f32,
}

/// 端口数据结构体
#[derive(Debug, Clone, Copy)]
pub struct AtkMs901mPortData {
    /// 端口D0数据
    pub d0: u16,
    /// 端口D1数据
    pub d1: u16,
    /// 端口D2数据
    pub d2: u16,
    /// 端口D3数据
    pub d3: u16,
}

/// 原始数据结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mRawData {
    /// X轴原始数据
    pub x: i16,
    /// Y轴原始数据
    pub y: i16,
    /// Z轴原始数据
    pub z: i16,
}

/// ATK-MS901M满量程数据
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mFsr {
    /// 陀螺仪满量程
    pub gyro: u8,
    /// 加速度计满量程
    pub accelerometer: u8,
}

/// UART通讯帧结构体
#[derive(Debug, Clone, Copy, Default)]
pub struct AtkMs901mFrame {
    /// 低位帧头
    pub head_l: u8,
    /// 高位帧头
    pub head_h: u8,
    /// 帧ID
    pub id: u8,
    /// 数据长度
    pub len: u8,
    /// 数据
    pub dat: [u8; ATK_MS901M_FRAME_DAT_MAX_SIZE],
    /// 校验和
    pub check_sum: u8,
}

/// 帧处理状态机状态枚举
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtkMs901mHandleState {
    /// 等待低位帧头
    WaitForHeadL = 0x00,
    /// 等待高位帧头
    WaitForHeadH = 0x01,
    /// 等待帧ID
    WaitForId = 0x02,
    /// 等待数据长度
    WaitForLen = 0x04,
    /// 等待数据
    WaitForDat = 0x08,
    /// 等待校验和
    WaitForSum = 0x16,
}
/* end 结构体 */
