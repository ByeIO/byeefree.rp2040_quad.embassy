#![allow(dead_code)]
#![allow(non_camel_case_types)]

//! 飞行命令

// 命令解析相关
use embedded_cli::Command;
use embedded_io::ErrorKind;
use embedded_io_async::{Read, Write};
use ufmt::uwrite;
use heapless::String;
use core::str::FromStr;
use crate::{shell::{UBuffer, INTERRUPT}, utils::signals::{self, FLIGHT_CHANNEL}};

// 异步相关
use embassy_time::{Duration, Ticker};
use embassy_futures::select::{select, Either};

// 内部库
use crate::utils::types::flight::{ FlightType, FlightModeEnum };
use crate::utils::variables::FlightModeEditor;

#[derive(Command, Clone)]
pub enum GoCommand {
    // 起飞命令
    /// take_off, `go take_off 100`
    Take_Off{
        /// 目标高度(cm)
        altitude: i16,
    },
    // 降落命令
    /// landing, `go landing`
    Landing,
    // 转圈命令
    /// circling in the air, `go circling 0 90`
    Circling{
        /// 半径(cm)
        radius: i16,
        /// 水平旋转角度(可用于机身旋转)
        angle: i16,
    },
    // 移动命令
    /// moving, `go moving forward 1`
    Moving{
        #[command(subcommand)]
        cmd: GoMovingCommand,
    },
    // 悬停命令
    Hovering,
}

// 实现CommandHandler特征
impl super::CommandHandler for GoCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            GoCommand::Take_Off { altitude } => {
                defmt::println!("go taking off...");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::TakeOff;
                flight_mode_wanted.altitude = *altitude;
                
                // 通过飞行任务管理器调用起飞任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
                serial.write_all(b"ok").await?;
            },// end Take_Off
            GoCommand::Landing => {
                defmt::println!("go landing...");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Landing;
                
                // 通过飞行任务管理器调用降落任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
                serial.write_all(b"ok").await?;
            },
            GoCommand::Circling { radius, angle } =>{
                defmt::println!("go circuling...");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Circling;
                flight_mode_wanted.radius = *radius;
                flight_mode_wanted.angle = *angle;
                
                // 通过飞行任务管理器调用绕圈任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
                serial.write_all(b"ok").await?;
            },
            GoCommand::Moving { cmd } =>{ 
                // 调用子命令处理函数
                cmd.handler(&mut serial).await?;
                
                serial.write_all(b"ok").await?;
            },
            GoCommand::Hovering => {
                defmt::println!("go hovering...");
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Hovering;
                
                // 通过飞行任务管理器调用悬停任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
                // 打印成功
                serial.write_all(b"ok").await?;
            },
        }// end self
        Ok(())
    }
}

/// 空中移动命令
#[derive(Command, Clone)]
pub enum GoMovingCommand{
    // 前进
    /// `go moving forward 1000`
    Forward{
        /// 移动毫秒数(不用距离, 精度不足)
        time: i16,
    },
    // 后退
    /// `go moving backward 1000`
    Backward{
        /// 移动秒数(不用距离, 精度不足)
        time: i16,
    },
    // 左移
    /// `go moving left 1000`
    Left{
        /// 移动秒数(不用距离, 精度不足)
        time: i16,
    },
    // 右移
    /// `go moving right 1000`
    Right{
        /// 移动秒数(不用距离, 精度不足)
        time: i16,
    },
}

// 实现CommandHandler特征
impl super::CommandHandler for GoMovingCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            GoMovingCommand::Forward { time } => {
                defmt::println!("go moving forward");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Forward;
                
                // 持续时间
                flight_mode_wanted.time = *time;
                
                // 通过飞行任务管理器执行前进任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
            },
            GoMovingCommand::Backward { time } => {
                defmt::println!("go moving backward");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Backward;
                
                // 持续时间
                flight_mode_wanted.time = *time;
                
                // 通过飞行任务管理器执行后退任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
                
            },
            GoMovingCommand::Left { time } => {
                defmt::println!("go moving left");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Left;
                
                // 通过飞行任务管理器执行左移任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
            },
            GoMovingCommand::Right { time } => {
                defmt::println!("go moving right");
                
                // 构造任务参数
                let mut flight_mode_wanted = FlightModeEditor::get_data().await;
                flight_mode_wanted.mode = FlightModeEnum::Right;
                
                // 持续时间
                flight_mode_wanted.time = *time;
                
                // 通过飞行任务管理器执行右移任务
                signals::FLIGHT_CHANNEL.publisher().unwrap().publish_immediate(flight_mode_wanted);
            },
        }// end self
        Ok(())
    }
}
