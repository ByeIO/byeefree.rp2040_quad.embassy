# 基于rp2040的四旋翼飞控固件(embassy框架版本)
- 参考[https://github.com/peterkrull/quad]和[https://github.com/holsatus/holsatus-flight]

## 项目目录
1. .cargo : 编译器配置
2. cargo-generate : (忽略)
3. docs : 相关文档
4. examples : 示例例程
5. src : 代码目录
6. static : 部分库代码
7. target : 编译产物
8. tests : 总体测试代码
9. vendor : 所有依赖库
10. Cargo.toml : 项目配置
11. memory.x : MCU内存分配
12. rust-toolchain.toml : 编译器版本配置

## 代码目录
1. drivers : 外设驱动(imu, esc)
    - imu : 惯性传感器单元
        * mpu6050.rs : mpu6050六轴传感器
        * atk_imu901.rs : 正点原子ATK-IMU901十轴传感器
    - esc : 电子调节器(电调)
        * little_bee.rs : 小蜜蜂电调
        * micoair_4in1.rs : 微空四合一电调
    - optical_flow : 光流位移传感器
        * micoair_mtf02p.rs : 微空MTF-02P光流传感器
    - tof : ToF距离传感器
        * benewake_tflc02.rs : 北醒TF-LC02激光测距传感器
2. filters : 滤波器
    - estimators : 位置估计
        * extended_kalman.rs : 扩展卡尔曼滤波器
        * linear_kalman.rs : 线性卡尔曼滤波器
3. tasks : 任务
    * blink.rs : led指示灯任务
    * circling.rs : 空中盘旋任务
    * demo.rs : 演示任务:起飞->盘旋->降落
    * landing.rs : 降落任务
    * logging.rs : 日志黑匣子任务
    * motors.rs : 电机控制任务@ESC
    * moving.rs : 空中前进后退左右移动任务
    * shell_cli.rs : 命令行解析任务
    * take_off.rs : 起飞任务
    * usb.rs : usb总线处理任务
    * motor_test.rs : 测试分别旋转四个电机任务
4. errors : 错误处理
5. calibration : 传感器校准/标定
6. types : 类型
7. signals : 信号量
8. utils : 工具
    * consts.rs : 常量
9. shell : 命令行交互
10. main.rs : 飞控程序入口

## 无人机构型
```rs
/// Quadcopter "x" configuration
/// ```text
///   front
/// M4     M2
///   \   /
///     |
///   /   \
/// M3     M1
/// ```
/// where \
/// `M1` spins CW \
/// `M2` spins CCW \
/// `M3` spins CCW \
/// `M4` spins CW
```