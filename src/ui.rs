use crate::app::SerialAssistant;
use eframe::egui;
use std::time::Duration;
use egui_plot::{Line, Plot, PlotPoints, Legend};  // 添加 Legend 导入
use std::io::Write;  // 添加导入
use rfd::FileDialog;  // 添加导入

pub fn render_ui(app: &mut SerialAssistant, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let available_size = ui.available_size();
        
        ui.vertical(|ui| {
            // 顶部控制区域
            render_top_controls(app, ui, available_size);

            // 内容区域
            ui.vertical(|ui| {
                let available_height = ui.available_height();
                let available_width = ui.available_width();
                
                // 修改 Grid 布局设置
                egui::Grid::new("main_content_grid")  // 修改为更具体的ID
                    .num_columns(2)
                    .spacing([10.0, 0.0])
                    .min_row_height(available_height)
                    .show(ui, |ui| {
                        // 发送区域 (左边)
                        ui.group(|ui| {
                            ui.set_max_width(available_width * 0.47);
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                ui.set_min_height(available_height - 20.0);
                                render_send_area(app, ui, ctx, available_width, available_height);
                            });
                        });
                        
                        // 接收区域 (右边)
                        ui.group(|ui| {
                            ui.set_max_width(available_width * 0.47);
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                                ui.set_min_height(available_height - 20.0);
                                render_receive_area(app, ui, ctx, available_width, available_height);
                            });
                        });
                    });
            });
        });
    });

    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 更新状态信息
            app.update_status();
            app.update_transfer_rate();
            ui.label(&app.status_message);
        });
    });

    // 波形显示窗口
    if app.plot_visible {
        egui::Window::new("波形显示")
            .id(egui::Id::new("serial_wave_window"))
            .default_size([600.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                // 计算所有通道的数据范围
                let mut min_y = f64::INFINITY;
                let mut max_y = f64::NEG_INFINITY;
                
                for plot_data in &app.plot_data_per_channel {
                    if !plot_data.is_empty() {
                        let (_, local_min) = plot_data.iter()
                            .map(|(_, y)| *y)
                            .fold((0.0, f64::INFINITY), |acc, y| (y, acc.1.min(y)));
                        let (_, local_max) = plot_data.iter()
                            .map(|(_, y)| *y)
                            .fold((0.0, f64::NEG_INFINITY), |acc, y| (y, acc.1.max(y)));
                        
                        min_y = min_y.min(local_min);
                        max_y = max_y.max(local_max);
                    }
                }
                
                // 添加边距，使显示更美观
                let range = if max_y > min_y { max_y - min_y } else { 1.0 };
                let margin = range * 0.1;
                let y_min = min_y - margin;
                let y_max = max_y + margin;

                Plot::new("serial_wave_plot")
                    .id(egui::Id::new("serial_wave_plot_area"))
                    .view_aspect(2.0)
                    .include_y(y_min)  // 使用计算出的范围
                    .include_y(y_max)  // 使用计算出的范围
                    .show_axes([true, true])
                    .show_grid([true, true])
                    .legend(Legend::default())
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        for (channel, plot_data) in app.plot_data_per_channel.iter().enumerate() {
                            if !plot_data.is_empty() {
                                let points: Vec<[f64; 2]> = plot_data
                                    .iter()
                                    .map(|(x, y)| [*x, *y])
                                    .collect();
                                
                                // 根据通道号选择不同的颜色
                                let color = match channel {
                                    0 => egui::Color32::from_rgb(255, 0, 0),   // 红色
                                    1 => egui::Color32::from_rgb(0, 255, 0),   // 绿色
                                    2 => egui::Color32::from_rgb(0, 0, 255),   // 蓝色
                                    3 => egui::Color32::from_rgb(255, 255, 0), // 黄色
                                    4 => egui::Color32::from_rgb(255, 0, 255), // 紫色
                                    5 => egui::Color32::from_rgb(0, 255, 255), // 青色
                                    6 => egui::Color32::from_rgb(128, 0, 128), // 深紫色
                                    7 => egui::Color32::from_rgb(128, 128, 0), // 橄榄色
                                    8 => egui::Color32::from_rgb(0, 128, 128), // 深青色
                                    9 => egui::Color32::from_rgb(128, 128, 128), // 灰色
                                    _ => egui::Color32::from_rgb(100, 200, 100), // 默认颜色
                                };
                                
                                let line = Line::new(PlotPoints::from_iter(points))
                                    .color(color)
                                    .name(format!("通道 {}", channel))
                                    .width(2.0);
                                plot_ui.line(line);
                            }
                        }
                    });
            });
    }

    // 帮助窗口
    if app.show_help {
        let mut show = true;
        egui::Window::new("协议说明")
            .id(egui::Id::new("protocol_help_window"))
            .default_size([400.0, 300.0])
            .resizable(true)
            .collapsible(true)
            .open(&mut show)
            .show(ctx, |ui| {
                ui.heading("当前波形协议说明");
                ui.add_space(8.0);
                ui.label("1. 数据格式: ");
                ui.label("   - 协议:AA + 数据长度(通道号 + 数据长度) + 通道号(0-9,大于9不绘制曲线)+ 4字节数据 + 4字节数据 + ...");
                ui.label("   - 包含固定协议头AA,数据长度,通道号和数据值");
                ui.label("   - 协议中FRAME_LENGT是整帧长度,是数据长度加2");
                ui.label("   - 每个数据点的字节数BYTES_PER_POINT固定4字节");
                ui.label("   - 数据类型根据实际情况DATA_TYPE可选: 'int' 或 'float'");
                ui.add_space(8.0);
                ui.label("2. 数据示例: ");
                ui.label("   - (DATA_TYPE = int 通道1,数值:305419896, -305419896) : AA 09 01 12 34 56 78 ED CB A9 88  // 长度9 = 1(通道号) + 8(两组数据)");
                ui.label("   - (DATA_TYPE = float 通道1,数值:10.0, -10.0) : AA 09 03 00 00 20 41 00 00 20 C1  // 长度9 = 1(通道号) + 8(两组数据)");
                ui.add_space(8.0);
                ui.label("3. 通道说明: ");
                ui.label("   - 支持多达 10 个通道");
                ui.label("   - 每个通道独立显示不同颜色");
                ui.add_space(8.0);
                ui.label("4. 显示控制: ");
                ui.label("   - 支持缩放和拖动查看历史数据");
                ui.label("   - 右键选中放大,左键双击还原,点击曲线图例显示和隐藏");
                ui.label("5. 自定义协议: ");
                ui.label("   - 编辑waveform.lua文件以自定义波形协议,满足返回通道数和数据即可,数据可以是整型或浮点型");
            });
        if !show {
            app.show_help = false;
        }
    }
}

fn render_top_controls(app: &mut SerialAssistant, ui: &mut egui::Ui, available_size: egui::Vec2) {
    ui.horizontal_wrapped(|ui| {
        ui.set_min_width(available_size.x);
        
        // 添加TCP/串口切换
        ui.horizontal(|ui| {
            ui.radio_value(&mut app.tcp_enabled, false, "串口模式");
            ui.radio_value(&mut app.tcp_enabled, true, "TCP模式");
        });
        
        if app.tcp_enabled {
            // TCP模式的控件
            ui.horizontal_wrapped(|ui| {
                ui.label("IP地址:");
                ui.add(egui::TextEdit::singleline(&mut app.tcp_address).desired_width(120.0));
                ui.label("端口:");
                ui.add(egui::TextEdit::singleline(&mut app.tcp_port).desired_width(60.0));
                
                if !app.tcp_connected {
                    if ui.button("连接").clicked() {
                        app.connect_tcp();
                    }
                } else {
                    if ui.button("断开").clicked() {
                        app.disconnect_tcp();
                    }
                }
            });
        } else {
            // 串口控件 - 第一行
            ui.horizontal_wrapped(|ui| {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("串口")
                            .selected_text(&app.selected_port)
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for port in &app.ports {
                                    ui.selectable_value(
                                        &mut app.selected_port,
                                        port.port_name.clone(),
                                        &port.port_name,
                                    );
                                }
                            });

                        // 添加打开/关闭串口按钮
                        if app.port_handle.is_none() {
                            if ui.button("打开串口").clicked() && !app.selected_port.is_empty() {
                                app.open_port();
                            }
                        } else {
                            if ui.button("关闭串口").clicked() {
                                app.close_port();
                            }
                        }
                        
                        if ui.button("刷新").clicked() {
                            app.ports = serialport::available_ports().unwrap_or_default();
                        }
                    });

                    ui.horizontal(|ui| {
                          
                        egui::ComboBox::from_label("波特率")
                            .selected_text(if app.selected_baud == 0 {
                                "自定义...".to_string()
                            } else {
                                app.selected_baud.to_string()
                            })
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.selected_baud, 0, "自定义...");
                                for &rate in &app.baud_rates {
                                    ui.selectable_value(
                                        &mut app.selected_baud,
                                        rate,
                                        rate.to_string(),
                                    );
                                }
                            });

                        if app.selected_baud == 0 {
                            ui.add(
                                egui::TextEdit::singleline(&mut app.custom_baud_text)
                                    .desired_width(80.0)
                                    .hint_text("输入波特率")
                            );
                            
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if let Ok(baud) = app.custom_baud_text.parse::<u32>() {
                                    if baud > 0 {
                                        app.selected_baud = baud;
                                    }
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("数据位")
                            .selected_text(format!("{:?}", app.data_bits))
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.data_bits, serialport::DataBits::Five, "5");
                                ui.selectable_value(&mut app.data_bits, serialport::DataBits::Six, "6");
                                ui.selectable_value(&mut app.data_bits, serialport::DataBits::Seven, "7");
                                ui.selectable_value(&mut app.data_bits, serialport::DataBits::Eight, "8");
                            });
                    });
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("停止位")
                            .selected_text(format!("{:?}", app.stop_bits))
                            .width(60.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.stop_bits, serialport::StopBits::One, "1");
                                ui.selectable_value(&mut app.stop_bits, serialport::StopBits::Two, "2");
                            });

                        egui::ComboBox::from_label("校验位")
                            .selected_text(format!("{:?}", app.parity))
                            .width(80.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut app.parity, serialport::Parity::None, "None");
                                ui.selectable_value(&mut app.parity, serialport::Parity::Even, "Even");
                                ui.selectable_value(&mut app.parity, serialport::Parity::Odd, "Odd");
                            });
                    });
                });
        }
        if ui.button("帮助").clicked() {
            app.show_help = true;  // 点击按钮时设置状态为 true
        }
    });
}

fn render_send_area(app: &mut SerialAssistant, ui: &mut egui::Ui, ctx: &egui::Context, available_width: f32, available_height: f32) {
    ui.group(|ui| {
        ui.set_max_width(available_width * 0.47);
        ui.set_min_width(0.0);
        ui.set_min_height(available_height - 20.0);
        ui.vertical(|ui| {
            // 顶部控制区域
            ui.horizontal(|ui| {
                ui.label("发送区域");
                ui.checkbox(&mut app.is_hex_send, "HEX发送");
                
                // 添加对自动发送状态的处理
                let auto_send_before = app.auto_send;
                ui.checkbox(&mut app.auto_send, "自动发送");
                // 如果取消了自动发送，也要重置激活状态
                if auto_send_before && !app.auto_send {
                    app.auto_send_active = false;
                }
                
                if app.auto_send {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut app.auto_send_interval)
                            .speed(100)
                            .clamp_range(1..=600000)
                            .prefix("间隔: ")
                            .suffix("ms"));
                    });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("已发送: {} 字节", app.bytes_sent));
                });
            });
            
            // 添加滚动区域
            egui::ScrollArea::vertical()
                .id_source("send_area_scroll")  // 为发送区域添加唯一的ID
                .max_height(available_height - 100.0)
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut app.send_data)
                        .desired_width(f32::INFINITY)
                        .desired_rows(15);
                    ui.add_sized([ui.available_width(), ui.available_height()], text_edit);
                });
            
            // 底部按钮区域
            ui.horizontal(|ui| {
                if ui.button("发送").clicked() {
                    let data = if app.is_hex_send {
                        // 将输入按空格分割，解析每个十六进制数
                        let hex_values: Vec<u8> = app.send_data
                            .split_whitespace()
                            .filter_map(|s| u8::from_str_radix(s, 16).ok())
                            .collect();
                        hex_values
                    } else {
                        app.send_data.as_bytes().to_vec()
                    };
                
                    if app.tcp_enabled {
                        // TCP 发送
                        if let Some(tcp) = &app.tcp_stream {
                            if let Ok(mut stream) = tcp.try_lock() {
                                if let Ok(written) = stream.write(&data) {
                                    app.bytes_sent += written;
                                    app.log_data_with_lock(&data, false);
                                }
                            }
                        }
                    } else {
                        // 串口发送
                        if let Some(port) = &app.port_handle {
                            if let Ok(mut port) = port.lock() {
                                if let Ok(written) = port.write(&data) {
                                    app.bytes_sent += written;
                                    app.log_data_with_lock(&data, false);
                                }
                            }
                        }
                    }
                
                    // 重置计时器，激活自动发送
                    app.last_send_time = std::time::Instant::now();
                    
                    // 如果启用了自动发送，则激活它并请求重绘以便继续发送
                    if app.auto_send {
                        app.auto_send_active = true;
                        ctx.request_repaint_after(Duration::from_millis(app.auto_send_interval));
                    }
                }

                if ui.button("清空发送").clicked() {
                    app.send_data.clear();
                    app.bytes_sent = 0;
                }
            });
        });
    });
}

fn render_receive_area(app: &mut SerialAssistant, ui: &mut egui::Ui, _ctx: &egui::Context, available_width: f32, available_height: f32) {
    ui.group(|ui| {
        ui.set_max_width(available_width * 0.47);
        ui.set_min_width(0.0);
        ui.set_min_height(available_height - 20.0);
        ui.vertical(|ui| {
            // 顶部控制区域
            ui.horizontal(|ui| {
                ui.label("接收区域");
                ui.checkbox(&mut app.is_hex_display, "HEX显示");
                ui.checkbox(&mut app.plot_visible, "波形显示");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("已接收: {} 字节", app.bytes_received));
                });
            });
            
            // 接收数据显示区域
            egui::ScrollArea::vertical()
                .id_source("receive_area_scroll")  // 为接收区域添加唯一的ID
                .max_height(available_height - 100.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add_sized(
                        [ui.available_width(), ui.available_height()],
                        egui::TextEdit::multiline(&mut app.received_data)
                            .desired_width(f32::INFINITY)
                            .desired_rows(15)
                            .lock_focus(true)
                    );
                });
            
            // 底部按钮区域
            ui.horizontal(|ui| {
                if ui.button("清空接收").clicked() {
                    app.received_data.clear();
                    app.bytes_received = 0;
                }
                
                // 处理日志记录复选框
                let log_enabled_before = app.log_enabled;
                ui.checkbox(&mut app.log_enabled, "记录日志");
                if app.log_enabled && !log_enabled_before {
                    // 弹出保存文件对话框
                    if let Some(file_path) = FileDialog::new()
                        .set_file_name("log.txt")
                        .save_file()
                    {
                        app.log_file = Some(file_path.to_string_lossy().into_owned());
                        println!("日志文件保存位置: {}", app.log_file.as_ref().unwrap());
                    } else {
                        // 如果没有选择有效的文件路径，则取消选中复选框
                        app.log_enabled = false;
                    }
                } else if !app.log_enabled {
                    // 取消勾选时关闭日志文件
                    app.log_file = None;
                }
            });

            // 添加显示日志文件路径的标签
            if let Some(ref log_file) = app.log_file {
                ui.label(format!("日志文件保存位置: {}", log_file));
            }
        });
    });
}


