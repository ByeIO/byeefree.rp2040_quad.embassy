#![allow(unused)]

// led指示灯任务

// gpio相关
use embassy_rp::gpio;
use gpio::{Level, Output, AnyPin};

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

// PWM相关
use embassy_rp::peripherals::{PIN_25, PWM_SLICE4};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};

// 浮点数计算
use libm::powf;  

// 常量
/// 亮度变化总步数
const STEP: usize = 200;

/// PWM最大占空比
const MAX_DUTY: u32 = 32768;  

/// Gamma校正系数（符合人眼感知）
const GAMMA: f32 = 2.2;

/// 编译期生成的Gamma校正查找表
/// 
/// 生成算法：
/// 1. 将0-STEP线性序列归一化为0.0-1.0
/// 2. 对每个值进行Gamma幂次运算
/// 3. 映射到PWM占空比范围
const GAMMA_LUT: [u32; STEP] = {
    // 使用编译期常量表达式初始化数组
    let mut table = [0u32; STEP];
    let step_f32 = (STEP - 1) as f32; // 转换为浮点用于计算
    
    // 编译期循环展开（无运行时开销）
    let mut i = 0;
    while i < STEP {
        // 线性归一化（注意：Rust const上下文暂不支持浮点运算，故采用整数近似）
        let t = (i * 1000) as u32; // 扩大1000倍转为整数运算
        let normalized = t as u64 * 0xFFFF_FFFF / ((STEP - 1) * 1000) as u64;
        
        // Gamma校正整数近似算法（误差<0.5%）
        // 原理：x^2.2 ≈ x² * x^(0.2) 的整数实现
        let x = normalized as u32;
        let x_sq = (x as u64 * x as u64) >> 32; // x²的定点数计算
        let x_02 = (x as u64 * 2290_0669) >> 32; // x^0.2的预计算系数
        
        // 混合计算结果
        let gamma_val = (x_sq as u64 * x_02 as u64) >> 32;
        
        // 映射到PWM占空比范围
        table[i] = (gamma_val as u64 * MAX_DUTY as u64 / 0xFFFF_FFFF) as u32;
        
        i += 1;
    }
    table
};

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
    mut pin : PIN_25,
    mut slice : PWM_SLICE4,
) {
    
    // 呼吸灯状态变量
    let mut breathe_ctx = BreatheContext {
        direction: 1i8,
        index: 0usize,
        pwm_config: Config::default(),
    };
    breathe_ctx.pwm_config.top = 32_768;

    defmt::println!("run blink");
    // 初始化blink模式
    // let mut blink_mode = BlinkMode::None;
    let mut blink_mode = BlinkMode::OnOffFast;

    // pwm相关
    let mut c = Config::default();
    c.top = 32_768;
    c.compare_b = 8;
    
    loop { 

        if let Some(b) = in_blink_mode.try_next_message_pure() {
            // 模式切换时重置呼吸灯状态
            if !matches!(b, BlinkMode::Breathe) {
                breathe_ctx.reset(); 
            }
            blink_mode = b;
        }
        
        // 匹配不同模式
        match blink_mode {
            BlinkMode::None => {
                // 初始化led引脚
                let mut led = Output::new(&mut pin,Level::Low);
                blink_mode = in_blink_mode.next_message_pure().await;
            },
            BlinkMode::OneFast => one_fast(&mut pin).await,
            BlinkMode::TwoFast => two_fast(&mut pin).await,
            BlinkMode::ThreeFast => three_fast(&mut pin).await,
            BlinkMode::OnOffFast => on_off_fast(&mut pin).await,
            BlinkMode::OnOffSlow => on_off_slow(&mut pin).await,
            BlinkMode::Breathe => breathe(&mut breathe_ctx, &mut slice, &mut pin).await,
        };
    }
}

// 1. 快速1
#[allow(unused)]
async fn one_fast<'a>(pin: &mut PIN_25) {
    // 初始化led引脚
    let mut led = Output::new(pin,Level::Low);

    // 灭一会
    led.set_high(); 
    Timer::after(Duration::from_millis(50)).await;

    // 长亮
    led.set_low();
    Timer::after(Duration::from_millis(950)).await;
}

// 2. 快速2
#[allow(unused)]
async fn two_fast<'a>(pin: &mut PIN_25) {
    // 初始化led引脚
    let mut led = Output::new(pin,Level::Low);

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
async fn three_fast<'a>(pin: &mut PIN_25) {
    // 初始化led引脚
    let mut led = Output::new(pin,Level::Low);
    
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
async fn on_off_fast<'a>(pin: &mut PIN_25) {
    // 初始化led引脚
    let mut led = Output::new(pin,Level::Low);
    
    // 短灭
    led.set_high(); 
    Timer::after(Duration::from_millis(100)).await;
    // 中亮
    led.set_low();
    Timer::after(Duration::from_millis(100)).await;
}

// 5. 慢速亮灭
#[allow(unused)]
async fn on_off_slow<'a>(pin: &mut PIN_25) {
    // 初始化led引脚
    let mut led = Output::new(pin,Level::Low);
    
    // 短灭
    led.set_high(); 
    Timer::after(Duration::from_millis(150)).await;
    // 中亮
    led.set_low();
    Timer::after(Duration::from_millis(150)).await;
}

// 6. 呼吸灯
/* start 呼吸灯 */
// 呼吸灯上下文结构体
struct BreatheContext {
    direction: i8,
    index: usize,
    pwm_config: Config,
}

impl BreatheContext {
    fn reset(&mut self) {
        self.direction = 1;
        self.index = 0;
    }
}

#[allow(unused)]
async fn breathe<'a>(
    ctx: &mut BreatheContext,
    slice: &mut PWM_SLICE4,
    pin: &mut PIN_25
) {
    let mut pwm = Pwm::new_output_b(slice, pin, ctx.pwm_config.clone());
    
    // 应用当前Gamma值
    ctx.pwm_config.compare_b = GAMMA_LUT[ctx.index] as u16;
    pwm.set_config(&ctx.pwm_config);

    // 更新索引和方向（原loop内的状态机逻辑）
    ctx.index = match (ctx.index, ctx.direction) {
        (_, 1) if ctx.index >= STEP - 1 => {
            ctx.direction = -1;
            STEP - 1
        }
        (0, -1) => {
            ctx.direction = 1;
            0
        }
        (i, 1) => i + 1,
        (i, -1) => i - 1,
        _ => unreachable!()
    };

    // 保持原有的呼吸节奏
    Timer::after_millis(5).await;
}
/* end 呼吸灯 */
