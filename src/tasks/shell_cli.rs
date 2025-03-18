#![allow(unused)]

// 命令行解析任务

use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embassy_rp::peripherals::USB;

use static_cell::StaticCell;

use crate::shell::CdcAcmIO; 

pub async fn cli_task<T: Instance>(
    class: &mut CdcAcmClass<'static, Driver<'static, T>>,
) {
    let mut cli_io = CdcAcmIO { class };
    
    loop {
        if let Err(e) = crate::modules::shell::run_cli(
            &mut cli_io, 
        ).await {
            defmt::println!("cli error!\nResetting cli...");
            embassy_time::Timer::after_millis(100).await;
            // FIXME: 无法重新初始化
            loop{}
        }
    }
}
