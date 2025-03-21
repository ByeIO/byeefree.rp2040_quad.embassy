#![allow(dead_code)]

// us延时函数别名
pub async fn delay_us(time : u64){
    embassy_time::Timer::after(embassy_time::Duration::from_micros(time)).await;
}

// ms延时函数别名
pub async fn delay_ms(time : u64){
    embassy_time::Timer::after(embassy_time::Duration::from_millis(time)).await;
}
