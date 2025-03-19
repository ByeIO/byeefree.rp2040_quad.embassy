#![allow(unused)]

//! 电机控制任务
// rp2040-pico默认时钟频率: 120MHz
// 小蜜蜂电调,BLHeli-S固件, DShot600控制

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// 电调驱动
use crate::{drivers::esc::little_bee, utils::signals::MOTOR_STATE};

// dshot相关
use dshot_pio::{ cal_clock_div, dshot_embassy_rp::DshotPio, ShotType, DshotPioTrait };

// 静态变量互斥访问相关
use embassy_sync::blocking_mutex::{Mutex, CriticalSectionMutex, raw::CriticalSectionRawMutex};
// use core::sync::atomic::{AtomicU16, Ordering};
use static_cell::StaticCell;
use core::{
    //内部可变性类型RefCell
    cell::RefCell, 
    fmt::Write, 
    //引用借用
    borrow::BorrowMut,
    borrow::Borrow,
};

// 内部库
/// 信号
use crate::utils::signals;
/// 全局变量
use crate::utils::variables;

// 电机序号
#[derive(Clone,Copy,PartialEq)]
pub enum Motors {
    One,
    Two,
    Three,
    Four
}

// 电机状态
#[derive(Clone,Copy,PartialEq)]
pub enum MotorState {
    READY,
    DISARMED,
    KEEP,
    RUNNING,
    MANUAL,
}

// 1. 解锁电调工具函数
pub async fn arming_esc(
    motor: Motors, 
    out_pio_motors : &mut DshotPio<'static, 4, embassy_rp::peripherals::PIO0>
    ){
    defmt::println!("hello from arming_esc");
    for _i in 0..50 {
        out_pio_motors.throttle_minimum();
        Timer::after(Duration::from_millis(50)).await;
    }
}

// 2. 电机转速控制任务(async)
// motor序号(e.g. Motors::One), 0~100转速(u16)
#[embassy_executor::task]
pub async fn motor_task(
    mut in_motor_speed : signals::MotorSpeedSub,
    mut in_motor_direction : signals::MotorDirSub,
    mut out_pio_motors : DshotPio<'static, 4, embassy_rp::peripherals::PIO0>,
    out_motor_state : signals::MotorStatePub,
){
    defmt::println!("hello from motor_task");
    
    // 初始化电机
    init_motors(&mut out_pio_motors).await;
    out_motor_state.publish_immediate(MotorState::READY);
    
    defmt::println!("done: motor init");
    
    loop{
        // 1. 根据转速信号调整电机
        /// 获取电机转速
        let mut motor_speed = in_motor_speed.next_message_pure().await;
        let (m1, m2, m3, m4) = motor_speed;
        
        /// 更新全局变量
        crate::utils::variables::MotorSpeedEditor::write_speeds(
            [m1.unwrap_or(0),m2.unwrap_or(0),m3.unwrap_or(0),m4.unwrap_or(0)]
        ).await;
        
        /// 发送到电调
        out_pio_motors.throttle_clamp([
            m1.unwrap_or(0),
            m2.unwrap_or(0),
            m3.unwrap_or(0),
            m4.unwrap_or(0)
        ]);
        
        // 2. 根据方向给电机换向
        /// 获取电机旋转方向设置
        let mut motor_dir = in_motor_direction.next_message_pure().await;
        let (m1_, m2_, m3_, m4_) = motor_dir;
        /// 更新全局变量
        crate::utils::variables::MotorDirectionEditor::write_directions(
            [m1_.unwrap_or(true),m2_.unwrap_or(true),m3_.unwrap_or(true),m4_.unwrap_or(true)]
        ).await;
        
        /// 发送到电调
        out_pio_motors.reverse([
            m1_.unwrap_or(true),
            m2_.unwrap_or(true),
            m3_.unwrap_or(true),
            m4_.unwrap_or(true)
        ]);
        
    }
}

// 3. 电调初始化
// p_pio, (p_motor1_gpio, p_motor2_gpio, p_motor3_gpio, p_motor4_gpio)
pub async fn init_motors(out_pio_motors :&mut DshotPio<'static, 4, embassy_rp::peripherals::PIO0>)
{
    defmt::println!("hello from init_motors");
    // 3.1 绑定gpio和dshot信号
    let _ = little_bee::init().await;
        
    // 3.2 发送小油门信号解锁电调, 让电机可以旋转
    arming_esc(Motors::One, out_pio_motors).await;
    arming_esc(Motors::Two, out_pio_motors).await;
    arming_esc(Motors::Three, out_pio_motors).await;
    arming_esc(Motors::Four, out_pio_motors).await;
}

// 4. 测试分别旋转四个电机, 然后让电机保持缓慢旋转
#[embassy_executor::task]
pub async fn motor_test_task(
    mut in_motor_state : signals::MotorStateSub,
){
    // 等待电机初始化完成
    let state = in_motor_state.next_message_pure().await;
    if state == MotorState::READY {
        defmt::println!("hello from motor_test");
        
        // 1. 依次旋转四个电机
        signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((Some(30), Some(0), Some(0), Some(0)));
        Timer::after(Duration::from_millis(3000)).await;
        signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((Some(0), Some(30), Some(0), Some(0)));
        Timer::after(Duration::from_millis(3000)).await;
        signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((Some(0), Some(0), Some(30), Some(0)));
        Timer::after(Duration::from_millis(3000)).await;
        signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((Some(0), Some(0), Some(0), Some(30)));
        Timer::after(Duration::from_millis(3000)).await;
        
        defmt::println!("done: motor_test");
        
        // 2. 保持慢速旋转, 需要一直发布信号
        loop{
            if state == MotorState::READY {
                signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((Some(10), Some(10), Some(10), Some(10)));
                // 留出时间给其他任务执行, 不能一直占用cpu
                Timer::after(Duration::from_millis(500)).await;
            }else{
                defmt::println!("done: motor_ready");
                break;
            }
        }
        
    }
}
