use super::commands::CommandHandler;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embedded_cli::{cli::CliBuilder, Command};
use embedded_cli::arguments::{ FromArgument, FromArgumentError};
use embedded_io::{ErrorKind, ErrorType, Write as SyncWrite};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use ufmt::{uWrite, uDisplay};

/// 同步写入器包装器，用于适配嵌入式CLI的同步写入接口
/// 这是一个临时解决方案，因为embedded-cli暂不支持异步且会独占写入器
pub struct SyncWriter<'a, W: Write<Error = E>, E: embedded_io::Error> {
    // 使用无锁互斥体包装的异步写入器
    pub writer: &'a Mutex<NoopRawMutex, W>, 
}

impl<'a, W: Write<Error = E>, E: embedded_io::Error> SyncWriter<'a, W, E> {
    pub fn new(writer: &'a Mutex<NoopRawMutex, W>) -> Self {
        Self { writer }
    }
}

impl<'a, W: Write<Error = E>, E: embedded_io::Error> ErrorType for SyncWriter<'a, W, E> {
    type Error = E;
}

impl<'a, W: Write<Error = E>, E: embedded_io::Error> SyncWrite for SyncWriter<'a, W, E> {
    /// 同步写入实现（内部实际是异步操作）
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // 预期互斥锁不会被阻塞，因为CLI处理在事件循环中最优先
        let mut writer = self.writer.try_lock().expect("获取写入器锁失败");
        embassy_futures::block_on(async { writer.write(buf).await })
    }

    /// 同步刷新实现
    fn flush(&mut self) -> Result<(), Self::Error> {
        let mut writer = self.writer.try_lock().expect("获取写入器锁失败");
        embassy_futures::block_on(async { writer.flush().await })
    }
}
