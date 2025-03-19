//! 小蜜蜂电调驱动

// dshot相关
use embassy_rp::peripherals::PIO0;
use dshot_pio::dshot_embassy_rp::*;
use dshot_pio::{ ShotType, cal_clock_div};

// 多任务相关
use embassy_time::{Duration, Timer};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;

pub async fn init(){
    // 打印调试信息
    defmt::println!("hello from drivers::esc::little_bee");
}
