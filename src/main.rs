use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use clap::Parser;
use image::Luma;
use qrcode::{Color, QrCode};

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

#[derive(Debug, Parser)]
#[command(version, about = "生成 Android 无线调试配对二维码")]
struct Cli {
    /// 自定义配对服务名称（ASCII 字母、数字和连字符）
    #[arg(long)]
    service_name: Option<String>,
    /// 自定义配对密码（ASCII 字母和数字）
    #[arg(long)]
    password: Option<String>,
    /// 将二维码同时保存为 PNG 文件
    #[arg(long)]
    png: Option<PathBuf>,
    /// 只输出二维码内容，适合脚本使用
    #[arg(long, conflicts_with_all = ["png", "pair"])]
    payload_only: bool,
    /// 扫码后通过 adb 自动发现设备并配对
    #[arg(long)]
    pair: bool,
    /// 自动配对等待时间（秒）
    #[arg(long, default_value_t = 120, requires = "pair")]
    timeout: u64,
}

fn random_string(length: usize) -> Result<String, String> {
    let mut result = String::with_capacity(length);
    while result.len() < length {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes).map_err(|e| format!("无法生成安全随机数: {e}"))?;
        for byte in bytes {
            // 丢弃超出 62 的最大整倍数的值，避免取模偏差。
            if byte < 248 {
                result.push(ALPHABET[usize::from(byte) % ALPHABET.len()] as char);
                if result.len() == length {
                    break;
                }
            }
        }
    }
    Ok(result)
}

fn validate(name: &str, value: &str, max: usize, hyphen: bool) -> Result<(), String> {
    if value.is_empty()
        || value.len() > max
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || (hyphen && b == b'-'))
    {
        return Err(format!(
            "{name} 必须为 1..={max} 个 ASCII 字母或数字{}",
            if hyphen {
                "（可包含连字符）"
            } else {
                ""
            }
        ));
    }
    Ok(())
}

fn pairing_payload(service: &str, password: &str) -> String {
    format!("WIFI:T:ADB;S:{service};P:{password};;")
}

fn render_terminal(qr: &QrCode) -> String {
    let width = qr.width();
    let colors = qr.to_colors();
    let mut output = String::new();
    const MARGIN: usize = 1;
    // 用背景色填满整个字符格，避免块字符在有行间距的字体中产生白线。
    for y in 0..width + 2 * MARGIN {
        let mut previous = None;
        for x in 0..width + 2 * MARGIN {
            let dark = (MARGIN..width + MARGIN).contains(&x)
                && (MARGIN..width + MARGIN).contains(&y)
                && colors[(y - MARGIN) * width + x - MARGIN] == Color::Dark;
            if previous != Some(dark) {
                output.push_str(if dark { "\x1b[40m" } else { "\x1b[47m" });
                previous = Some(dark);
            }
            output.push_str("  ");
        }
        output.push_str("\x1b[0m\n");
    }
    output
}

fn find_service<'a>(output: &'a str, service: &str) -> Option<&'a str> {
    output.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        match (fields.next(), fields.next(), fields.next()) {
            (Some(name), Some("_adb-tls-pairing._tcp"), Some(address)) if name == service => {
                Some(address)
            }
            _ => None,
        }
    })
}

fn pair(service: &str, password: &str, timeout: u64) -> Result<(), String> {
    eprintln!("等待手机扫码并公布配对服务（最多 {timeout} 秒）…");
    let start = Instant::now();
    loop {
        let output = Command::new("adb")
            .args(["mdns", "services"])
            .output()
            .map_err(|e| {
                format!("无法启动 adb，请安装 Android SDK Platform-Tools 并将 adb 加入 PATH: {e}")
            })?;
        if !output.status.success() {
            return Err(format!(
                "adb mdns services 失败: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        if let Some(address) = find_service(&text, service) {
            let output = Command::new("adb")
                .args(["pair", address, password])
                .output()
                .map_err(|e| format!("无法执行 adb pair: {e}"))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !output.status.success() || !stdout.contains("Successfully paired") {
                return Err(format!("配对失败: {} {}", stdout.trim(), stderr.trim()));
            }
            println!("{}", stdout.trim());
            println!(
                "配对成功。如设备未自动连接，请在无线调试页面查看 IP 和调试端口，执行 adb connect IP:端口。"
            );
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(timeout) {
            return Err(
                "等待配对服务超时；请确认手机和电脑处于同一网络，且手机正在扫描无线调试二维码"
                    .into(),
            );
        }
        std::thread::sleep(
            Duration::from_secs(1)
                .min(Duration::from_secs(timeout).saturating_sub(start.elapsed())),
        );
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let service = match cli.service_name {
        Some(s) => s,
        None => format!("studio-{}", random_string(10)?),
    };
    let password = match cli.password {
        Some(s) => s,
        None => random_string(16)?,
    };
    validate("服务名称", &service, 63, true)?;
    validate("配对密码", &password, 64, false)?;
    let payload = pairing_payload(&service, &password);
    if cli.payload_only {
        println!("{payload}");
        return Ok(());
    }
    let qr = QrCode::new(payload.as_bytes()).map_err(|e| format!("生成二维码失败: {e}"))?;
    if let Some(path) = cli.png {
        let image = qr
            .render::<Luma<u8>>()
            .module_dimensions(8, 8)
            .quiet_zone(true)
            .build();
        image
            .save_with_format(&path, image::ImageFormat::Png)
            .map_err(|e| format!("无法保存 {}: {e}", path.display()))?;
        eprintln!("二维码已保存至 {}", path.display());
    }
    print!("{}", render_terminal(&qr));
    println!("在手机的「开发者选项 → 无线调试 → 使用二维码配对设备」中扫码。");
    if cli.pair {
        pair(&service, &password, cli.timeout)?;
    } else {
        println!("服务名称: {service}\n配对密码: {password}");
        println!(
            "扫码后执行 adb mdns services 查找上述服务，再执行 adb pair 地址:端口 配对密码；也可使用 --pair 自动配对。"
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn payload_and_validation() {
        assert_eq!(
            pairing_payload("studio-test", "abc123"),
            "WIFI:T:ADB;S:studio-test;P:abc123;;"
        );
        assert!(validate("服务", "studio-test", 63, true).is_ok());
        assert!(validate("服务", "test;P:bad", 63, true).is_err());
        assert!(validate("密码", "", 64, false).is_err());
    }
    #[test]
    fn selects_only_matching_pairing_service() {
        let services = "List of discovered mdns services\nstudio-test _adb-tls-connect._tcp 192.0.2.1:1234\nstudio-other _adb-tls-pairing._tcp 192.0.2.2:1234\nstudio-test _adb-tls-pairing._tcp [::1]:4567\n";
        assert_eq!(find_service(services, "studio-test"), Some("[::1]:4567"));
        assert_eq!(find_service(services, "missing"), None);
    }
    #[test]
    fn random_values_have_expected_shape() {
        let value = random_string(128).unwrap();
        assert_eq!(value.len(), 128);
        assert!(value.bytes().all(|b| ALPHABET.contains(&b)));
    }
}
