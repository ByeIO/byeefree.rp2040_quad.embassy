#![no_std]    // 禁用标准库
#![no_main]   // 不使用标准main函数入口

use defmt::info;  // 嵌入式友好日志
use defmt_rtt as _; // 实时传输日志后端
// 启动函数的宏
use rp_pico::entry;  // RP2040入口宏

// 确保程序panic时停机（如果没有显式链接这个crate则不会生效）
// use panic_halt as _;
use panic_probe as _ ;

// 引入重要trait
use rp_pico::hal::prelude::*;

// 外设访问包（PAC）的别名，提供底层寄存器访问
use rp_pico::hal::pac;

// 硬件抽象层（HAL）的别名，提供高级驱动
use rp_pico::hal;

// 引入PIO相关crate
use hal::pio::{PIOBuilder, Running, StateMachine, Tx, ValidStateMachine, SM0};
use pio::{Instruction, InstructionOperands, OutDestination};
use pio_proc::pio_file;

/// 设置PIO PWM周期
///
/// 使用巧妙技巧在占空比之外设置第二个值。首先将值写入TX FIFO，
/// 不同于普通指令，这里会停止状态机并注入自定义指令将写入值移动到ISR
fn pio_pwm_set_period<T: ValidStateMachine>(
    sm: StateMachine<(hal::pac::PIO0, SM0), Running>, // 运行中的状态机
    tx: &mut Tx<T>,    // 传输通道
    period: u32,       // 周期值
) -> StateMachine<(hal::pac::PIO0, SM0), Running> {
    // 确保队列清空（通常应该已为空）
    while !tx.is_empty() {}

    let mut sm = sm.stop();    // 停止状态机
    tx.write(period);           // 写入周期值
    
    // 注入PULL指令
    sm.exec_instruction(Instruction {
        operands: InstructionOperands::PULL {
            if_empty: false,
            block: false,
        },
        delay: 0,
        side_set: None,
    });
    
    // 注入OUT指令到ISR
    sm.exec_instruction(Instruction {
        operands: InstructionOperands::OUT {
            destination: OutDestination::ISR,
            bit_count: 32,
        },
        delay: 0,
        side_set: None,
    });
    
    sm.start()  // 重启状态机
}

/// 设置PIO PWM占空比
///
/// 写入TX FIFO的值直接由普通PIO程序使用
fn pio_pwm_set_level<T: ValidStateMachine>(tx: &mut Tx<T>, level: u32) {
    tx.write(level);  // 将占空比写入TX FIFO
}

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

    // 分割PIO0外设
    let (mut pio0, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    // 加载PIO程序
    let program = pio_file!("./examples/pio_pwm_led.pio", select_program("pwm"),);
    let installed = pio0.install(&program.program).unwrap();

    // 配置GPIO25为PIO功能
    let _led: hal::gpio::Pin<_, hal::gpio::FunctionPio0, hal::gpio::PullNone> =
        pins.led.reconfigure();
    let led_pin_id = 25;  // 板载LED引脚号

    // 构建PIO程序并配置引脚（主设置和边带设置）
    let (mut sm, _, mut tx) = PIOBuilder::from_installed_program(installed)
        .set_pins(led_pin_id, 1)          // 设置主引脚
        .side_set_pin_base(led_pin_id)    // 设置边带引脚
        .build(sm0);

    // 配置GPIO25为输出方向
    sm.set_pindirs([(led_pin_id, hal::pio::PinDir::Output)]);

    // 启动状态机
    let sm = sm.start();

    // 设置PWM周期（最大值-1）
    pio_pwm_set_period(sm, &mut tx, u16::MAX as u32 - 1);

    // 无限循环调整占空比实现呼吸灯
    let mut level = 0;
    loop {
        info!("亮度等级 = {}", level);  // 记录当前亮度
        pio_pwm_set_level(&mut tx, level * level);  // 平方增长更平滑
        level = (level + 1) % 256;       // 循环0-255
        delay.delay_ms(10);              // 10ms延迟
    }
}

// 文件结束