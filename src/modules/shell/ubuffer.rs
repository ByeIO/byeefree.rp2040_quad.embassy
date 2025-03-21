use super::commands::CommandHandler;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embedded_cli::{cli::CliBuilder, Command};
use embedded_cli::arguments::{ FromArgument, FromArgumentError};
use embedded_io::{ErrorKind, ErrorType, Write as SyncWrite};
use embedded_io_async::{Read, Write};
use static_cell::StaticCell;
use ufmt::{uWrite, uDisplay};

// 强制多态
use core::ops::Deref;
use core::ops::DerefMut;

// 字符串
use core::str::FromStr;

/// 通用缓冲区结构，用于ufmt格式化输出
#[derive(Clone)]
pub struct UBuffer<const N: usize> {
    // 固定容量堆内存分配
    pub inner: heapless::Vec<u8, N>, 
}

impl<const N: usize> UBuffer<N> {
    pub fn new() -> Self {
        Self {
            inner: heapless::Vec::new(),
        }
    }
    
}

/// 实现Default特征
impl<const N: usize> Default for UBuffer<N> {
    fn default() -> Self {
        let mut buf = Self::new();
        buf.inner.extend_from_slice(b"World").unwrap();
        buf
    }
}

/// 实现uWrite trait以支持格式化写入
impl<const N: usize> uWrite for UBuffer<N> {
    type Error = embedded_io::ErrorKind;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.inner
            .extend_from_slice(s.as_bytes())
            .map_err(|_| ErrorKind::OutOfMemory)
    }
}

/// 实现uDisplay trait以支持格式化打印
impl<const N: usize> uDisplay for UBuffer<N> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.write_str(self.into())
    }
}

impl<const N: usize> Deref for UBuffer<N> {
    // 目标类型是字符串切片
    type Target = str;  
    
    // 通过内部 Vec<u8> 的字节数据构造 &str
    fn deref(&self) -> &Self::Target {
        // 安全前提：保证内部字节是有效的 UTF-8 编码
        unsafe { core::str::from_utf8_unchecked(&self.inner) }
    }
}

// 实现从 &UBuffer 到 &str 的显式转换
impl<'a, const N: usize> From<&'a UBuffer<N>> for &'a str {
    fn from(buffer: &'a UBuffer<N>) -> Self {
        // 生命周期通过 'a 显式绑定
        buffer.deref() 
    }
}

// 实现参数转换
impl<'a, const N: usize> FromArgument<'a> for UBuffer<N> {
    fn from_arg(arg: &'a str) -> Result<Self, FromArgumentError<'a>> {
        let mut buffer = UBuffer::new();
        // 将字符串字节写入缓冲区
        buffer.inner
            .extend_from_slice(arg.as_bytes())
            .map_err(|_| FromArgumentError {
                value: arg,
                expected: "a string fitting buffer capacity",
            })?;
        Ok(buffer)
    }
}

// 实现字符串转换特征
impl<const N: usize> FromStr for UBuffer<N> {
    type Err = FromArgumentError<'static>;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = UBuffer::new();
        buffer.inner.extend_from_slice(s.as_bytes())
            .map_err(|_| FromArgumentError {
                value : "",
                expected: "string length within buffer capacity"
            })?;
        Ok(buffer)
    }
}
