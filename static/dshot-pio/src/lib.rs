#![no_std]

#[cfg(feature = "embassy-rp")]
pub mod dshot_embassy_rp;

#[cfg(feature = "rp2040-hal")]
pub mod dshot_rp2040_hal;

pub trait DshotPioTrait<const N: usize> {
    fn command(&mut self, command: [u16;N]);
    fn reverse(&mut self, reverse: [bool;N]);
    fn throttle_clamp(&mut self, throttle: [u16;N]);
    fn throttle_minimum(&mut self);
}

// 各种Shot类型
pub enum ShotType{
    DSHOT150, 
    DSHOT300,
    DSHOT600,
    DSHOT1200,
    OneSHOT42,
    OneShot125,
    MultiShot
}

// 计算分频数
pub fn cal_clock_div(sys_clk_hz: u32, shot_type: ShotType) -> (u16, u8) {
    let bit_rate = match shot_type {
        ShotType::DSHOT150 => 150_000,
        ShotType::DSHOT300 => 300_000,
        ShotType::DSHOT600 => 600_000,
        ShotType::DSHOT1200 => 1_200_000,
        // 默认DShot600，或根据需要panic
        _ => 600_000, 
    };

    let denominator = 8u64 * bit_rate as u64;
    let sys_clk = sys_clk_hz as u64;
    
    let divider = (sys_clk / denominator) as u16;
    let remainder = sys_clk % denominator;
    // (remainder * 256)/denominator
    let fraction = ((remainder << 8) / denominator) as u8; 

    (divider, fraction)
}
