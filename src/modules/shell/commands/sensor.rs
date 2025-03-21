#![allow(non_camel_case_types)]
#![allow(unused_mut)]

//! 解析传感器命令

// 命令解析相关
use embedded_cli::Command;
use embedded_io::ErrorKind;
use embedded_io_async::{Read, Write};
use ufmt::uwrite;
use heapless::String;
use core::str::FromStr;
use crate::{shell::{UBuffer, INTERRUPT}, utils::variables::Imu10DofDataEditor};

// 异步相关
use embassy_time::{Duration, Ticker};
use embassy_futures::select::{select, Either};

#[derive(Command, Clone)]
pub enum SensorCommand {
    Get{
        #[command(subcommand)]
        cmd: SensorGetEnum,
    },
    Set{
        #[command(subcommand)]
        cmd: SensorSetEnum,
    },
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
           SensorCommand::Get { cmd } =>{
               cmd.handler(&mut serial).await?
           },
           SensorCommand::Set { cmd } => {
               cmd.handler(&mut serial).await?
            }
           }// end match
        Ok(())
    }
}

// 获取传感器数据命令枚举
#[derive(Command, Clone)]
pub enum SensorGetEnum{
    // 十轴传感器
    Imu,
    // 光流传感器
    Optical_Flow,
    // 距离传感器
    Tof,
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorGetEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorGetEnum::Imu =>{
                // 非阻塞获取IMU数据
                let mut sub : crate::signals::ImuReadingSub = crate::signals::IMU_READING.subscriber().unwrap();
                
                // FIXME: 暂时使用全局变量进行数据交换
                use crate::utils::types::sensor::Imu10DofData;
                let imu_data : Imu10DofData<f32> = Imu10DofDataEditor::get_data().await;
                
                // let data = match sub.try_next_message_pure() {
                //     Some(data) => data,
                //     None => {
                //         defmt::error!("No IMU data available");
                //         return Ok(());
                //     }
                // };

                // 使用UBuffer格式化数据
                let mut buf = UBuffer::<128>::new();
                
                uwrite!(&mut buf, "imu: ")?;

                // 格式化陀螺仪数据
                for (i, &val) in imu_data.gyr.iter().enumerate() {
                    if i > 0 {
                        uwrite!(&mut buf, " ")?;
                    }
                    uwrite!(&mut buf, "{}", val as u64)?;
                }
                uwrite!(&mut buf, " ")?;

                // 格式化加速度计数据
                for (i, &val) in imu_data.acc.iter().enumerate() {
                    if i > 0 {
                        uwrite!(&mut buf, " ")?;
                }
                    uwrite!(&mut buf, "{}", val as u64)?;
                }
                uwrite!(&mut buf, " ")?;

                // 格式化磁力计数据
                for (i, &val) in imu_data.mag.iter().enumerate() {
                    if i > 0 {
                        uwrite!(&mut buf, " ")?;
                    }
                    uwrite!(&mut buf, "{}", val as u64)?;
                }
                uwrite!(&mut buf, " ")?;

                // 格式化气压数据
                for &val in imu_data.pressure.iter() {
                    uwrite!(&mut buf, "{}", val as u64)?;
                }

                // 打印调试信息并发送数据
                defmt::println!(
                    "IMU: {}",
                    core::str::from_utf8(&buf.inner).unwrap_or("Invalid UTF-8")
                );
                serial.write_all(&buf.inner).await?;
               
            },
            SensorGetEnum::Optical_Flow => {
              // TODO 获取传感器数据并打印
            },
            SensorGetEnum::Tof => {
              // TODO 获取传感器数据并打印
            },
           }// end match
        Ok(())
    }
}

// 配置传感器命令枚举
#[derive(Command, Clone)]
pub enum SensorSetEnum{
    // 配置十轴传感器
    Imu{
        #[command(subcommand)]
        cmd: SensorSetImuEnum,
    },
    // 配置光流传感器
    Optical_Flow{
        #[command(subcommand)]
        cmd: SensorSetOpticalFlowEnum,
    },
    // 配置距离传感器
    Tof{
        #[command(subcommand)]
        cmd: SensorSetTofEnum,
    },
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorSetEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorSetEnum::Imu { cmd } => {
                // 传给子程序继续解析
                cmd.handler(&mut serial).await?
            },
            SensorSetEnum::Optical_Flow { cmd } => {
                // 传给子程序继续解析
                cmd.handler(&mut serial).await?
            },
            SensorSetEnum::Tof { cmd } => {
                // 传给子程序继续解析
                cmd.handler(&mut serial).await?
            },
           }// end match
        Ok(())
    }
}

// 配置N轴复合传感器模块命令枚举
#[derive(Command, Clone)]
pub enum SensorSetImuEnum{
    // 参数设置
    Flags{
        // 数据更新频率
        /// The update frequency
        #[arg(short = 'f', long)]
        frequency: Option<u8>,
        
        // 串口波特率
        /// uart baudrate
        #[arg(short = 'b', long)]
        baudrate: Option<u32>
        
    },
    
    // 重启
    Restart,
    
    // 标定, 校准
    Calibrate,
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorSetImuEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorSetImuEnum::Flags { frequency, baudrate } => {
                // TODO 配置参数
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
            SensorSetImuEnum::Restart => {
                // TODO 重启传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            }, 
            
            SensorSetImuEnum::Calibrate => {
                // TODO 校准/标定传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
           }// end match
        Ok(())
    }
}

// 配置光流传感器命令枚举, `sensor set optical_flow flags -f 100 -b 115200
#[derive(Command, Clone)]
pub enum SensorSetOpticalFlowEnum{
    // 参数设置
    Flags{
        // 数据更新频率
        /// The update frequency
        #[arg(short = 'f', long)]
        frequency: Option<u8>,
        
        // 串口波特率
        /// uart baudrate
        #[arg(short = 'b', long)]
        baudrate: Option<u32>
        
    },
    
    // 重启
    Restart,
    
    // 标定, 校准
    Calibrate,
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorSetOpticalFlowEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorSetOpticalFlowEnum::Flags { frequency, baudrate } => {
                // TODO 配置参数
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
            SensorSetOpticalFlowEnum::Restart => {
                // TODO 重启传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            }, 
            
            SensorSetOpticalFlowEnum::Calibrate => {
                // TODO 校准/标定传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
           }// end match
        Ok(())
    }
}

// 配置距离传感器命令枚举
#[derive(Command, Clone)]
pub enum SensorSetTofEnum{
    // 参数设置
    Flags{
        // 数据更新频率
        /// The update frequency
        #[arg(short = 'f', long)]
        frequency: Option<u8>,
        
        // 串口波特率
        /// uart baudrate
        #[arg(short = 'b', long)]
        baudrate: Option<u32>
        
    },
    
    // 重启
    Restart,
    
    // 标定, 校准
    Calibrate,
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorSetTofEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorSetTofEnum::Flags { frequency, baudrate } => {
                // TODO 配置参数
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
            SensorSetTofEnum::Restart => {
                // TODO 重启传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            }, 
            
            SensorSetTofEnum::Calibrate => {
                // TODO 校准/标定传感器
                
                // 成功
                serial.write_all(b"ok").await?;
            },
            
           }// end match
        Ok(())
    }
}
