use winres::WindowsResource;
use std::path::Path;
use std::env;

fn main() {
    println!("正在执行 build.rs...");
    
    if cfg!(target_os = "windows") {
        // 获取当前工作目录
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("无法获取项目目录");
        println!("项目目录: {}", manifest_dir);
        
        // 构建图标的绝对路径
        let icon_path = Path::new(&manifest_dir).join("assets").join("icon.ico");
        let icon_path_str = icon_path.to_str().expect("路径包含无效字符");
        
        println!("图标路径: {}", icon_path_str);
        
        // 检查文件是否存在
        if !icon_path.exists() {
            println!("错误: 图标文件不存在: {}", icon_path_str);
            panic!("找不到图标文件，构建中止");
        }
        
        println!("图标文件存在，继续编译资源");
        
        let mut res = WindowsResource::new();
        res.set_icon(icon_path_str);
        
        match res.compile() {
            Ok(_) => println!("资源编译成功"),
            Err(e) => panic!("资源编译失败: {}", e),
        }
    } else {
        println!("非 Windows 平台，跳过资源编译");
    }
}