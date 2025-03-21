//! # Pico PWM呼吸灯示例
//!
//! 使用PWM外设在Pico开发板上实现LED呼吸灯效果
//!
//! 该程序将渐亮/渐灭连接到GP25引脚的LED，这是Pico开发板上的板载LED
//!
//! 查看`Cargo.toml`文件获取版权和许可证信息

#![no_std]    // 禁用标准库
#![no_main]   // 不使用标准main函数入口

// 启动函数的宏
use rp_pico::entry;  // RP2040入口宏

// GPIO特性
use embedded_hal::pwm::SetDutyCycle;  // PWM占空比设置特性

// 确保程序panic时停机（如果没有显式链接这个crate则不会生效）
use panic_halt as _;

// 引入重要trait
use rp_pico::hal::prelude::*;

// 外设访问包（PAC）的别名，提供底层寄存器访问
use rp_pico::hal::pac;

// 硬件抽象层（HAL）的别名，提供高级驱动
use rp_pico::hal;

// 最小PWM值（对应LED最暗状态）
const LOW: u16 = 0;

// 最大PWM值（对应LED最亮状态）
const HIGH: u16 = 25000;

/// 裸机应用程序入口点
///
/// `#[entry]`宏确保Cortex-M启动代码在所有全局变量初始化后立即调用本函数
///
/// 函数配置RP2040外设，然后无限循环实现LED呼吸灯效果
#[entry]
fn main() -> ! {
    // 获取单例外设对象
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // 初始化看门狗（时钟配置需要）
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // 配置时钟（默认生成125MHz系统时钟）
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // SIO模块控制GPIO引脚
    let sio = hal::Sio::new(pac.SIO);

    // 根据开发板功能配置引脚
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // 延时对象（毫秒级）
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // 初始化PWM模块
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // 配置PWM4
    let pwm = &mut pwm_slices.pwm4;
    pwm.set_ph_correct();  // 启用相位校正模式
    pwm.enable();          // 使能PWM

    // 将PWM4的B通道输出到LED引脚
    let channel = &mut pwm.channel_b;
    channel.output_to(pins.led);  // 绑定通道到LED引脚

    // 无限循环实现呼吸灯效果
    loop {
        // 亮度渐增（跳过前100个值加速效果）
        for i in (LOW..=HIGH).skip(100) {
            delay.delay_us(8);       // 8微秒延时
            let _ = channel.set_duty_cycle(i);  // 设置占空比
        }

        // 亮度渐减（反向遍历并跳过前100个值）
        for i in (LOW..=HIGH).rev().skip(100) {
            delay.delay_us(8);
            let _ = channel.set_duty_cycle(i);
        }

        delay.delay_ms(500);  // 完成一次呼吸周期后暂停500ms
    }
}

// 文件结束