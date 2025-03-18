//! 从usb读取数据的适配器

extern crate alloc;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

// 字符串相关, 引入无堆分配的字符串类型
use heapless::String;
use core::future::Future;
use embedded_io::{ErrorKind, ErrorType};
use embedded_io_async::{Read, Write};

// hal相关
use embassy_rp::Peripheral;

// usb相关
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};
use embassy_rp::peripherals::USB;

// 执行器相关
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// 打印调试信息
use defmt::{info, panic};
use { defmt_rtt as _, panic_probe as _ };

// 使用中断
use crate::shell::{UBuffer, INTERRUPT};

/// 适配器将CdcAcmClass转换为嵌入式IO的AsyncRead/Write
pub struct CdcAcmIO<'a, T: Instance> {
    pub class: &'a mut CdcAcmClass<'static, Driver<'static, T>>,
}

// 为结构体实现ErrorType trait
impl<T: Instance> ErrorType for CdcAcmIO<'_, T> {
    type Error = ErrorKind; // 明确指定错误类型为ErrorKind
}

impl<T: Instance> embedded_io_async::Read for CdcAcmIO<'_, T> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.class.read_packet(buf).await.map_err(|e| {
            match e {
                EndpointError::BufferOverflow => ErrorKind::Other,
                EndpointError::Disabled => ErrorKind::NotConnected,
            }
        })
    }
}

impl<T: Instance> embedded_io_async::Write for CdcAcmIO<'_, T> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.class.write_packet(buf).await.map_err(|e| {
            match e {
                EndpointError::BufferOverflow => ErrorKind::Other,
                EndpointError::Disabled => ErrorKind::NotConnected,
            }
        })?;
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
