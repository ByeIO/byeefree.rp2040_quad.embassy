// 内部库
use super::super::defines::*;

/// 帧解析状态机
pub struct FrameStateMachine {
    pub frame: AtkMs901mFrame,
    pub state: AtkMs901mHandleState,
    pub dat_index: usize,
}

impl FrameStateMachine {
    /// 构造函数
    pub fn new() -> Self {
        Self {
            frame: AtkMs901mFrame::default(),
            state: AtkMs901mHandleState::WaitForHeadL,
            dat_index: 0,
        }
    }

    /// 字节处理核心逻辑
    pub fn process_byte(&mut self, byte: u8, id: u8, id_type: u8) -> Option<Result<(), AtkMs901mError>> {
        match self.state {
            AtkMs901mHandleState::WaitForHeadL => {
                if byte == AtkMs901mFrameHeader::verify(id_type) {
                    self.frame.head_l = byte;
                    self.frame.check_sum = byte;
                    self.state = AtkMs901mHandleState::WaitForHeadH;
                }
            }
            AtkMs901mHandleState::WaitForHeadH => {
                if byte == AtkMs901mFrameHeader::verify(id_type) {
                    self.frame.head_h = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    self.state = AtkMs901mHandleState::WaitForId;
                } else {
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForId => {
                if byte == id {
                    self.frame.id = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    self.state = AtkMs901mHandleState::WaitForLen;
                } else {
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForLen => {
                if byte as usize > ATK_MS901M_FRAME_DAT_MAX_SIZE {
                    self.reset();
                } else {
                    self.frame.len = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    // 初始化数组
                    self.frame.dat = [0u8; ATK_MS901M_FRAME_DAT_MAX_SIZE]; 
                    // 重置索引
                    self.dat_index = 0; 
                    self.state = if byte == 0 {
                        AtkMs901mHandleState::WaitForSum
                    } else {
                        AtkMs901mHandleState::WaitForDat
                    };
                }
            }
            AtkMs901mHandleState::WaitForDat => {
                if self.dat_index < self.frame.dat.len() {
                    self.frame.dat[self.dat_index] = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    self.dat_index += 1;
                    
                    if self.dat_index == self.frame.dat.len() {
                        self.state = AtkMs901mHandleState::WaitForSum;
                    }
                } else {
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForSum => {
                let result = if byte == self.frame.check_sum {
                    Ok(())
                } else {
                    Err(AtkMs901mError::Checksum)
                };
                self.reset();
                return Some(result);
            }
        }
        None
    }

    /// 重置状态机
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
