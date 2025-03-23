#![allow(unused)]

//! lqr姿态控制器

/// 类型
use crate::utils::types::sensor::Imu10DofData;
/// 全局变量
use crate::utils::variables;
/// 数学计算
use nalgebra::{Quaternion, Vector3};
use libm::{acos, sqrtf};

/* start 系统参数配置 */
/// 控制周期(s)
const DT: f32 = 0.02;               
/// 基础油门量(悬停状态)
const BASE_THRUST: f32 = 30.0;      
/// 最大油门限制
const MAX_THRUST: f32 = 100.0;      
/// 最小油门限制
const MIN_THRUST: f32 = 0.0;        

// LQR增益参数 (需要根据实际系统调整)
/// (角度增益, 角速度增益)
const K_ROLL: (f32, f32) = (5.0, 0.3);  
const K_PITCH: (f32, f32) = (5.0, 0.3);
const K_YAW: (f32, f32) = (2.0, 0.1);
/* end 系统参数配置 */ 

/// LQR姿态控制器核心算法
/// 系统要素:
/// - 反馈量: 四元数误差、角速度误差
/// - 被控量: 飞行器姿态角
/// - 调节量: 电机推力差
pub async fn lqr_controller(
    in_quat_wanted : [f32; 4],
){
    // 获取当前位姿
    let imu_data_now = variables::Imu10DofDataEditor::get_data().await;
    let q_target = in_quat_wanted;
    let q_current = imu_data_now.quat;
    let gyro_current = imu_data_now.gyr;
    
    // 1. 四元数误差计算 (使用nalgebra库)
    let q_desired = Quaternion::from_parts(
        // w
        q_target[0], 
        Vector3::new(q_target[1], q_target[2], q_target[3])
    );
    let q_now = Quaternion::from_parts(
        q_current[0], 
        Vector3::new(q_current[1], q_current[2], q_current[3])
    );
    let q_error = q_desired * q_now.conjugate();
    
    // 2. 转换为轴角表示 (小角度近似)
    let (axis, angle) = if q_error.scalar() >= 1.0 {
        // 无旋转
        (Vector3::zeros(), 0.0)  
    } else {
        // 将标量转换为f64进行计算，结果再转回f32
        let scalar = q_error.scalar() as f64;
        let theta = 2.0 * acos(scalar) as f32;
        let axis = q_error.vector().normalize();
        (axis, theta)
    };
    
    // 3. 构建误差向量 (角度误差 + 角速度误差)
    // 等效旋转矢量
    let angle_error = axis * angle;         
    // 期望角速度为0
    let omega_error = Vector3::from_row_slice(&gyro_current) * -1.0; 
    
    // 4. LQR控制律计算
    let torque = Vector3::new(
        // Roll轴力矩
        -K_ROLL.0 * angle_error.x - K_ROLL.1 * omega_error.x,  
        // Pitch轴力矩 
        -K_PITCH.0 * angle_error.y - K_PITCH.1 * omega_error.y,
        // Yaw轴力矩
        -K_YAW.0 * angle_error.z - K_YAW.1 * omega_error.z     
    );
    
    // 5. 控制分配 (X型布局混控)
    let motors = control_allocation(
        BASE_THRUST,
        torque.x, 
        torque.y,
        torque.z
    ).await;
    
    // 6. 发布电机控制指令
    crate::signals::MOTOR_SPEED.publisher().unwrap().publish_immediate((
        Some(clamp_thrust(motors[0]).await), 
        Some(clamp_thrust(motors[1]).await),
        Some(clamp_thrust(motors[2]).await),
        Some(clamp_thrust(motors[3]).await)
    ));
    
}

/// X型四旋翼控制分配矩阵
/// 参数:
/// - thrust: 基础推力
/// - roll:  横滚力矩 (+向右倾斜)
/// - pitch: 俯仰力矩 (+向前倾斜)
/// - yaw:   偏航力矩 (+顺时针旋转)
/// 返回: [m1, m2, m3, m4] 四个电机的推力百分比
pub async fn control_allocation(thrust: f32, roll: f32, pitch: f32, yaw: f32) -> [f32; 4] {
    [
        // 右前电机 (顺时针)
        thrust + roll + pitch - yaw,   
        // 左后电机 (逆时针) 
        thrust - roll + pitch + yaw,   
        // 左前电机 (逆时针)
        thrust - roll - pitch - yaw,   
        // 右后电机 (顺时针)
        thrust + roll - pitch + yaw    
    ]
}

/// 推力限幅保护
pub async fn clamp_thrust(value: f32) -> u16 {
    value.clamp(MIN_THRUST, MAX_THRUST) as u16
}
