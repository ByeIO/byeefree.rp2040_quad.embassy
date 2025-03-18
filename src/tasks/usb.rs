//! USB通信模块

// 内存分配相关
extern crate alloc;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

// 字符串相关, 引入无堆分配的字符串类型
use heapless::String;
use core::fmt::Write;

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

/* start 绑定中断函数 */
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<embassy_rp::peripherals::USB>;
});
/* end 绑定中断函数 */

// 导出公共接口
pub struct UsbResources<T: Instance> {
    pub usb: embassy_usb::UsbDevice<'static, Driver<'static, T>>,
    pub class: CdcAcmClass<'static, Driver<'static, T>>,
    pub logger_class: CdcAcmClass<'static, Driver<'static, T>>,
}

/// 初始化USB子系统
pub async fn init_usb(usb_periph: embassy_rp::peripherals::USB, spawner: &Spawner) 
-> UsbResources<embassy_rp::peripherals::USB> 
{
    // 使用defmt库在probe-rs界面打印
    defmt::println!("run init_usb");
    let driver = Driver::new(usb_periph, Irqs);
    
    let mut config = Config::new(0xc0de, 0xcafe);
    
    config.manufacturer = Some("Embassy");
    config.product = Some("byeefree_quad USB-serial");
    config.serial_number = Some("66666666");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // 使用StaticCell静态分配缓冲区
    static CONFIG_DESC: StaticCell<heapless::Vec<u8, 256>> = StaticCell::new();
    static BOS_DESC: StaticCell<heapless::Vec<u8, 256>> = StaticCell::new();
    static CONTROL_BUF: StaticCell<heapless::Vec<u8, 64>> = StaticCell::new();
    
    let config_descriptor = CONFIG_DESC.init(heapless::Vec::new());
    let bos_descriptor = BOS_DESC.init(heapless::Vec::new());
    let control_buf = CONTROL_BUF.init(heapless::Vec::new());
    
    // 预填充0用于占位
    config_descriptor.extend_from_slice(&[0u8; 256]).unwrap(); 
    bos_descriptor.extend_from_slice(&[0u8; 256]).unwrap();    
    control_buf.resize(64, 0).unwrap();

    // 使用StaticCell分配状态（替代原pool!宏）
    static STATE_CELL: StaticCell<State> = StaticCell::new();
    static LOGGER_STATE_CELL: StaticCell<State> = StaticCell::new();
    
    let state = STATE_CELL.init(State::new());
    let logger_state = LOGGER_STATE_CELL.init(State::new());

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor[..],  // 转换为切片
        &mut bos_descriptor[..],
        &mut [],
        &mut control_buf[..],
    );

    // 创建CDC类实例（参考网页69 USB总线特性）
    let class = CdcAcmClass::new(&mut builder, state, 64);
    let logger_class = CdcAcmClass::new(&mut builder, logger_state, 64);

    let usb = builder.build();

    // 返回值
    UsbResources {
        usb,
        class,
        logger_class,
    }
}

/// 启动USB任务
#[embassy_executor::task]
pub async fn usb_task(
    res : UsbResources<embassy_rp::peripherals::USB>
) {
    defmt::println!("run usb_task");
    // 解构资源获取logger_class
    let UsbResources { 
        mut class,
        logger_class,
        mut usb,
        ..
    } = res;

    // 正确调用宏（传递解构后的变量）
    let log_fut = embassy_usb_logger::with_class!(
        1024, 
        log::LevelFilter::Info, 
        logger_class
    );
    
    let usb_fut = usb.run();
    // let echo_fut = echo_task(&mut class);
    
    /* start 命令行任务 */
    use super::shell_cli::cli_task;
    let cli_fut = cli_task(&mut class);
    /* end 命令行任务 */
    
    // 加await才能初始化usb
    let _ = join(usb_fut, join(log_fut, cli_fut)).await;
    defmt::println!("loop usb_task");
}

/// 串口回显任务
async fn echo_task<T: Instance>(
    class: &mut CdcAcmClass<'static, Driver<'static, T>>
) -> Result<(), Disconnected> {
    defmt::println!("run echo_task");
    let mut buf = [0u8; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        let prefix = b"Received data: ";
        // 栈数组
        let mut echo = [0u8; 128]; 
        let pos = prefix.len();
        
        // 分步复制
        echo[..prefix.len()].copy_from_slice(prefix);
        echo[prefix.len()..prefix.len() + data.len()].copy_from_slice(data);
        
        // 最终有效数据长度
        let valid_len = prefix.len() + data.len();
        
        defmt::println!("Received raw: {:?}", HexSlice(data));
        class.write_packet(&echo[..valid_len]).await?;
    }
}

/// 错误处理类型
pub struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

/// 自定义格式化包装器
#[derive(Debug)]
struct HexSlice<'a>(&'a [u8]);

impl defmt::Format for HexSlice<'_> {
    fn format(&self, fmt: defmt::Formatter) {
        for &b in self.0 {
            defmt::write!(fmt, "{:02x}", b);
        }
    }
}
