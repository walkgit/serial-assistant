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