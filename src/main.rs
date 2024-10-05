use clap;
use std::fs;
use std::io;
use std::process::Command;

// 通用读取文件函数
fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

// 读取电池电量
fn get_battery_capacity(battery_path: &str) -> Result<String, io::Error> {
    read_file(&(battery_path.to_string() + "capacity"))
}

// 读取充电状态
fn get_battery_status(battery_path: &str) -> Result<String, io::Error> {
    read_file(&(battery_path.to_string() + "status"))
}

// 打印帮助信息
fn print_help() {
    println!(
        "Usage: 
        --battery        Output battery status and capacity.
        --battery-state  Output battery status only.
        --battery-level  Output battery capacity only.
        --volume-level   Output volume level.
        --backlight      Output backlight"
    );
}

// 读取音量
// 使用 `amixer` 读取，依赖 `alsa-utils`
fn get_volume_level() -> Result<String, io::Error> {
    let output = Command::new("amixer").arg("get").arg("Master").output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        if line.contains("[off]") {
            return Ok("MUTED".to_string());
        }
        if line.contains("Mono:") || line.contains("Front Left:") {
            // 从形如 "[65%]" 的字符串中提取音量百分比
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find('%') {
                    let mut rst = "VOL: ".to_string();
                    rst.push_str(&line[start + 1..end + 1].to_string());
                    return Ok(rst);
                }
            }
        }
    }

    Ok("Unknown".to_string())
}

fn get_brightness() -> Result<String, io::Error> {
    let brightness_path = "/sys/class/backlight/amdgpu_bl1/brightness";
    let max_brightness_path = "/sys/class/backlight/amdgpu_bl1/max_brightness";

    let current_brightness = read_file(brightness_path)?;
    let current_brightness: i32 = current_brightness.parse().unwrap_or(0);

    let max_brightness = read_file(max_brightness_path)?;
    let max_brightness: i32 = max_brightness.parse().unwrap_or(1);

    let brightness_percentage = (current_brightness * 100) / max_brightness;

    Ok(format!("BL: {}%", brightness_percentage))
}

fn main() -> io::Result<()> {
    let battery_path = "/sys/class/power_supply/BAT0/";

    // 使用 clap 解析命令行参数
    let matches = clap::Command::new("Battery Info")
        .version("1.0")
        .about("Retrieve laptop battery status and level")
        .arg(
            clap::Arg::new("battery")
                .long("battery")
                .help("Output battery status and capacity")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("battery-state")
                .long("battery-state")
                .help("Output battery status only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("battery-capacity")
                .long("battery-capacity")
                .help("Output battery capacity only")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("volume-level")
                .long("volume-level")
                .help("Output volume level")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("backlight")
                .long("backlight")
                .help("Output backlight percentage")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // 根据不同参数输出信息
    if matches.get_flag("battery") {
        let capacity = get_battery_capacity(battery_path).unwrap_or_else(|e| {
            eprintln!("Error reading battery capacity: {}", e);
            "Unknown".to_string()
        });
        let status = get_battery_status(battery_path).unwrap_or_else(|e| {
            eprintln!("Error reading battery status: {}", e);
            "Unknown".to_string()
        });
        println!("{}: {}%", status, capacity);
    } else if matches.get_flag("battery-state") {
        let status = get_battery_status(battery_path).unwrap_or_else(|e| {
            eprintln!("Error reading battery status: {}", e);
            "Unknown".to_string()
        });
        println!("{}", status);
    } else if matches.get_flag("battery-capacity") {
        let capacity = get_battery_capacity(battery_path).unwrap_or_else(|e| {
            eprintln!("Error reading battery capacity: {}", e);
            "Unknown".to_string()
        });
        println!("{}%", capacity);
    } else if matches.get_flag("volume-level") {
        let volume_level = get_volume_level().unwrap_or_else(|e| {
            eprintln!("Error reading volume level: {}", e);
            "Unknown".to_string()
        });
        println!("{}", volume_level);
    } else if matches.get_flag("backlight") {
        let backlight_percentage = get_brightness().unwrap_or_else(|e| {
            eprintln!("Error reading backlight: {}", e);
            "Unknown".to_string()
        });
        println!("{}", backlight_percentage);
    } else {
        // 未指定参数时打印帮助信息
        print_help();
    }

    Ok(())
}
