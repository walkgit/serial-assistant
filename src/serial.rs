// 修复未使用的导入
// use crate::utils;  // 注释掉这行
use serialport::SerialPort;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono;  // 添加 chrono 导入

#[derive(Default)]
pub struct SharedData {
    pub packet_buffer: Vec<u8>,
    pub plot_data: Vec<(f64, f64)>,
}

pub fn open_port(
    port_name: &str,
    baud_rate: u32,
    data_bits: serialport::DataBits,
    stop_bits: serialport::StopBits,
    parity: serialport::Parity,
    received_data_shared: Arc<Mutex<String>>,
    show_timestamp: bool,
    auto_newline: bool
) -> Option<Arc<Mutex<Box<dyn SerialPort>>>> {
    if let Ok(port) = serialport::new(port_name, baud_rate)
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .timeout(Duration::from_millis(10))
        .open()
    {
        let port_handle = Arc::new(Mutex::new(port));
        
        let port_clone = Arc::clone(&port_handle);
        let received_data_clone = Arc::clone(&received_data_shared);
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            
            loop {
                match port_clone.lock() {
                    Ok(mut port) => {
                        match port.read(&mut buf) {
                            Ok(bytes_read) if bytes_read > 0 => {
                                if let Ok(mut received) = received_data_clone.lock() {
                                    // 添加时间戳
                                    if show_timestamp {
                                        let timestamp = chrono::Local::now().format("[%H:%M:%S%.3f] ");
                                        received.push_str(&timestamp.to_string());
                                    }
                                    
                                    // 直接将原始字节数据编码为特殊字符
                                    for byte in &buf[..bytes_read] {
                                        received.push(char::from_u32(0xFF00 + *byte as u32).unwrap_or('?'));
                                    }
                                    
                                    // 添加换行符
                                    if auto_newline {
                                        received.push('\n');
                                    }
                                }
                            },
                            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                                // 超时是正常的，继续尝试
                            },
                            Err(_) => {
                                // 其他错误，短暂休眠后继续
                                thread::sleep(Duration::from_millis(50));
                            },
                            _ => {}
                        }
                    },
                    Err(_) => {
                        // 无法获取锁，短暂休眠后继续
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                thread::sleep(Duration::from_millis(5));
            }
        });
        
        Some(port_handle)
    } else {
        None
    }
}

pub fn parse_packet(packet_buffer: &mut Vec<u8>, plot_data: &mut Vec<(f64, f64)>) {
    while packet_buffer.len() >= 7 {  // AA 55 03 01 XX XX CS 格式需要至少7个字节
        // 查找起始符
        if packet_buffer[0] == 0xAA && packet_buffer[1] == 0x55 {
            // 检查长度字节
            let length = packet_buffer[2] as usize;
            
            // 检查数据包是否完整
            if packet_buffer.len() >= length + 3 {  // 起始符(2) + 长度(1) + 数据(length)
                // 提取命令和数据
                let cmd = packet_buffer[3];
                
                if cmd == 0x01 && length == 3 {  // 命令0x01，数据长度为3
                    // 提取数据值
                    let high_byte = packet_buffer[4];
                    let low_byte = packet_buffer[5];
                    let value = ((high_byte as u16) << 8) | (low_byte as u16);
                    
                    // 获取当前时间作为X轴
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs_f64();
                    
                    // 添加到绘图数据
                    plot_data.push((now, value as f64));
                    
                    // 限制数据点数量
                    if plot_data.len() > 1000 {
                        plot_data.remove(0);
                    }
                }
                
                // 移除已处理的数据包
                packet_buffer.drain(0..length + 3);
            } else {
                // 数据包不完整，等待更多数据
                break;
            }
        } else {
            // 不是起始符，移除一个字节
            packet_buffer.remove(0);
        }
    }
}

// 将 process_packet 函数改为 pub，因为它在 parse_packet_data 中被使用
pub fn process_packet(packet: &[u8], plot_data: &mut Vec<(f64, f64)>) {
    if packet.len() < 7 {  // AA 55 03 01 XX XX CS
        return;
    }
    
    // 验证起始符
    if packet[0] != 0xAA || packet[1] != 0x55 {
        return;
    }
    
    // 验证长度
    let length = packet[2] as usize;
    if length != 3 || packet.len() != length + 4 {  // +4 是因为起始符(2)+长度(1)+校验(1)
        return;
    }
    
    // 验证命令
    if packet[3] != 0x01 {
        return;
    }
    
    // 计算校验和
    let checksum = packet[3] ^ packet[4] ^ packet[5];
    if checksum != packet[6] {
        return;
    }
    
    // 提取数据
    let high_byte = packet[4];
    let low_byte = packet[5];
    let value = ((high_byte as u16) << 8) | (low_byte as u16);
    
    // 获取当前时间
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    
    // 添加到绘图数据
    plot_data.push((time, value as f64));
    
    // 限制数据点数量
    if plot_data.len() > 1000 {
        plot_data.remove(0);
    }
}