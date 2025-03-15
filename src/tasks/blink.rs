#![allow(unused)]

// led指示灯任务

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

// pio相关(led:gpio25)
use embassy_rp::peripherals::PIO0;

// 内部库:信号
use crate::signals;

// 闪灯模式
#[derive(Clone,Copy)]
pub enum BlinkMode {
    // 不闪灯
    None,
    // 快速1
    OneFast,
    // 快速2
    TwoFast,
    // 快速3
    ThreeFast,
    // 快速闪
    OnOffFast,
    // 慢速闪
    OnOffSlow,
    // 呼吸灯
    Breathe,
}

// embassy任务
#[embassy_executor::task]
pub async fn blink(
    // 订阅blink模式
    mut in_blink_mode : signals::BlinkModeSub,
    pin : AnyPin
) {

    let mut blink_mode = BlinkerMode::None;

    let mut led = Output::new(pin,Level::Low);
    loop { 

        // 尝试获取新的blink模式
        if let Some(b) = in_blink_mode.try_next_message_pure() {blink_mode = b}

        // 匹配不同模式
        match blink_mode {
            BlinkMode::None => {
                led.set_low();
                blink_mode = in_blink_mode.next_message_pure().await;
            },
            BlinkMode::OneFast => one_fast(&mut led).await,
            BlinkMode::TwoFast => two_fast(&mut led).await,
            BlinkMode::ThreeFast => three_fast(&mut led).await,
            BlinkMode::OnOffFast => on_off_fast(&mut led).await,
            BlinkMode::OnOffSlow => on_off_slow(&mut led).await,
            BlinkMode::Breathe => breathe(&mut led).await,
        };
    }
}

// 1. 快速1
#[allow(unused)]
async fn one_fast<'a>(led: &mut Output<'a,AnyPin>) {

    // 灭一会
    led.set_high(); 
    Timer::after(Duration::from_millis(50)).await;

    // 长亮
    led.set_low();
    Timer::after(Duration::from_millis(950)).await;
}

// 2. 快速2
#[allow(unused)]
async fn two_fast<'a>(led: &mut Output<'a,AnyPin>) {
    // 短灭
    led.set_high();
    Timer::after(Duration::from_millis(50)).await;

    // 中亮
    led.set_low();
    Timer::after(Duration::from_millis(100)).await;

    // 短灭
    led.set_high(); 
    Timer::after(Duration::from_millis(50)).await;

    // 长亮
    led.set_low();
    Timer::after(Duration::from_millis(800)).await;
}

// 3. 快速3
#[allow(unused)]
async fn three_fast<'a>(led: &mut Output<'a,AnyPin>) {
    // 短灭
    led.set_high();
    Timer::after(Duration::from_millis(50)).await;

    // 中亮
    led.set_low();
    Timer::after(Duration::from_millis(100)).await;

    // 短灭
    led.set_high(); 
    Timer::after(Duration::from_millis(50)).await;

    // 中亮
    led.set_low();
    Timer::after(Duration::from_millis(100)).await;

    // 短灭
    led.set_high();
    Timer::after(Duration::from_millis(50)).await;

    // 长亮
    led.set_low();
    Timer::after(Duration::from_millis(650)).await;
}

// 4. 快速亮灭
#[allow(unused)]
async fn on_off_fast<'a>(led: &mut Output<'a,AnyPin>) {
    led.set_high(); // Short high
    Timer::after(Duration::from_millis(100)).await;

    led.set_low(); // Medium low
    Timer::after(Duration::from_millis(100)).await;
}

// 5. 慢速亮灭
#[allow(unused)]
async fn on_off_slow<'a>(led: &mut Output<'a,AnyPin>) {
    led.set_high(); // Short high
    Timer::after(Duration::from_millis(100)).await;

    led.set_low(); // Medium low
    Timer::after(Duration::from_millis(100)).await;
}

// 6. 呼吸灯
#[allow(unused)]
async fn breathe<'a>(led: &mut Output<'a,AnyPin>) {

}