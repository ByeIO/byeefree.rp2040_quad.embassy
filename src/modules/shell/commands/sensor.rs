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

#[derive(Command, Clone)]
pub enum SensorCommand {
    // 打印hello+名称
    /// Say hello to World or someone else
    Hello {
        /// To whom to say hello (World by default)
        name: Option<UBuffer<32>>,
    },
}

// 实现CommandHandler特征
impl super::CommandHandler for SensorCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        match self {
            SensorCommand::Hello { name } => {
                // 使用UBuffer结构体
                let mut buf = UBuffer::<32>::new();
                uwrite!(
                    &mut buf,
                    "Hello {}...\n",
                    name.as_ref().unwrap_or(&UBuffer::default())
                )?;
                // 写入到输出接口
                serial.write_all(&buf.inner).await?;
            }
        }
        Ok(())
    }
}
