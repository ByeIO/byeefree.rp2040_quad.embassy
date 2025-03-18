//! 嵌入式CLI命令行接口实现

// 命令解析器相关
use commands::CommandHandler;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embedded_cli::{cli::CliBuilder, Command};
use embedded_cli::arguments::{ FromArgument, FromArgumentError};
use embedded_io::{ErrorKind, ErrorType, Write as SyncWrite};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use ufmt::{uWrite, uDisplay};

// 强制多态
use core::ops::Deref;
use core::ops::DerefMut;

// 1. 适配器
mod usb_adapter;
pub use usb_adapter::CdcAcmIO;
mod sync_writer;
pub use sync_writer::SyncWriter;
mod ubuffer;
pub use ubuffer::UBuffer;

// 2. 命令定义
pub mod commands;

/// 运行命令行接口的主函数, 需要serial实现SERIAL特征
pub async fn run_cli<SERIAL: Read<Error = ErrorKind> + Write<Error = ErrorKind>>(
    serial: SERIAL,
) -> Result<(), ErrorKind> {
    // 初始化命令历史和当前命令缓冲区
    let (command_buffer, history_buffer) = {
        static COMMAND_BUFFER: StaticCell<[u8; 32]> = StaticCell::new();
        static HISTORY_BUFFER: StaticCell<[u8; 32]> = StaticCell::new();
        (COMMAND_BUFFER.init([0; 32]), HISTORY_BUFFER.init([0; 32]))
    };

    // 使用互斥锁保护串口设备
    let mutexed_serial = Mutex::new(serial);

    // 等待终端连接就绪
    Timer::after_secs(1).await;

    // 初始化时清屏并显示启动LOGO
    {
        let mut m_serial = mutexed_serial.lock().await;
        m_serial.write_all(CLEAR_SCREEN).await?;
        m_serial.write_all(LOGO_GRAPHIC).await?;
    }

    // 构建cli实例
    let mut sync_writer = SyncWriter::new(&mutexed_serial);
    let mut cli = CliBuilder::default()
        .writer(&mut sync_writer)
        .command_buffer(command_buffer.as_mut_slice())
        .history_buffer(history_buffer.as_mut_slice())
        .prompt(CMD_PROMPT)
        .build()?;

    // 主事件循环处理输入
    let mut buffer = [0u8; 8];
    loop {
        let n = {
            let mut m_serial = mutexed_serial.lock().await;
            let Ok(n) = m_serial.read(&mut buffer).await else {
                Timer::after_millis(100).await;
                continue;
            };
            n
        };

        defmt::println!("Read {} bytes from usb : {:?}", n, buffer[..n]);

        // 处理每个输入字节
        for byte in buffer.iter().take(n) {
            let mut parsed_command = None;
            cli.process_byte::<Base, _>(
                *byte,
                &mut Base::processor(|_, command| {
                    parsed_command = Some(command);
                    Ok(())
                }),
            )?;

            // 命令解析成功后的处理
            if let Some(command) = parsed_command {
                let mut m_serial = mutexed_serial.lock().await;
                let mut serial = m_serial.deref_mut();

                // 清空当前行并回车
                serial.write_all(CLEAR_LINE).await?;
                serial.write_all(b"\r").await?;

                // 执行对应命令的处理函数
                match command {
                    // Base::Cal { cmd } => cmd.handler(&mut serial).await?,
                    // Base::Sys { cmd } => cmd.handler(&mut serial).await?,
                    // Base::Mavlink2 { cmd } => cmd.handler(&mut serial).await?,
                    Base::Clear => serial.write_all(CLEAR_SCREEN).await?,
                    Base::Logo => serial.write_all(LOGO_GRAPHIC).await?,
                    Base::Hello { cmd }=> cmd.handler(&mut serial).await?,
                }

                // 命令执行完成后重新显示提示符
                serial.write_all(CMD_PROMPT.as_bytes()).await?;
            }
        }
    }
}

// 终端控制序列
/// 清屏指令
const CLEAR_SCREEN: &[u8] = b"\x1B[2J";  
/// 清除当前行
const CLEAR_LINE: &[u8] = b"\x1B[2K";    
/// 命令行提示符
const CMD_PROMPT: &str = "$ ";           
/// Ctrl+C中断字符
const INTERRUPT: &u8 = &0x03;            
/// 启动LOGO
const LOGO_GRAPHIC: &[u8] =          
b"\x1B[32m
\r  (=^_^=)
\x1B[0m";

/// 基础命令枚举, (`///`的文档注释会形成命令帮助文件, 所以只能使用英文)
#[derive(Command)]
#[command(help_title = "Basic commands")]
enum Base {
    
    // 1. 清空终端屏幕
    /// Clear the shell output, `clear`
    Clear,

    // 2. 显示启动LOGO, 
    /// Show the logo graphic, `logo`
    Logo,

    // // 3. 校准相关子命令
    // /// Calibration commands, `calibrate`
    // Calibrate {
    //     #[command(subcommand)]
    //     cmd: commands::calibrate::CalibrateCommand,
    // },

    // 4. 系统管理子命令
    // /// System management commands, `system`
    // System {
    //     #[command(subcommand)]
    //     cmd: commands::system::SystemCommand,
    // },

    // // 5. mavlink2通信相关子命令
    // /// Mavlink_v2 commands, `mavlink2`
    // Mavlink2 {
    //     #[command(subcommand)]
    //     cmd: commands::mavlink2::MavlinkCommand,
    // },
    
    // 6. hello子命令(模板)
    /// Hello commands, `hello`
    Hello{
        #[command(subcommand)]
        cmd: commands::hello::HelloCommand,
    },
}
