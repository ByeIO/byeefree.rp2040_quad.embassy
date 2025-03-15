#![allow(unused_variables)]

#![no_std]
#![no_main]

// gpio相关
use embassy_rp::gpio;
use gpio::{Level, Output};

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// usb相关
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::{Builder, Config};

// 打印调试信息
use defmt::{info, panic};
use { defmt_rtt as _, panic_probe as _ };

// pio相关
use embassy_rp::peripherals::PIO0;
use dshot_pio::dshot_embassy_rp::*;

/* start 绑定中断处理函数 */
bind_interrupts!( struct Pio0Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
});

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});
/* end 绑定中断处理函数 */

/* start 主任务 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("ByeIO-quad-embassy!");

    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    // 从HAL中创建usb驱动程序
    let driver = Driver::new(p.USB, Irqs);

    // 创建embassy-usb配置
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // 使用驱动程序和配置创建embassy-usb DeviceBuilder。
    // 它需要一些缓冲区来构建描述符。
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();
    let mut logger_state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // 没有msos描述符
        &mut control_buf,
    );

    // 在构建器上创建类。
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // 为日志记录器创建一个类
    let logger_class = CdcAcmClass::new(&mut builder, &mut logger_state, 64);

    // 创建日志记录器并返回日志记录器的未来
    // 注意：之后你需要使用log::info!而不是info!（这也适用于所有其他log::*宏）
    let log_fut = embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, logger_class);

    // 构建构建器。
    let mut usb = builder.build();

    // 运行USB设备。
    let usb_fut = usb.run();

    // 发送dshot信号
    let dshot_embassy = DshotPio::<4,_>::new(
        p.PIO0,
        Pio0Irqs,
        p.PIN_14,
        p.PIN_15,
        p.PIN_16,
        p.PIN_17,
        // 时钟分频
        (52, 0)
    );

    // 对类进行操作！
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            log::info!("Connected");
            let _ = echo(&mut class).await;
            log::info!("Disconnected");
        }
    };

    let led_task = async {
        loop {
            log::info!("led on!");
            led.set_high();
            Timer::after(Duration::from_secs(1)).await;

            log::info!("led off!");
            led.set_low();
            Timer::after(Duration::from_secs(1)).await;
        }
    };

    // 并发运行所有内容。
    // 如果我们上面将所有内容都设置为`'static`，则可以使用单独的任务来执行此操作
    
    // 添加任务并执行
    join(usb_fut, join(echo_fut, join(log_fut,led_task))).await;
}
/* end 主任务 */

/* start 断开usb连接处理 */
struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}
/* end 断开usb连接处理 */

/// 串口回显
async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}