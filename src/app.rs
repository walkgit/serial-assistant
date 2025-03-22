// 修复导入
use crate::ui;
use crate::utils;
use rfd::FileDialog;
use eframe::egui;
use serialport::{DataBits};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io::{Read, Write};  
use rlua::Lua;
use crate::serial::SerialPortHandle;
// 在 SerialAssistant 结构体中添加新字段
pub struct SerialAssistant {
    pub ports: Vec<serialport::SerialPortInfo>,
    pub selected_port: String,
    pub baud_rates: Vec<u32>,
    pub selected_baud: u32,
    pub port_handle: Option<SerialPortHandle>,  // 修改类型
    pub received_data: String,
    pub send_data: String,
    pub is_hex_display: bool,
    pub is_hex_send: bool,
    pub data_bits: DataBits,
    pub stop_bits: serialport::StopBits,
    pub parity: serialport::Parity,
    pub auto_send: bool,
    pub auto_send_interval: u64,
    pub auto_send_active: bool,
    pub last_send_time: Instant,
    pub bytes_received: usize,
    pub bytes_sent: usize,
    pub log_enabled: bool,
    pub log_file: Option<String>,
    pub received_data_shared: Arc<Mutex<String>>,
    pub received_buffer: Vec<u8>,
    pub auto_scroll: bool,
    pub show_raw_data: bool,
    pub status_message: String,
    pub last_stats_update: Instant,
    pub bytes_per_second: f32,
    pub packet_buffer: Vec<u8>,
    pub plot_data: Vec<(f64, f64)>,
    pub plot_visible: bool,
    pub plot_start_time: Option<f64>,  // 添加起始时间
    pub plot_time_span: f64,           // 添加时间跨度
    pub received_bytes: Vec<u8>,
    pub lua_script_path: String,  // 添加一个字段来存储Lua文件的路径
    pub lua_state: Option<Lua>,
    pub plot_data_per_channel: Vec<Vec<(f64, f64)>>,  // 添加一个字段来存储每个通道的绘图数据
    pub tcp_enabled: bool,
    pub tcp_address: String,
    pub tcp_port: String,
    pub tcp_stream: Option<Arc<Mutex<std::net::TcpStream>>>,
    pub tcp_connected: bool,
    pub show_help: bool, 
    pub custom_baud_text: String,
}

impl Default for SerialAssistant {
    fn default() -> Self {
        Self {
            ports: serialport::available_ports().unwrap_or_default(),
            selected_port: String::new(),
            baud_rates: vec![9600, 19200, 38400, 57600, 115200],
            selected_baud: 115200,
            port_handle: None,
            received_data: String::new(),
            send_data: String::new(),
            is_hex_display: false,
            is_hex_send: false,
            data_bits: DataBits::Eight,
            stop_bits: serialport::StopBits::One,
            parity: serialport::Parity::None,
            auto_send: false,
            auto_send_interval: 1000,
            auto_send_active: false,
            last_send_time: Instant::now(),
            bytes_received: 0,
            bytes_sent: 0,
            log_enabled: false,
            log_file: None,
            received_data_shared: Arc::new(Mutex::new(String::new())),
            received_buffer: Vec::new(),
            auto_scroll: true,
            show_raw_data: false,
            status_message: String::new(),
            last_stats_update: Instant::now(),
            bytes_per_second: 0.0,
            packet_buffer: Vec::new(),
            plot_data: Vec::with_capacity(1000),
            plot_visible: false,
            plot_start_time: None,
            plot_time_span: 10.0,  // 默认显示10秒数据
            received_bytes: Vec::new(),  // 添加这一行
            plot_data_per_channel: vec![Vec::with_capacity(1000); 10],
            lua_script_path: String::from("config/waveform.lua"),
            lua_state: None,
            tcp_enabled: false,
            tcp_address: String::from("127.0.0.1"),
            tcp_port: String::from("8080"),
            tcp_stream: None,
            tcp_connected: false,
            show_help: false, 
            custom_baud_text: String::from("256000")
        }
    }
}

impl SerialAssistant {
    // 更新状态信息
    pub fn update_status(&mut self) {         
        if self.tcp_enabled == true {
            self.status_message = format!("串口: {} {}|状态: {}| 收发速率: {:.1} KB/s",
            self.tcp_address,
            self.tcp_port,
            if self.tcp_connected { "已连接" } else { "未连接" },
            self.bytes_per_second / 1024.0);           
        } else {
            self.status_message = format!("串口: {}|状态: {}| 波特率: {} | 收发速率: {:.1} KB/s",
            self.selected_port,
            if self.port_handle.is_some() { "已打开" } else { "未打开" },
            self.selected_baud,
            self.bytes_per_second / 1024.0);
        }        
    }

    // 计算传输速率
    pub fn update_transfer_rate(&mut self) {
        let elapsed = self.last_stats_update.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            self.bytes_per_second = (self.bytes_received + self.bytes_sent) as f32 / elapsed;
            self.last_stats_update = Instant::now();
        }
    }

    // 记录数据
    pub fn log_data_with_lock(&self, data: &[u8], is_received: bool) {
        if self.log_enabled {
            if let Some(ref path) = self.log_file {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path) 
                {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    let direction = if is_received { "RX" } else { "TX" };
                    let hex_data = utils::bytes_to_hex(data);
                    
                    // 将原始数据转换为字符串形式
                    let raw_data_string = String::from_utf8_lossy(data);
                    let _ = writeln!(file, "[{}] {}: raw_data: {}", timestamp, direction, raw_data_string);
                    
                    let _ = writeln!(file, "[{}] {}: {}", timestamp, direction, hex_data);
                }
            }
        }
    }

     // 初始化Lua环境
    pub fn init_lua(&mut self) {         
        if self.lua_state.is_none() {
            let lua = Lua::new();

            // 尝试加载外部Lua脚本文件
            if let Err(e) = lua.context(|ctx| {
                let script_content = std::fs::read_to_string(&self.lua_script_path)
                    .expect("无法读取Lua脚本文件");
                ctx.load(&script_content).exec()
            }) {
                println!("Lua脚本加载失败: {}", e);
            }
            
            self.lua_state = Some(lua);
            println!("Lua环境初始化成功");
        }
    }

    pub fn open_port(&mut self) -> bool {
        // 检查是否选择了串口
        if self.selected_port.is_empty() {
            println!("未选择串口");
            return false;
        }

        // 处理自定义波特率
        if self.selected_baud == 0 {
            if let Ok(baud) = self.custom_baud_text.parse::<u32>() {
                if baud > 0 {
                    self.selected_baud = baud;
                } else {
                    println!("无效的波特率值");
                    return false;
                }
            } else {
                println!("波特率解析失败");
                return false;
            }
        }

        // 确保之前的串口已经完全关闭并释放资源
        if self.port_handle.is_some() {
            if let Some(port) = &self.port_handle {
                if let Ok(port) = port.try_lock() {  // 移除 mut
                    let _ = port.clear(serialport::ClearBuffer::All);
                }
            }
            self.port_handle.take();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // 刷新可用串口列表
        self.ports = serialport::available_ports().unwrap_or_default();
        
        // 验证选择的串口是否在可用列表中
        if !self.ports.iter().any(|p| p.port_name == self.selected_port) {
            println!("选择的串口不可用: {}", self.selected_port);
            return false;
        }
        
        println!("正在打开串口: {} 波特率: {}", self.selected_port, self.selected_baud);
        if let Some(port_handle) = crate::serial::open_port(
            &self.selected_port,
            self.selected_baud,
            self.data_bits,
            self.stop_bits,
            self.parity,
            Arc::clone(&self.received_data_shared),
        ) {
            self.port_handle = Some(port_handle);
            println!("串口打开成功");
            true
        } else {
            println!("串口打开失败");
            false
        }
    }

    // 修复 close_port 函数
    pub fn close_port(&mut self) -> bool {
        if let Some(port_handle) = self.port_handle.take() {
            crate::serial::close_port(port_handle);
            
            // 重置相关状态
            self.bytes_received = 0;
            self.bytes_sent = 0;
            self.bytes_per_second = 0.0;
            self.packet_buffer.clear();
            self.received_buffer.clear();
            
            println!("串口已关闭");
            true
        } else {
            println!("串口未打开");
            false
        }
    }

    pub fn connect_tcp(&mut self) -> bool {
        if let Ok(stream) = std::net::TcpStream::connect(format!("{}:{}", self.tcp_address, self.tcp_port)) {
            stream.set_nonblocking(true).unwrap();
            self.tcp_stream = Some(Arc::new(Mutex::new(stream)));
            self.tcp_connected = true;
            println!("TCP连接成功");
            true
        } else {
            println!("TCP连接失败");
            false
        }
    }

    pub fn disconnect_tcp(&mut self) {
        self.tcp_stream = None;
        self.tcp_connected = false;
        println!("TCP断开连接");
    }

    pub fn process_received_data(&mut self, data: &[u8]) {
        // 将数据添加到缓冲区
        self.packet_buffer.extend_from_slice(data);
        self.bytes_received += data.len();        
       
        println!("接收完成，处理数据: {:02X?}", self.packet_buffer);
        
        // TCP模式和串口模式都可以使用波形显示功能
        if self.plot_visible {
            // 使用Lua脚本解析数据帧
            if let Some(lua) = &self.lua_state {
                let mut frames_to_process = Vec::new(); // 创建一个临时缓冲区来存储帧
            
                    if let Err(e) = lua.context(|ctx| {
                        let frame_length: usize = ctx.globals().get("FRAME_LENGTH")?;
                        println!("用户填写的帧长度: {}", frame_length);
                
                        // 检查是否有完整的数据帧
                        while self.packet_buffer.len() >= frame_length {
                            // 提取完整的帧并存储到临时缓冲区
                            let frame = self.packet_buffer.drain(0..frame_length).collect::<Vec<u8>>();
                            frames_to_process.push(frame);
                        }
                        Ok::<(), rlua::Error>(())
                    }) {
                        println!("Lua脚本执行错误: {}", e);
                    }
                
                // 在闭包外处理帧
                for frame in frames_to_process {
                    self.process_frame(&frame);
                }
            }
        }
        // 缓冲区超过最大长度时清空（防止内存溢出）
        if self.packet_buffer.len() > 1024 {
            println!("缓冲区溢出，清空数据");
            self.packet_buffer.clear();
        }
    }

    fn process_frame(&mut self, frame: &[u8]) {
        if let Some(lua) = &self.lua_state {
            if let Err(e) = lua.context(|ctx| {
                let parse_fn = ctx.globals().get::<_, rlua::Function>("parse_waveform")?;
                let lua_data = ctx.create_table()?;
       
                for (i, &byte) in frame.iter().enumerate() {
                    lua_data.set(i + 1, byte)?;
                }
                
                let result: rlua::Result<Option<rlua::Table>> = parse_fn.call(lua_data);
                
                if let Ok(Some(result_table)) = result {
                    if let Ok(channel) = result_table.get::<_, Option<u8>>("channel") {
                        if let Some(channel) = channel {
                            let channel: usize = channel as usize;
                            
                            if channel <= 9 {
                                let points: rlua::Table = result_table.get("points")?;
                                
                                if channel < self.plot_data_per_channel.len() {
                                    for y in points.sequence_values::<f64>() {
                                        if let Ok(y_value) = y {
                                            let x_value = self.plot_data_per_channel[channel].len() as f64;
                                            println!("通道 {}: x={}, y={}", channel, x_value, y_value);
                                            self.plot_data_per_channel[channel].push((x_value, y_value as f64));
                                        }
                                    }
                                    
                                    // 保持数据点数量限制
                                    while self.plot_data_per_channel[channel].len() > 1000 {
                                        self.plot_data_per_channel[channel].remove(0);
                                    }
                                }
                            }
                        }
                    } else {
                        println!("通道号解析失败: 未返回有效的通道号");
                    }
                }
                Ok::<(), rlua::Error>(())
            }) {
                println!("Lua脚本执行错误: {}", e);
            }
        }
    }
}

impl eframe::App for SerialAssistant {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut data_to_process = Vec::new();
        
        // 处理TCP数据接收
        if self.tcp_connected {
            let mut buffer = [0; 1024];
            let mut should_disconnect = false;
            
            if let Some(tcp) = &self.tcp_stream {
                if let Ok(mut stream) = tcp.try_lock() {
                    match stream.read(&mut buffer) {
                        Ok(n) if n > 0 => {
                            data_to_process.extend_from_slice(&buffer[..n]);
                        },
                        Ok(_) => {
                            should_disconnect = true;
                        },
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // 非阻塞模式下没有数据可读
                        },
                        Err(_) => {
                            should_disconnect = true;
                        }
                    }
                }
            }
            
            if should_disconnect {
                self.disconnect_tcp();
            }
            
            if !data_to_process.is_empty() {
                self.process_received_data(&data_to_process);
            }
        }
        
        // 处理串口数据接收
        if !self.tcp_enabled {
            if let Ok(mut received) = self.received_data_shared.try_lock() {
                if !received.is_empty() {
                    data_to_process = received.chars()
                        .filter_map(|c| {
                            if c as u32 >= 0xFF00 {
                                Some((c as u32 - 0xFF00) as u8)
                            } else {
                                None
                            }
                        })
                        .collect();
                    received.clear();
                }
            }            
        }
        
        // 然后处理数据
        if !data_to_process.is_empty() {
            self.process_received_data(&data_to_process);
            
            // 更新缓冲区
            self.received_buffer.extend(&data_to_process);
            
            // 更新显示区域
            if self.is_hex_display {
                let mut hex_string = String::new();
                for &byte in data_to_process.iter() {
                    hex_string.push_str(&format!("{:02X} ", byte));
                }
                self.received_data.push_str(&hex_string);
                self.received_data.push('\n'); 
            } else {
                // 文本显示模式，支持汉字等 UTF-8 字符
                let text = String::from_utf8_lossy(&data_to_process);
                self.received_data.push_str(&text);
                self.received_data.push('\n');                
            }
            
            // 记录日志
            if self.log_enabled {
                // 弹出保存文件对话框
                if self.log_file.is_none() {
                    if let Some(file_path) = FileDialog::new()
                        .set_file_name("log.txt")
                        .save_file()
                    {
                        self.log_file = Some(file_path.to_string_lossy().into_owned());
                        println!("日志文件保存位置: {}", self.log_file.as_ref().unwrap());
                    }
                }
                self.log_data_with_lock(&data_to_process, true);
            } else {
                // 取消勾选时关闭日志文件
                self.log_file = None;
            }
            
            ctx.request_repaint();
        }

        // 自动发送逻辑
        if self.auto_send && self.auto_send_active && 
           self.last_send_time.elapsed().as_millis() as u64 >= self.auto_send_interval {
            
            // TCP 模式的自动发送
            if self.tcp_enabled && self.tcp_connected {
                if let Some(tcp) = &self.tcp_stream {
                    if let Ok(mut stream) = tcp.lock() {
                        let data = if self.is_hex_send {
                            utils::hex_to_bytes(&self.send_data)
                        } else {
                            self.send_data.as_bytes().to_vec()
                        };
                        
                        println!("TCP发送数据: {:?}", data);
                        if let Ok(written) = stream.write(&data) {
                            self.bytes_sent += written;
                            self.log_data_with_lock(&data, false);
                            println!("TCP实际发送字节数: {}", written);
                        }
                    }
                }
            }
            // 串口模式的自动发送
            else if let Some(port) = &self.port_handle {
                if let Ok(mut port) = port.lock() {
                    println!("原始输入: {}", self.send_data);
                    
                    let data = if self.is_hex_send {
                        utils::hex_to_bytes(&self.send_data)
                    } else {
                        self.send_data.as_bytes().to_vec()
                    };
                    
                    println!("发送数据: {:?}", data);
                    println!("发送数据(HEX): {}", data.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" "));
                    
                    if let Ok(written) = port.write(&data) {
                        self.bytes_sent += written;
                        self.log_data_with_lock(&data, false);
                        println!("实际发送字节数: {}", written);
                    }
                }
            }
            
            self.last_send_time = Instant::now();
            ctx.request_repaint_after(Duration::from_millis(self.auto_send_interval));
        }

        // 渲染UI
        ui::render_ui(self, ctx);
        ctx.request_repaint();
        // 删除这里重复的波形显示代码块
    }
}
