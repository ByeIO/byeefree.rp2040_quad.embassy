#![allow(dead_code)]

/*
此固件上常用的通信信道类型是Embassy PubSubChannel。
该模块定义了连接所有embassy任务的通道类型，并作为唯一的
传达任务之间的值和状态的方法。当PubSubChannel类型的
发布者和容量都为1时，其为点对点通信。
*/

// 同步消息传递相关
use embassy_sync::{
    pubsub::{PubSubChannel,Publisher,Subscriber},
    blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex}
};

// 从信道中获取存在的数据并向变量赋值, 注意信道容量
pub fn try_assign_from_channel<const CAP : usize, const SUBS : usize, const PUBS : usize, M:RawMutex, T:Clone>
(channel : &mut Subscriber<M,T,CAP,SUBS,PUBS>, variable : &mut T) {
    if let Some(new_value) = channel.try_next_message_pure() { *variable = new_value }
}

// 信道互斥锁
type ChannelMutex = CriticalSectionRawMutex;

// PubSubChannel的简短称呼
type Pub<T,const N: usize> = Publisher<'static,ChannelMutex,T,2,N,2>;
type Sub<T,const N: usize> = Subscriber<'static,ChannelMutex,T,2,N,2>;
type Ch<T,const N: usize> = PubSubChannel<ChannelMutex,T,2,N,2>;

/* 
固定模板: 
发送者->信道->订阅者
const CH_PROTOTYPE_X_NUM: usize = 1;
pub type ChPrototypeXType = bool;
pub type ChPrototypeXPub = Pub<ChPrototypeXType,1,CH_PROTOTYPE_X_NUM,1>;
pub type ChPrototypeXSub = Sub<ChPrototypeXType,1,CH_PROTOTYPE_X_NUM,1>;
pub static CH_PROTOTYPE_X : Ch<ChPrototypeXType,1,CH_PROTOTYPE_X_NUM,1> = PubSubChannel::new();
*/

// 1. blink任务的信道
/// 信号大小
const BLINK_MODE_NUM: usize = 1;
/// 信号枚举定义
pub type BlinkModeType = crate::tasks::blink::BlinkMode;
/// 信号发送者
pub type BlinkModePub = Pub<BlinkModeType,BLINK_MODE_NUM>;
/// 信号订阅者
pub type BlinkModeSub = Sub<BlinkModeType,BLINK_MODE_NUM>;
/// 定义信道
pub static BLINK_MODE : Ch<BlinkModeType,BLINK_MODE_NUM> = PubSubChannel::new();

// 2. motors任务的信道
/// 电机转速
const MOTOR_SPEED_NUM: usize = 2;
/// 可对电机进行单独设置而不影响其他电机
pub type MotorSpeedType = (Option<u16>, Option<u16>, Option<u16>, Option<u16>);
pub type MotorSpeedPub = Pub<MotorSpeedType,MOTOR_SPEED_NUM>;
pub type MotorSpeedSub = Sub<MotorSpeedType,MOTOR_SPEED_NUM>;
pub static MOTOR_SPEED : Ch<MotorSpeedType,MOTOR_SPEED_NUM> = PubSubChannel::new();

/// 电机方向
const MOTOR_DIR_NUM: usize = 2;
/// 可对电机进行单独设置而不影响其他电机
pub type MotorDirType = (Option<bool>, Option<bool>, Option<bool>, Option<bool>);
pub type MotorDirPub = Pub<MotorDirType,MOTOR_DIR_NUM>;
pub type MotorDirSub = Sub<MotorDirType,MOTOR_DIR_NUM>;
pub static MOTOR_DIR : Ch<MotorDirType,MOTOR_DIR_NUM> = PubSubChannel::new();

/// 电机状态
const MOTOR_STATE_NUM: usize = 2;
pub type MotorStateType = crate::tasks::motors::MotorState;
pub type MotorStatePub = Pub<MotorStateType,MOTOR_STATE_NUM>;
pub type MotorStateSub = Sub<MotorStateType,MOTOR_STATE_NUM>;
pub static MOTOR_STATE : Ch<MotorStateType,MOTOR_STATE_NUM> = PubSubChannel::new();
