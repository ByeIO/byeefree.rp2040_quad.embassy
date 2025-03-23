#![allow(unused)]

//! 传感器任务

// 五个串口:
// 1. ATK
//   - ATK_RX : GP4
//   - ATK_TX : GP5
// 2. TF
//   - TF_TX : GP10
//   - TF_RX : GP11
// 3. MICOAIR
//   - MICO_TX : GP12
//   - MICO_RX : GP13
// 4. SUB
//   - SUB_RX : GP26
//   - SUB_TX : GP27
// 5. MICOAIR2
//   - MICO_RX2 : GP2

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch as SyncWatch;
use core::sync::atomic::{AtomicBool, AtomicU16, AtomicUsize};
use embassy_sync::{channel::Channel, once_lock::OnceLock, pubsub::{PubSubBehavior, PubSubChannel}};

// pio相关
use embassy_rp::{pac::Interrupt::PIO1_IRQ_0, peripherals::PIO1, Peripherals};
use embassy_rp::{
    pio::{ Instance, Pio, Config, PioPin, ShiftConfig, ShiftDirection::Left, InterruptHandler},
    Peripheral, interrupt::typelevel::Binding
};

// 线性代数相关
use nalgebra::UnitQuaternion;

// 内部库
use crate::types::sensor::Imu10DofData;

// 传感器任务启动器
pub async fn sensor_task_launcher(
    // 任务调度器
    spawner: &Spawner,
    // 消息通道
    // mut in_blink_mode : signals::BlinkModeSub,
    
    // PIO通道
    mut pio : PIO1,
    // mut pio_irq : impl Binding<Instance::Interrupt, InterruptHandler<Instance>>,
    
    // 串口
    mut uart0 : embassy_rp::uart::Uart<'static, embassy_rp::peripherals::UART0, embassy_rp::uart::Async, >,
    mut uart1 : embassy_rp::uart::Uart<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async, >,
    
    // IMU传感器串口
    mut imu_pin_rx : impl PioPin,
    mut imu_pin_tx : impl PioPin,
    
    // 光流传感器串口
    mut opti_pin_rx : impl PioPin,
    mut opti_pin_tx : impl PioPin,
    
    // 距离传感器串口
    mut tof_pin_rx : impl PioPin,
    mut tof_pin_tx : impl PioPin,
    
    // 预留接口
    mut sub_pin_rx : impl PioPin,
    mut sub_pin_tx : impl PioPin,
    
    // 飞控塔接口
    mut mico_rx2_pin : impl PioPin,
    
    
) {
    // 打印调试信息
    defmt::println!("hello from tasks::sensor_task_launcher");
    
    // 1. imu任务
    spawner.must_spawn(sensor_imu_task(
        uart1,
        // 发布imu数据
        crate::signals::IMU_READING.publisher().unwrap(),
    ));
    
    // 2. 光流传感器任务
    spawner.must_spawn(sensor_opti_task());
    
    // 3. 距离传感器任务
    spawner.must_spawn(sensor_tof_task());
    
}

// 1. imu任务
#[embassy_executor::task]
pub async fn sensor_imu_task(
    mut uart1: embassy_rp::uart::Uart<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async, >,
    out_imu_reading: crate::signals::ImuReadingPub,
){
    use crate::drivers::imu::atk_imu901::defines::*;
    use crate::drivers::imu::atk_imu901::functions::atk_ms901m::AtkMs901m;
    
    // 打印调试信息
    defmt::println!("hello from tasks::sensors::sensor_imu_task");

    // 初始化传感器
    let mut sensor = AtkMs901m::new(uart1).await;
    
    // 传感器数据结构初始化
    let mut gyro_data = AtkMs901mGyroData::default();
    let mut accel_data = AtkMs901mAccelerometerData::default();
    let mut mag_data = AtkMs901mMagnetometerData::default();
    let mut baro_data = AtkMs901mBarometerData::default();
    let mut quat_data = AtkMs901mQuaternionData::default();
    
    loop {
        let mut frames : [AtkMs901mFrame; 32] = [AtkMs901mFrame::default(); 32];
        // 获取所有有效帧
        let frame_num = sensor.get_all_valid_frames(&mut frames).await;
        if frame_num > 0{
            // 调用封装函数处理帧数据并组合为IMU数据
            let imu_data = sensor.process_frames_and_combine_data(
                &frames,
                frame_num,
                &mut gyro_data,
                &mut accel_data,
                &mut mag_data,
                &mut baro_data,
                &mut quat_data,
            ).await;
            
            out_imu_reading.publish_immediate(imu_data);
            
            // FIXME: 暂时使用全局变量交换数据
            use crate::utils::variables::Imu10DofDataEditor;
            Imu10DofDataEditor::write_data(imu_data).await;
            
            // DEBUG专用
            // defmt::println!("sensor_imu_task publish data ok");
            // defmt::error!("imu_data: {}", imu_data);
            
        }// end if
        
        // 维持2Hz发布速率
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
   
}

// 2. 光流传感器任务
#[embassy_executor::task]
pub async fn sensor_opti_task(){
    // 打印调试信息
    defmt::println!("hello from tasks::sensors::sensor_opti_task");
}

// 3. 距离传感器任务
#[embassy_executor::task]
pub async fn sensor_tof_task(){
    // 打印调试信息
    defmt::println!("hello from tasks::sensors::sensor_tof_task");
}
