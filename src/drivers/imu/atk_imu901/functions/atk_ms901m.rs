// 串口
use embassy_rp::uart::Uart;

// 内部库
/// 状态机
use super::frame_state_machine::FrameStateMachine;

/// 类型定义
use super::super::defines::*;

/// 模块交互类
pub struct AtkMs901m {
    uart: embassy_rp::uart::Uart<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async, >,
}

impl AtkMs901m {
    /// 构造函数
    pub async fn new(
        uart: Uart<'static, embassy_rp::peripherals::UART1, embassy_rp::uart::Async, >
    ) -> Self {
        AtkMs901m { uart }
    }
    
    /// 基于DMA的异步UART帧接收函数（增强版）
    /// 特点：
    /// 1. 使用DMA批量传输特性
    /// 2. Embassy异步定时器精确控制
    /// 3. 全状态错误恢复机制
    pub async fn get_frame_by_id(
        &mut self,
        frame: &mut AtkMs901mFrame,
        id: u8,
        id_type: u8,
        timeout_ms: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut state_machine = FrameStateMachine::new();
        let mut buffer = [0u8; 256]; // DMA缓冲区（根据UART FIFO深度调整）
        let deadline = embassy_time::Instant::now() + embassy_time::Duration::from_millis(timeout_ms as u64);
    
        // 主接收循环
        loop {
            // 异步等待数据到达（DMA传输完成）
            match self.uart.read(&mut buffer).await {
                Ok(_) => {
                    // 关键点：逐字节处理DMA缓冲区
                    for &byte in &buffer {
                        if let Some(result) = state_machine.process_byte(byte, id, id_type) {
                            *frame = state_machine.frame;
                            return result;
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
    
            // 超时检查
            if embassy_time::Instant::now() > deadline {
                return Err(AtkMs901mError::Timeout);
            }
    
            // 异步等待避免忙循环
            embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;
        }
    }
    
    /// 通过id读取寄存器
    pub async fn read_reg_by_id(
        &mut self,
        id: u8,
        dat: &mut [u8],
        timeout: u32,
    ) -> Result<usize, AtkMs901mError> {
        let mut buf = [0u8; 7];
        let mut frame = AtkMs901mFrame::default();

        buf[0] = ATK_MS901M_FRAME_HEAD_L;
        buf[1] = ATK_MS901M_FRAME_HEAD_ACK_H;
        buf[2] = ATK_MS901M_READ_REG_ID(id);
        buf[3] = 1;
        buf[4] = 0;
        // 使用wrapping_add防止加法溢出
        buf[5] = buf[0]
        .wrapping_add(buf[1])
        .wrapping_add(buf[2])
        .wrapping_add(buf[3])
        .wrapping_add(buf[4]);
        
        // 发送请求
        if self.uart.write(&buf[..6]).await.is_ok(){};

        if self.get_frame_by_id(&mut frame, id, ATK_MS901M_FRAME_ID_TYPE_ACK, timeout).await.is_ok() {
            for (i, &val) in frame.dat.iter().enumerate().take(frame.len as usize) {
                dat[i] = val;
            }
            Ok(frame.len as usize)
        } else {
            Err(AtkMs901mError::Invalid)
        }
    }
    
    /// 通过id写入寄存器
    pub async fn write_reg_by_id(
        &mut self,
        id: u8,
        len: u8,
        dat: &[u8],
    ) -> Result<(), AtkMs901mError> {
        let mut buf = [0u8; 7];

        buf[0] = ATK_MS901M_FRAME_HEAD_L;
        buf[1] = ATK_MS901M_FRAME_HEAD_ACK_H;
        buf[2] = ATK_MS901M_WRITE_REG_ID(id);
        buf[3] = len;

        match len {
            1 => {
                buf[4] = dat[0];
                // 使用wrapping_add防止加法溢出
                buf[5] = buf[0]
                .wrapping_add(buf[1])
                .wrapping_add(buf[2])
                .wrapping_add(buf[3])
                .wrapping_add(buf[4]);
                let _ = self.uart.write(&buf[..6]).await;
            }
            2 => {
                buf[4] = dat[0];
                buf[5] = dat[1];
                // 使用wrapping_add防止加法溢出
                buf[5] = buf[0]
                .wrapping_add(buf[1])
                .wrapping_add(buf[2])
                .wrapping_add(buf[3])
                .wrapping_add(buf[4])
                .wrapping_add(buf[5]);
                
                let _ = self.uart.write(&buf[..7]).await;
            }
            _ => return Err(AtkMs901mError::Invalid),
        }

        Ok(())
    }

    /// 初始化传感器
    pub async fn init(&mut self, baudrate: u32) -> Result<(), AtkMs901mError> {
        
        // self.uart.init(baudrate);

        let mut fsr = AtkMs901mFsr::default();
        if self.read_reg_by_id(
            AtkMs901mFrameAckId::RegGyroFsr as u8,  // 转换为u8
            &mut [fsr.gyro],  // 创建单元素数组的可变切片
            100
        ).await.is_err() {
            return Err(AtkMs901mError::Error);
        }

        if self.read_reg_by_id(
            AtkMs901mFrameAckId::RegAccFsr as u8, 
            &mut [fsr.accelerometer], 
            100
        ).await.is_err() {
            return Err(AtkMs901mError::Error);
        }

        Ok(())
    }

    /// 获取高度
    pub async fn get_attitude(
        &mut self,
        attitude_dat: &mut AtkMs901mAttitudeData,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::Attitude.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            attitude_dat.roll = ((frame.dat[1] as i16) << 8 | frame.dat[0] as i16) as f32 / 32768.0 * 180.0;
            attitude_dat.pitch = ((frame.dat[3] as i16) << 8 | frame.dat[2] as i16) as f32 / 32768.0 * 180.0;
            attitude_dat.yaw = ((frame.dat[5] as i16) << 8 | frame.dat[4] as i16) as f32 / 32768.0 * 180.0;
            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取四元数
    pub async fn get_quaternion(
        &mut self,
        quaternion_dat: &mut AtkMs901mQuaternionData,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::Quat.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            quaternion_dat.q0 = ((frame.dat[1] as i16) << 8 | frame.dat[0] as i16) as f32 / 32768.0;
            quaternion_dat.q1 = ((frame.dat[3] as i16) << 8 | frame.dat[2] as i16) as f32 / 32768.0;
            quaternion_dat.q2 = ((frame.dat[5] as i16) << 8 | frame.dat[4] as i16) as f32 / 32768.0;
            quaternion_dat.q3 = ((frame.dat[7] as i16) << 8 | frame.dat[6] as i16) as f32 / 32768.0;
            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取加速度
    pub async fn get_gyro_accelerometer(
        &mut self,
        gyro_dat: Option<&mut AtkMs901mGyroData>,
        accelerometer_dat: Option<&mut AtkMs901mAccelerometerData>,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();
        let mut fsr = AtkMs901mFsr::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::GyroAcce.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            if let Some(gyro_dat) = gyro_dat {
                gyro_dat.raw.x = (frame.dat[7] as i16) << 8 | frame.dat[6] as i16;
                gyro_dat.raw.y = (frame.dat[9] as i16) << 8 | frame.dat[8] as i16;
                gyro_dat.raw.z = (frame.dat[11] as i16) << 8 | frame.dat[10] as i16;

                gyro_dat.x = gyro_dat.raw.x as f32 / 32768.0 * G_ATK_MS901M_GYRO_FSR_TABLE[fsr.gyro as usize] as f32;
                gyro_dat.y = gyro_dat.raw.y as f32 / 32768.0 * G_ATK_MS901M_GYRO_FSR_TABLE[fsr.gyro as usize] as f32;
                gyro_dat.z = gyro_dat.raw.z as f32 / 32768.0 * G_ATK_MS901M_GYRO_FSR_TABLE[fsr.gyro as usize] as f32;
            }

            if let Some(accelerometer_dat) = accelerometer_dat {
                accelerometer_dat.raw.x = (frame.dat[1] as i16) << 8 | frame.dat[0] as i16;
                accelerometer_dat.raw.y = (frame.dat[3] as i16) << 8 | frame.dat[2] as i16;
                accelerometer_dat.raw.z = (frame.dat[5] as i16) << 8 | frame.dat[4] as i16;

                accelerometer_dat.x = accelerometer_dat.raw.x as f32 / 32768.0 * G_ATK_MS901M_ACCELEROMETER_FSR_TABLE[fsr.accelerometer as usize] as f32;
                accelerometer_dat.y = accelerometer_dat.raw.y as f32 / 32768.0 * G_ATK_MS901M_ACCELEROMETER_FSR_TABLE[fsr.accelerometer as usize ] as f32;
                accelerometer_dat.z = accelerometer_dat.raw.z as f32 / 32768.0 * G_ATK_MS901M_ACCELEROMETER_FSR_TABLE[fsr.accelerometer as usize] as f32;
            }

            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取磁力计数据
    pub async fn get_magnetometer(
        &mut self,
        magnetometer_dat: &mut AtkMs901mMagnetometerData,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::Mag.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            magnetometer_dat.x = (frame.dat[1] as i16) << 8 | frame.dat[0] as i16;
            magnetometer_dat.y = (frame.dat[3] as i16) << 8 | frame.dat[2] as i16;
            magnetometer_dat.z = (frame.dat[5] as i16) << 8 | frame.dat[4] as i16;
            magnetometer_dat.temperature = ((frame.dat[7] as i16) << 8 | frame.dat[6] as i16) as f32 / 100.0;
            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取气压计数据
    pub async fn get_barometer(
        &mut self,
        barometer_dat: &mut AtkMs901mBarometerData,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::Baro.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            barometer_dat.pressure = (frame.dat[3] as i32) << 24
                | (frame.dat[2] as i32) << 16
                | (frame.dat[1] as i32) << 8
                | frame.dat[0] as i32;
            barometer_dat.altitude = (frame.dat[7] as i32) << 24
                | (frame.dat[6] as i32) << 16
                | (frame.dat[5] as i32) << 8
                | frame.dat[4] as i32;
            barometer_dat.temperature = ((frame.dat[9] as i16) << 8 | frame.dat[8] as i16) as f32 / 100.0;
            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取可编程端口
    pub async fn get_port(
        &mut self,
        port_dat: &mut AtkMs901mPortData,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut frame = AtkMs901mFrame::default();

        if self.get_frame_by_id(
            &mut frame,
            AtkMs901mFrameUploadId::Port.into(),
            ATK_MS901M_FRAME_ID_TYPE_UPLOAD,
            timeout,
        ).await.is_ok() {
            port_dat.d0 = (frame.dat[1] as u16) << 8 | frame.dat[0] as u16;
            port_dat.d1 = (frame.dat[3] as u16) << 8 | frame.dat[2] as u16;
            port_dat.d2 = (frame.dat[5] as u16) << 8 | frame.dat[4] as u16;
            port_dat.d3 = (frame.dat[7] as u16) << 8 | frame.dat[6] as u16;
            Ok(())
        } else {
            Err(AtkMs901mError::Error)
        }
    }

    /// 获取led指示灯状态
    pub async fn get_led_state(
        &mut self,
        state: &mut AtkMs901mLedState,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let mut state_byte = 0u8;
        if self.read_reg_by_id(AtkMs901mFrameAckId::RegLedOff as u8, &mut [state_byte], timeout).await.is_ok() {
            return Err(AtkMs901mError::Error);
        }
        *state = match state_byte {
            0 => AtkMs901mLedState::On,
            1 => AtkMs901mLedState::Off,
            _ => return Err(AtkMs901mError::Invalid),
        };
        Ok(())
    }

    /// 设置led指示灯状态
    pub async fn set_led_state(
        &mut self,
        state: AtkMs901mLedState,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let state_byte = match state {
            AtkMs901mLedState::On => 0,
            AtkMs901mLedState::Off => 1,
        };
        let _ = self.write_reg_by_id(AtkMs901mFrameAckId::RegLedOff as u8, 1, &[state_byte]).await.is_ok();
        let mut state_recv = AtkMs901mLedState::On;
        let _ = self.get_led_state(&mut state_recv, timeout).await.is_ok();
        if state_recv != state {
            return Err(AtkMs901mError::Error);
        }
        Ok(())
    }

    /// 获取可编程端口模式
    pub async fn get_port_mode(
        &mut self,
        port: AtkMs901mPort,
        mode: &mut AtkMs901mPortMode,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D0 => AtkMs901mFrameAckId::RegD0Mode,
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Mode,
            AtkMs901mPort::D2 => AtkMs901mFrameAckId::RegD2Mode,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Mode,
        };
        let mut mode_byte = 0u8;
        if self.read_reg_by_id(id as u8, &mut [mode_byte], timeout).await.is_err() {
            return Err(AtkMs901mError::Error);
        }
        *mode = match mode_byte {
            0 => AtkMs901mPortMode::AnalogInput,
            1 => AtkMs901mPortMode::Input,
            2 => AtkMs901mPortMode::OutputHigh,
            3 => AtkMs901mPortMode::OutputLow,
            4 => AtkMs901mPortMode::OutputPwm,
            _ => return Err(AtkMs901mError::Invalid),
        };
        Ok(())
    }

    /// 设置可编程端口的模式
    pub async fn set_port_mode(
        &mut self,
        port: AtkMs901mPort,
        mode: AtkMs901mPortMode,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D0 => AtkMs901mFrameAckId::RegD0Mode,
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Mode,
            AtkMs901mPort::D2 => AtkMs901mFrameAckId::RegD2Mode,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Mode,
        };
        let mode_byte = match mode {
            AtkMs901mPortMode::AnalogInput => 0,
            AtkMs901mPortMode::Input => 1,
            AtkMs901mPortMode::OutputHigh => 2,
            AtkMs901mPortMode::OutputLow => 3,
            AtkMs901mPortMode::OutputPwm => 4,
        };
        let _ = self.write_reg_by_id(id as u8, 1, &[mode_byte]).await.is_ok();
        let mut mode_recv = AtkMs901mPortMode::AnalogInput;
        let _ = self.get_port_mode(port, &mut mode_recv, timeout).await.is_ok();
        if mode_recv != mode {
            return Err(AtkMs901mError::Error);
        }
        Ok(())
    }

    /// 获取可编程端口的pwm脉冲
    pub async fn get_port_pwm_pulse(
        &mut self,
        port: AtkMs901mPort,
        pulse: &mut u16,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Pulse,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Pulse,
            _ => return Err(AtkMs901mError::Invalid),
        };
        let mut pulse_bytes = [0u8; 2];
        if self.read_reg_by_id(id as u8, &mut pulse_bytes, timeout).await.is_err() {
            return Err(AtkMs901mError::Error);
        }
        *pulse = (pulse_bytes[1] as u16) << 8 | pulse_bytes[0] as u16;
        Ok(())
    }

    /// 设置可编程端口的pwm脉冲
    pub async fn set_port_pwm_pulse(
        &mut self,
        port: AtkMs901mPort,
        pulse: u16,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Pulse,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Pulse,
            _ => return Err(AtkMs901mError::Invalid),
        };
        let pulse_bytes = [(pulse & 0xFF) as u8, (pulse >> 8) as u8];
        let _ = self.write_reg_by_id(id as u8, 2, &pulse_bytes).await.is_ok();
        let mut pulse_recv = 0u16;
        let _ = self.get_port_pwm_pulse(port, &mut pulse_recv, timeout).await.is_ok();
        if pulse_recv != pulse {
            return Err(AtkMs901mError::Error);
        }
        Ok(())
    }

    /// 获取可编程端口的pwm脉冲周期
    pub async fn get_port_pwm_period(
        &mut self,
        port: AtkMs901mPort,
        period: &mut u16,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Period,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Period,
            _ => return Err(AtkMs901mError::Invalid),
        };
        let mut period_bytes = [0u8; 2];
        if self.read_reg_by_id(id as u8, &mut period_bytes, timeout).await.is_err() {
            return Err(AtkMs901mError::Error);
        }
        *period = (period_bytes[1] as u16) << 8 | period_bytes[0] as u16;
        Ok(())
    }

    /// 设置可编程端口的pwm脉冲周期
    pub async fn set_port_pwm_period(
        &mut self,
        port: AtkMs901mPort,
        period: u16,
        timeout: u32,
    ) -> Result<(), AtkMs901mError> {
        let id = match port {
            AtkMs901mPort::D1 => AtkMs901mFrameAckId::RegD1Period,
            AtkMs901mPort::D3 => AtkMs901mFrameAckId::RegD3Period,
            _ => return Err(AtkMs901mError::Invalid),
        };
        let period_bytes = [(period & 0xFF) as u8, (period >> 8) as u8];
        let _ = self.write_reg_by_id(id as u8, 2, &period_bytes).await.is_ok();
        let mut period_recv = 0u16;
        let _ = self.get_port_pwm_period(port, &mut period_recv, timeout).await.is_ok();
        if period_recv != period {
            return Err(AtkMs901mError::Error);
        }
        Ok(())
    }

}
