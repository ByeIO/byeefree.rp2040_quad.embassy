// 1. 闪灯任务
pub mod blink;

// 2. usb协议栈处理任务
pub mod usb;

// 3. 交互式命令行任务
pub mod shell_cli;

// 4. 电机控制任务
pub mod motors;

// 5. 传感器数据处理任务
pub mod sensors;

// 6. 重要函数功能测试任务
pub mod tests;

// 7. 滤波器任务
pub mod filters;

// 8. 姿态控制器任务
pub mod controllers;

// 9. 起飞任务
pub mod take_off;

// 10. 降落任务
pub mod landing;

// 11. 悬停任务
pub mod hovering;

// 12. 绕圈任务
pub mod circling;

// 13. 前后左右移动任务
pub mod moving;

// 14. 飞行模式管理任务
pub mod flights;
