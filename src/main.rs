//  #![windows_subsystem = "windows"] // 添加此行以避免启动控制台

use walk_assistant::SerialAssistant;
use egui::{ViewportBuilder, FontDefinitions, FontData, FontFamily};
use eframe::{NativeOptions, Result};
use std::fs::read;
// use std::path::PathBuf;
// use std::sync::Arc; // 添加 Arc 导入

// 定义常量
const WINDOW_TITLE: &str = "串口助手";
const WINDOW_SIZE: [f32; 2] = [1000.0, 620.0];
const FONT_NAME: &str = "microsoft_yahei";
// const ICON_PATH: &str = "./assets/icon.png"; // 使用绝对路径

fn main() -> Result<()> {
    // 使用条件编译选择字体路径
    #[cfg(target_os = "windows")]
    const FONT_PATH: &str = "C:\\Windows\\Fonts\\msyh.ttc";
    #[cfg(target_os = "linux")]
    const FONT_PATH: &str = "/usr/share/fonts/truetype/msyh.ttc";
    #[cfg(target_os = "macos")]
    const FONT_PATH: &str = "/Library/Fonts/msyh.ttc";
    
    // 尝试加载图标
    // let icon_result = load_icon(ICON_PATH);
    
    // 创建 ViewportBuilder
    let viewport_builder = ViewportBuilder::default()
        .with_inner_size(WINDOW_SIZE)
        .with_decorations(true)
        .with_title(WINDOW_TITLE);
    
    // 如果图标加载成功，则设置图标
    // if let Ok(icon_data) = icon_result {
    //     viewport_builder = viewport_builder.with_icon(Arc::new(icon_data));
    // } else if let Err(e) = &icon_result {
    //     eprintln!("加载图标失败: {}", e);
    // }

    let native_options = NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_TITLE,
        native_options,
        Box::new(|cc| {
            // 设置字体
            let mut fonts = FontDefinitions::default();
            if let Ok(font_data) = read(FONT_PATH) {
                fonts.font_data.insert(
                    FONT_NAME.to_owned(),
                    FontData::from_owned(font_data),
                );
                fonts.families
                    .get_mut(&FontFamily::Proportional)
                    .unwrap()
                    .insert(0, FONT_NAME.to_owned());
                cc.egui_ctx.set_fonts(fonts);
            }
            
            Box::new(SerialAssistant::default())
        }),
    )
}

// 加载图标函数
// fn load_icon(path: &str) -> std::result::Result<egui::IconData, Box<dyn std::error::Error>> {
//     let icon_path = PathBuf::from(path);
//     println!("尝试加载图标: {:?}", icon_path); // 添加日志以便调试
    
//     let image = match image::open(icon_path) {
//         Ok(img) => img.into_rgba8(),
//         Err(e) => {
//             eprintln!("打开图片失败: {}", e);
//             return Err(Box::new(e));
//         }
//     };
    
//     let (width, height) = image.dimensions();
//     println!("图标尺寸: {}x{}", width, height); // 添加尺寸信息
    
//     let rgba = image.into_raw();
//     Ok(egui::IconData {
//         rgba,
//         width,
//         height,
//     })
// }