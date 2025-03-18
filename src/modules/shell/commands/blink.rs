// 命令解析相关
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Ticker};
use embedded_cli::Command;
use embedded_io::ErrorKind;
use embedded_io_async::{Read, Write};
use ufmt::uwrite;
use crate::shell::{UBuffer, INTERRUPT};
use heapless::String;
use core::str::FromStr;

// 内部库
use crate::tasks::blink::BlinkMode;
use crate::utils::signals;

// 命令定义部分
#[derive(Command, Clone)]
pub enum BlinkCommand {
    // 关闭led灯
    /// led off
    None,
    /// fast#1
    One_Fast,
    /// fast#2
    Two_Fast,
    /// fast#3
    Three_Fast,
    /// blink fast
    On_Off_Fast,
    /// blink slow
    On_Off_Slow,
    /// breathe...
    Breathe,
}

// 实现CommandHandler特征
impl super::CommandHandler for BlinkCommand {
    async fn handler(
        &self,
        mut serial: impl Read<Error = ErrorKind> + Write<Error = ErrorKind>,
    ) -> Result<(), ErrorKind> {
        // 匹配所有blink子命令变体
        let (mode_str, blink_mode) = match self {
            BlinkCommand::None => ("none", BlinkMode::None),
            BlinkCommand::One_Fast => ("one_fast", BlinkMode::OneFast),
            BlinkCommand::Two_Fast => ("two_fast", BlinkMode::TwoFast),
            BlinkCommand::Three_Fast => ("three_fast", BlinkMode::ThreeFast),
            BlinkCommand::On_Off_Fast => ("on_off_fast", BlinkMode::OnOffFast),
            BlinkCommand::On_Off_Slow => ("on_off_slow", BlinkMode::OnOffSlow),
            BlinkCommand::Breathe => ("breathe", BlinkMode::Breathe),
        };

        // 通过signal发布新模式
        signals::BLINK_MODE.publisher().unwrap().publish_immediate(blink_mode);
        
        // 写入确认信息到串口
        let mut buf = UBuffer::<32>::new();
        uwrite!(&mut buf, "Blink mode set to: {}\n\r", mode_str)?;
        serial.write_all(&buf.inner).await?;

        Ok(())
    }
}
