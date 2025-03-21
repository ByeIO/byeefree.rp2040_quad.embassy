#![allow(dead_code)]

//! 测试状态机

use panic_probe as _;
use defmt::{assert, assert_eq, assert_ne, println};
use defmt_test;

// 构造示例数据
const DMA_SAMPLE_DATA: &[u8] = &[
    174, 85, 85, 6, 8, 12, 0, 2, 0, 0, 0, 6, 1, 205, 85, 85, 1, 6, 2, 228, 100, 255, 26, 103, 123, 85, 85, 2, 8, 143, 36, 228, 243, 158, 214, 214, 114, 250, 85, 85, 3, 12, 117, 0, 153, 235, 214, 24, 1, 0, 253, 255, 0, 0, 157, 85, 85, 6, 8, 0, 0, 0, 0, 0, 0, 6, 1, 191, 85, 85, 5, 10, 226, 139, 1, 0, 80, 255, 255, 255, 29, 11, 156, 85, 85, 1, 6, 2, 228, 100, 255, 26, 103, 123, 85, 85, 2, 8, 143, 36, 228, 243, 158, 214, 214, 114, 250, 85, 85, 3, 12, 122, 0, 158, 235, 207, 24, 15, 0, 245, 255, 253, 255, 162, 85, 85, 6, 8, 0, 0, 0, 0, 0, 0, 7, 1, 192, 85, 85, 1, 6, 2, 228, 100, 255, 26, 103, 123, 85, 85, 2, 8, 143, 36, 228, 243, 158, 214, 214, 114, 250, 85, 85, 3, 12, 127, 0, 155, 235, 205, 24, 2, 0, 0, 0, 255, 255, 163, 85, 85, 6, 8, 0, 0, 0, 0, 0, 0, 6, 1, 191, 85, 85, 1, 6, 2, 228, 100, 255, 26, 103, 123, 85, 85, 2, 8, 143, 36, 228, 243, 157, 214, 214, 114, 249, 85, 85, 3, 12, 128, 0, 173, 235, 202, 24, 1, 0, 1, 0, 2, 0, 183, 85, 85, 6, 8, 0, 0, 0, 0, 0, 0, 5, 1, 190, 85, 85, 1, 6, 2, 228, 100, 255, 26, 103, 123
];

/// 测试帧解析状态机
pub async fn test_state_machine(){
    use super::super::defines::*;
    use super::super::functions::frame_state_machine::*;
    use super::super::functions::atk_ms901m::*;
    
    defmt::println!("hello from test_state_machine");
    let mut sm = FrameStateMachine::new();
    let expected_types = [AtkMs901mFrameHeader::UploadH; 3];
    // 调用封装函数
    let mut frames : [AtkMs901mFrame; 32] = [AtkMs901mFrame::default(); 32];
    // 获取所有有效帧
    let frame_num = AtkMs901m::find_all_valid_frames(DMA_SAMPLE_DATA, &mut sm, &mut frames).await;
    defmt::println!("test_state_machine : frame_num {}", frame_num);
}
