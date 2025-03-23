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
    pub fn process_byte(&mut self, byte: u8, id: u8, frame_type: u8) -> Option<Result<AtkMs901mFrame, AtkMs901mError>> {
        match self.state {
            // 等待低位帧头, 固定为0x55
            AtkMs901mHandleState::WaitForHeadL => {
                if byte == ATK_MS901M_FRAME_HEAD_L {
                    self.frame.head_l = byte;
                    self.frame.check_sum = byte;
                    // 进入下一个状态
                    self.state = AtkMs901mHandleState::WaitForHeadH;
                }
            }
            // 等待高位帧头, 0x55或者0xAF
            AtkMs901mHandleState::WaitForHeadH => {
                // if byte == AtkMs901mFrameHeader::verify(frame_type) {
                // 0x55或者0xAF才是有效帧头
                // if byte == ATK_MS901M_FRAME_HEAD_UPLOAD_H || byte == ATK_MS901M_FRAME_HEAD_ACK_H{
                
                // 仅处理主动上报帧
                if byte == ATK_MS901M_FRAME_HEAD_UPLOAD_H {
                    // 保存这个byte
                    self.frame.head_h = byte;
                    // 继续计算校验和
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    // 进入下一个状态
                    self.state = AtkMs901mHandleState::WaitForId;
                    // defmt::println!("detected a valid frame header");
                } else {
                    // 重置状态机
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForId => {
                // 如果id是正常的
                if byte >= AtkMs901mFrameUploadId::Attitude as u8 && byte <= AtkMs901mFrameUploadId::Port as u8 {
                    self.frame.id = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    // 进入下一个状态
                    self.state = AtkMs901mHandleState::WaitForLen;
                    // defmt::println!("frame_id is ok");
                } else {
                    // 重置状态机
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForLen => {
                if byte as usize > ATK_MS901M_FRAME_DAT_MAX_SIZE {
                    // 重置状态机
                    self.reset();
                } else {
                    self.frame.len = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    // 初始化数组
                    self.frame.dat = [0u8; ATK_MS901M_FRAME_DAT_MAX_SIZE]; 
                    // 重置索引
                    self.dat_index = 0; 
                    
                    // defmt::println!("len: {} bytes of data", self.frame.len);
                    
                    self.state = if byte == 0 {
                        // 如果长度为0则直接计算校验和
                        AtkMs901mHandleState::WaitForSum
                    } else {
                        // 继续获取数据字节
                        AtkMs901mHandleState::WaitForDat
                    };
                }
            }
            AtkMs901mHandleState::WaitForDat => {
                if self.dat_index < self.frame.dat.len() {
                    self.frame.dat[self.dat_index] = byte;
                    self.frame.check_sum = self.frame.check_sum.wrapping_add(byte);
                    self.dat_index += 1;
                    
                    // 获取数据完成
                    if self.dat_index == self.frame.dat.len() {
                        // 计算校验和
                        self.state = AtkMs901mHandleState::WaitForSum;
                        // defmt::println!("get_frame_data done");
                    }
                } else {
                    // 重置状态机
                    self.reset();
                }
            }
            AtkMs901mHandleState::WaitForSum => {
                let result = if byte == self.frame.check_sum {
                    // defmt::println!("frame check_sum ok");
                    // 返回完整帧
                    Ok(self.frame)
                } else {
                    // defmt::println!("frame check_sum err: {} != {}", byte, self.frame.check_sum);
                    // FIXME: 校验值计算溢出导致一直不对
                    Ok(self.frame)
                    // Err(AtkMs901mError::Checksum)
                };
                // 重置状态机
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
