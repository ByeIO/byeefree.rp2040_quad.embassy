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

// 电机控制相关
use embassy_rp::peripherals::PIO0;
use embassy_rp::peripherals::PIN_14;
use embassy_rp::peripherals::PIN_15;
use embassy_rp::peripherals::PIN_16;
use embassy_rp::peripherals::PIN_17;

// 打印调试信息
use defmt::{info, panic};
use { defmt_rtt as _, panic_probe as _ };

// 内存分配相关
use embedded_alloc::LlffHeap;
#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

// 引入内部库
/// 算法模块
mod modules;
use modules::{
    calibration, errors, filters,
    shell, 
};

/// 硬件驱动
mod drivers;

/// 任务
mod tasks;
use tasks::{
    blink::{self, BlinkMode}, usb
};

/// 实用工具
mod utils;
use utils::{ types, signals, consts};

/* start 主任务 */
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("ByeIO-quad-embassy!");

    /* start 初始化 */
    let p = embassy_rp::init(Default::default());
    let _ = crate::utils::variables::init_variables();
    
    /* 初始化任务 */
    // 1. 初始化USB
    let usb_res = tasks::usb::init_usb(p.USB, &_spawner).await;
    
    // 2. 初始化电调
    let quad_pio_motors = {
            use dshot_pio::dshot_embassy_rp::DshotPio;
            use embassy_rp::{pio::*,bind_interrupts,peripherals::PIO0};
            bind_interrupts!(struct Pio0Irqs {PIO0_IRQ_0 => InterruptHandler<PIO0>;});
            DshotPio::<4,_>::new(
                p.PIO0,
                Pio0Irqs,
                p.PIN_14,
                p.PIN_15,
                p.PIN_16,
                p.PIN_17,
                // 时钟分频, 自动计算分频系数, 120MHz主频
                dshot_pio::cal_clock_div(120_000_000, dshot_pio::ShotType::DSHOT600),
            )
    };
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

    // 3. 启动电机任务
    _spawner.must_spawn(tasks::motors::motor_task(
        signals::MOTOR_SPEED.subscriber().unwrap(),
        signals::MOTOR_DIR.subscriber().unwrap(),
        quad_pio_motors,
        signals::MOTOR_STATE.publisher().unwrap(),
    ));
    
    // 4. 启动电机测试任务
    _spawner.must_spawn(tasks::motors::motor_test_task(signals::MOTOR_STATE.subscriber().unwrap()));

    /* end 启动任务 */
    
    /* end 初始化 */
    
    /* start 初始化完成 */
    // 等待稳定
    Timer::after(Duration::from_millis(2000)).await;
    // 通知blink任务切换为呼吸灯模式
    signals::BLINK_MODE.publisher().unwrap().publish_immediate(BlinkMode::Breathe);
    /* end 初始化完成 */

}
/* end 主任务 */
