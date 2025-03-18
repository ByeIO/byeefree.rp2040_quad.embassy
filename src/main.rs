#![allow(unused_variables)]
#![allow(unused_imports)]

#![no_std]
#![no_main]

// 飞控总入口

// gpio相关
use embassy_rp::gpio;
use gpio::{Level, Output};

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// 打印调试信息
use defmt::{info, panic};
use { defmt_rtt as _, panic_probe as _ };

// pio相关
use embassy_rp::peripherals::PIO0;
use dshot_pio::dshot_embassy_rp::*;

// 内存分配相关
use embedded_alloc::LlffHeap; // 需要添加依赖项
#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

// 引入内部库
mod modules;
use modules::{
    calibration, errors, filters,
    shell, 
};

mod drivers;

mod tasks;
use tasks::{
    blink::{self, BlinkMode}, usb
};

mod utils;
use utils::{ types, signals, consts};

/* start 绑定中断处理函数 */
bind_interrupts!( struct Pio0Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
});
/* end 绑定中断处理函数 */

/* start 主任务 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("ByeIO-quad-embassy!");

    /* start 初始化 */
    let p = embassy_rp::init(Default::default());
    
    /* 初始化任务 */
    // 1. 初始化USB
    let usb_res = tasks::usb::init_usb(p.USB, &_spawner).await;
    /* end 初始化任务 */
    
    /* start 启动任务 */
    // 1. 启动blink任务
    use crate::tasks::blink::blink;
    _spawner.must_spawn(blink(
        signals::BLINK_MODE.subscriber().unwrap(),
        p.PIN_25,
        p.PWM_SLICE4
    ));
    
    // 2. 启动USB任务
    _spawner.must_spawn(tasks::usb::usb_task(usb_res));

    /* end 启动任务 */
    
    /* end 初始化 */
    
    /* start 初始化完成 */
    // 通知blink任务切换为呼吸灯模式
    Timer::after(Duration::from_millis(2000)).await;
    signals::BLINK_MODE.publisher().unwrap().publish_immediate(BlinkMode::Breathe);
    // Timer::after(Duration::from_millis(3000)).await;
    // signals::BLINK_MODE.publisher().unwrap().publish_immediate(BlinkMode::OneFast);
    /* end 初始化完成 */

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

    // 并发运行所有内容。
    // 如果我们上面将所有内容都设置为`'static`，则可以使用单独的任务来执行此操作
    
    // 添加任务并执行
    // join(usb_fut, join(echo_fut, log_fut)).await;
}
/* end 主任务 */
