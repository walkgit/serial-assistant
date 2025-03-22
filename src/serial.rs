use serialport::SerialPort;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Default)]
pub struct SharedData {
    pub packet_buffer: Vec<u8>,
    pub plot_data: Vec<(f64, f64)>,
}

pub struct SerialPortHandle {
    pub(crate) port: Arc<Mutex<Box<dyn SerialPort>>>,
    pub(crate) running: Arc<Mutex<bool>>,
}

impl SerialPortHandle {
    pub fn try_lock(&self) -> Result<impl std::ops::DerefMut<Target = Box<dyn SerialPort>>, std::sync::TryLockError<impl std::ops::DerefMut<Target = Box<dyn SerialPort>>>> {
        self.port.try_lock()
    }

    pub fn lock(&self) -> Result<impl std::ops::DerefMut<Target = Box<dyn SerialPort>>, std::sync::PoisonError<impl std::ops::DerefMut<Target = Box<dyn SerialPort>>>> {
        self.port.lock()
    }
}

pub fn open_port(
    port_name: &str,
    baud_rate: u32,
    data_bits: serialport::DataBits,
    stop_bits: serialport::StopBits,
    parity: serialport::Parity,
    received_data_shared: Arc<Mutex<String>>,
) -> Option<SerialPortHandle> {
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
        
        let running = Arc::new(Mutex::new(true));
        let running_clone = Arc::clone(&running);
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            
            while *running_clone.lock().unwrap() {
                match port_clone.lock() {
                    Ok(mut port) => {
                        match port.read(&mut buf) {
                            Ok(bytes_read) if bytes_read > 0 => {
                                if let Ok(mut received) = received_data_clone.lock() {
                                    for byte in &buf[..bytes_read] {
                                        received.push(char::from_u32(0xFF00 + *byte as u32).unwrap_or('?'));
                                    }
                                    received.push('\n');
                                }
                            },
                            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {},
                            Err(_) => {
                                thread::sleep(Duration::from_millis(50));
                            },
                            _ => {}
                        }
                    },
                    Err(_) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                thread::sleep(Duration::from_millis(5));
            }
            println!("串口读取线程已退出");
        });
        
        Some(SerialPortHandle {
            port: port_handle,
            running,
        })
    } else {
        None
    }
}

pub fn close_port(handle: SerialPortHandle) {
    // 首先停止读取线程
    if let Ok(mut running) = handle.running.lock() {
        *running = false;
    }
    
    // 等待线程完全退出
    thread::sleep(Duration::from_millis(100));
    
    // 清理并关闭串口
    if let Ok(mut port) = handle.port.lock() {
        let _ = port.clear(serialport::ClearBuffer::All);
        let _ = port.flush();
        drop(port);  // 显式释放串口资源
    }
}
