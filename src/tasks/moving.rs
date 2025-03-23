#![allow(unused)]

//! 前进后退左右移动任务

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

// 宏编程
use paste::paste;

// 内部库
/// 信号
use crate::utils::signals;
use crate::utils::types::sensor::Imu10DofData;
/// 全局变量
use crate::utils::variables;
/// 滤波器
use crate::filters::baro_altitude_fusion_filter::baro_altitude_fusion_filter;
/// 类型
use crate::utils::types::flight::{ FlightModeEnum, FlightType};

/// 空中移动任务启动器
pub async fn moving_task_launcher(
    spawner: &Spawner,
){
    defmt::println!("hello from moving_task_launcher");
    
    // // 启动前进任务并监听消息
    // spawner.must_spawn(moving_forward_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动后退任务并监听消息
    // spawner.must_spawn(moving_backward_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动左移消息并监听消息
    // spawner.must_spawn(moving_left_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // // 启动右移任务并监听消息
    // spawner.must_spawn(moving_right_task(signals::FLIGHT_CHANNEL.subscriber().unwrap()));
    
    // 使用宏生成任务启动代码
    macro_rules! spawn_task {
        ($direction:ident) => {
            spawner.must_spawn(
                paste! {[<moving_ $direction _task>]}(
                    signals::FLIGHT_CHANNEL.subscriber().unwrap()
                )
            );
        };
    }

    spawn_task!(forward);
    spawn_task!(backward);
    spawn_task!(left);
    spawn_task!(right);
}

/// 空中前进任务
#[embassy_executor::task]
pub async fn moving_forward_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from moving_forward_task");
    loop{
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态(在空中)才前进
        if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Forward){
            // 获取消息中的参数: 移动持续时间(ms)
            let duration = flight_mode_wanted.time;
            
            // 获取当前位姿, TODO 获取滤波后当前位姿
            let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
            let quat_now = attitude_original.quat;
            
            // TODO 构造前进姿态的四元数
            let mut attitude_wanted = Imu10DofData::<f32>::default();
            let mut quat_wanted = attitude_wanted.quat;
            
            // 调用姿态控制器
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
            // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
            Timer::after(Duration::from_millis(duration as u64)).await;
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
        }// end if
        
    }// end loop
    
}

/// 空中后退任务
#[embassy_executor::task]
pub async fn moving_backward_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from moving_backward_task");
    
    loop{
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态(在空中)才后退
        if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Backward){
            // 获取消息中的参数: 移动持续时间(ms)
            let duration = flight_mode_wanted.time;
            
            // 获取当前位姿, TODO 获取滤波后当前位姿
            let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
            let quat_now = attitude_original.quat;
            
            // TODO 构造后退姿态的四元数
            let mut attitude_wanted = Imu10DofData::<f32>::default();
            let mut quat_wanted = attitude_wanted.quat;
            
            // 调用姿态控制器
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
            // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
            Timer::after(Duration::from_millis(duration as u64)).await;
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
        }// end if
        
    }// end loop
    
}

/// 空中左移任务
#[embassy_executor::task]
pub async fn moving_left_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from moving_left_task");
    
    loop{
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态(在空中)才左移
        if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Left){
            // 获取消息中的参数: 移动持续时间(ms)
            let duration = flight_mode_wanted.time;
            
            // 获取当前位姿, TODO 获取滤波后当前位姿
            let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
            let quat_now = attitude_original.quat;
            
            // TODO 构造左移姿态的四元数
            let mut attitude_wanted = Imu10DofData::<f32>::default();
            let mut quat_wanted = attitude_wanted.quat;
            
            // 调用姿态控制器
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
            // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
            Timer::after(Duration::from_millis(duration as u64)).await;
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
        }// end if
        
    }// end loop
    
}

/// 空中右移任务
#[embassy_executor::task]
pub async fn moving_right_task(
    // 飞行参数
    mut in_flight_args : signals::FlightSub,
){
    defmt::println!("hello from moving_right_task");
    
    loop{
        // 尝试获取消息
        let flight_mode_wanted = in_flight_args.next_message_pure().await;
        
        // 从全局变量获取当前飞行模式
        let flight_mode_now = variables::FlightModeEditor::get_data().await.mode;
        
        // 当前状态不为READY状态(在空中)才右移
        if !matches!(flight_mode_now, FlightModeEnum::Ready) && matches!(flight_mode_wanted.mode, FlightModeEnum::Right){
            // 获取消息中的参数: 移动持续时间(ms)
            let duration = flight_mode_wanted.time;
            
            // 获取当前位姿, TODO 获取滤波后当前位姿
            let mut attitude_original = variables::Imu10DofDataEditor::get_data().await;
            let quat_now = attitude_original.quat;
            
            // TODO 构造右移姿态的四元数
            let mut attitude_wanted = Imu10DofData::<f32>::default();
            let mut quat_wanted = attitude_wanted.quat;
            
            // 调用姿态控制器
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_wanted);
            
            // 移动持续时间计时到, 返回刚才的位姿, 暂停移动
            Timer::after(Duration::from_millis(duration as u64)).await;
            signals::CONTROLLER_CHANNEL.publisher().unwrap().publish_immediate(attitude_original);
        }// end if
        
    }// end loop
    
}
