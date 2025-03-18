#![allow(unused)]

// 命令行解析任务

use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embassy_rp::peripherals::USB;

use crate::shell::CdcAcmIO; 

pub async fn cli_task<T: Instance>(
    class: &mut CdcAcmClass<'static, Driver<'static, T>>
) {
    let mut cli_io = CdcAcmIO { class };
    loop {
        if let Err(e) = crate::modules::shell::run_cli(&mut cli_io).await {
            // defmt::println!("CLI error: {:?}", e);
            defmt::println!("cli error!");
            // 根据错误类型决定是否继续循环
            break;
        }
    }
}
