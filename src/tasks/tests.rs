// 引入测试函数
use crate::drivers::imu::atk_imu901::tests::test_state_machine::test_state_machine;

/// 测试重要函数的任务
pub async fn test_task(){
    defmt::println!("hello from tasks::test_task");
    
    test_state_machine().await;
    
}
