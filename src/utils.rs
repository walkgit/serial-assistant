// 将字节数组转换为十六进制字符串
// 删除第一个 bytes_to_hex 函数，保留改进版本
// pub fn bytes_to_hex(bytes: &[u8]) -> String {
//     bytes.iter()
//         .map(|b| format!("{:02X} ", b))
//         .collect()
// }

// 将十六进制字符串转换为字节数组
pub fn hex_to_bytes(hex_str: &str) -> Vec<u8> {
    hex_str
        .split_whitespace()  // 按空格分割
        .filter_map(|s| {    // 过滤并转换
            if let Ok(byte) = u8::from_str_radix(s, 16) {
                Some(byte)
            } else {
                None
            }
        })
        .collect()
}

// 修改 bytes_to_hex 函数，使其格式与 hex_to_bytes 兼容
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::with_capacity(bytes.len() * 3);
    
    for (i, &byte) in bytes.iter().enumerate() {
        // 每个字节格式化为两位十六进制，后跟空格
        hex_string.push_str(&format!("{:02X} ", byte));
        
        // 每16个字节添加一个换行符，提高可读性
        if (i + 1) % 16 == 0 && i < bytes.len() - 1 {
            hex_string.push('\n');
        }
    }
    
    hex_string
}

// 解析数据包
pub fn parse_packet(packet_buffer: &mut Vec<u8>, plot_data: &mut Vec<(f64, f64)>) {
    while packet_buffer.len() >= 4 {  // 最小包长度：起始符(2) + 长度(1) + 校验(1)
        // 查找起始符
        if let Some(start_pos) = packet_buffer.windows(2)
            .position(|window| window == [0xAA, 0x55]) 
        {
            // 移除起始符之前的数据
            if start_pos > 0 {
                packet_buffer.drain(0..start_pos);
            }
            
            // 检查数据包是否完整
            if packet_buffer.len() >= 4 {
                let length = packet_buffer[2] as usize;
                let total_length = length + 4;  // 总长度 = 数据长度 + 起始符(2) + 长度字节(1) + 校验字节(1)
                
                if packet_buffer.len() >= total_length {
                    let packet = packet_buffer.drain(0..total_length).collect::<Vec<_>>();
                    process_packet(&packet, plot_data);
                } else {
                    break;  // 等待更多数据
                }
            }
        } else {
            // 没找到起始符，移除第一个字节
            packet_buffer.remove(0);
        }
    }
}

// 处理完整的数据包
fn process_packet(packet: &[u8], plot_data: &mut Vec<(f64, f64)>) {
    let data_length = packet[2] as usize;
    let data = &packet[3..3+data_length];
    let checksum = packet[3+data_length];
    
    // 计算校验和
    let calc_checksum = data.iter().fold(0u8, |acc, &x| acc ^ x);
    
    if calc_checksum == checksum {
        match data[0] {  // 命令字节
            0x01 => {  // 绘图数据
                if data.len() >= 3 {  // 至少需要命令字节+2字节数据
                    let value = ((data[1] as u16) << 8 | data[2] as u16) as f64;
                    let time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64();
                    plot_data.push((time, value));
                    
                    // 限制数据点数量
                    if plot_data.len() > 1000 {
                        plot_data.remove(0);
                    }
                }
            },
            _ => {}
        }
    }
}

// 解析数据包的辅助函数
pub fn parse_packet_data(packet_buffer: &mut Vec<u8>, plot_data: &mut Vec<(f64, f64)>) {
    while packet_buffer.len() >= 4 {  // 最小包长度：起始符(2) + 长度(1) + 校验(1)
        // 查找起始符
        if let Some(start_pos) = packet_buffer.windows(2)
            .position(|window| window == [0xAA, 0x55]) 
        {
            // 移除起始符之前的数据
            if start_pos > 0 {
                packet_buffer.drain(0..start_pos);
            }
            
            // 检查数据包是否完整
            if packet_buffer.len() >= 4 {
                let length = packet_buffer[2] as usize;
                let total_length = length + 4;  // 总长度 = 数据长度 + 起始符(2) + 长度字节(1) + 校验字节(1)
                
                if packet_buffer.len() >= total_length {
                    let packet = packet_buffer.drain(0..total_length).collect::<Vec<_>>();
                    process_packet(&packet, plot_data);
                } else {
                    break;  // 等待更多数据
                }
            }
        } else {
            // 没找到起始符，移除第一个字节
            packet_buffer.remove(0);
        }
    }
}