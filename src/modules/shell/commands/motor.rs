// 命令解析相关
use embedded_cli::Command;
use embedded_io::ErrorKind;
use embedded_io_async::{Read, Write};
use ufmt::uwrite;
use heapless::String;
use core::str::FromStr;
use crate::shell::{UBuffer, INTERRUPT};

// 异步相关
use embassy_time::{Duration, Ticker};
use embassy_futures::select::{select, Either};

// 内部库
/// 信号
use crate::utils::signals;
/// 全局变量
use crate::utils::variables;

#[derive(Command, Clone)]
pub enum MotorCommand {
    /// set speed and direction
    Set {
        #[command(subcommand)]
        cmd: MotorSetEnum,
    },
    /// get speed and direction
    Get,
}

// 实现CommandHandler特征
impl super::CommandHandler for MotorCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            // 具体解析命令
            MotorCommand::Set { cmd } => {
                cmd.handler(&mut serial).await?
            },
            MotorCommand::Get => {
                let speeds = crate::utils::variables::MotorSpeedEditor::get_speeds().await;
                let directions = crate::utils::variables::MotorDirectionEditor::get_directions().await;
                // 使用UBuffer结构体
                let mut buf = UBuffer::<32>::new();
                
                // 遍历数组元素逐个写入
                for (i, speed) in speeds.as_ref().iter().enumerate() {
                    if i > 0 {
                        // 添加分隔符
                        uwrite!(&mut buf, " ")?; 
                    }
                    uwrite!(&mut buf, "{}", speed)?;
                }
                
                // 加一个空格
                uwrite!(&mut buf, " ")?;
                
                // 调试信息打印
                defmt::println!("motor speed: {}", core::str::from_utf8(&buf.inner).unwrap());
                // 写入到输出接口
                serial.write_all(&buf.inner).await?;
                
                // 使用UBuffer结构体
                let mut buf2 = UBuffer::<32>::new();
                
                // 遍历数组元素逐个写入
                for (i, direction) in directions.as_ref().iter().enumerate() {
                    if i > 0 {
                        // 添加分隔符
                        uwrite!(&mut buf2, " ")?; 
                    }
                    uwrite!(&mut buf2, "{}", direction)?;
                }
                
                // 调试信息打印
                defmt::println!("motor directions: {}", core::str::from_utf8(&buf2.inner).unwrap());
                // 写入到输出接口
                serial.write_all(&buf2.inner).await?;
            }
        }
        Ok(())
    }
}

#[derive(Command, Clone)]
pub enum MotorSetEnum{
    // 设置速度, `motor set speed 1 100`
    /// set speed
    Speed{
        motor: usize,
        speed: u16,
    },
    // 设置方向, `motor set dir 1 false`
    /// set direction
    Dir{
        motor: usize,
        direction: bool,
    }
}

// 实现CommandHandler特征
impl super::CommandHandler for MotorSetEnum {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            MotorSetEnum::Speed{motor, speed}=>{
                // 转换电机模式到RUNNING
                signals::MOTOR_STATE.publisher().unwrap().publish_immediate(crate::tasks::motors::MotorState::MANUAL);
                
                // TODO 检查数值合法性
                
                // 获取全局变量的电机转速
                let mut speeds = crate::utils::variables::MotorSpeedEditor::get_speeds().await;
                speeds[motor-1] = *speed;
                
                // 发布电机转速设置
                signals::MOTOR_SPEED.publisher().unwrap().publish_immediate(
                    (Some(speeds[0]),Some(speeds[1]),Some(speeds[2]),Some(speeds[3]))
                );
                
                serial.write_all(b"ok").await?;
            },
            MotorSetEnum::Dir{motor, direction}=>{
                
                // TODO 检查数值合法性
                
                // 获取全局变量的电机方向
                let mut directions = crate::utils::variables::MotorDirectionEditor::get_directions().await;
                directions[motor-1] = *direction;
                
                // 发布电机方向设置
                signals::MOTOR_DIR.publisher().unwrap().publish_immediate(
                    (Some(directions[0]),Some(directions[1]),Some(directions[2]),Some(directions[3]))
                );
                
                serial.write_all(b"ok").await?;
            }
        }
        Ok(())
    }
}
