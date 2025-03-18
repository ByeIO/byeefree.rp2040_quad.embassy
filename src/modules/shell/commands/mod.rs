use core::future::Future;

use embedded_io::ErrorKind;
use embedded_io_async::{Read, Write};

// 使用中断
use crate::shell::{UBuffer, INTERRUPT};

// 1. hello命令
pub mod hello;

// 2. blink命令
pub mod blink;

// 3. mavlink2命令
pub mod mavlink2;

// 4. calbrate命令
pub mod calibrate;

/// 命令处理器特征
pub trait CommandHandler {
    fn handler(&self, serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>) -> impl Future<Output = Result<(), ErrorKind>>;
}
