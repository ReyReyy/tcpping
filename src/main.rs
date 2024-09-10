use reqwest::blocking::get;
use semver::Version;
use serde_json::Value;
use std::env;
use std::fs;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{self, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

fn get_latest_version() -> Option<(String, bool)> {
    let url = "https://api.github.com/repos/ReyReyy/tcpping/releases/";
    match get(url) {
        Ok(response) => {
            let json: Value = response.json().ok()?;
            let releases = json.as_array()?;
            let latest_release = releases.first()?;
            let latest_version = latest_release["tag_name"]
                .as_str()?
                .trim_start_matches('v')
                .to_string();
            let is_prerelease = latest_release["prerelease"].as_bool().unwrap_or(false);
            Some((latest_version, is_prerelease))
        }
        Err(_) => None,
    }
}

fn is_installed() -> bool {
    Path::new("/usr/local/bin/tcpping").exists()
}

fn install() -> Result<(), std::io::Error> {
    let current_exe = env::current_exe()?;
    fs::copy(current_exe, "/usr/local/bin/tcpping")?;
    Ok(())
}

fn tcp_ping(
    program_name: &str,
    host: &str,
    port: u16,
    count: Option<u32>,
    force_ip_version: Option<IpAddr>,
) {
    let addr = format!("{}:{}", host, port);
    let socket_addrs: Vec<_> = match addr.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(_) => {
            eprintln!("{}: {}: Name or service not known", program_name, host);
            return;
        }
    };

    if socket_addrs.is_empty() {
        eprintln!("{}: {}: Name or service not known", program_name, host);
        return;
    }

    let ip = match force_ip_version {
        Some(IpAddr::V4(_)) => socket_addrs.iter().find(|addr| addr.is_ipv4()),
        Some(IpAddr::V6(_)) => socket_addrs.iter().find(|addr| addr.is_ipv6()),
        None => Some(socket_addrs.first().unwrap()),
    };

    let ip = match ip {
        Some(ip) => ip,
        None => {
            eprintln!(
                "{}: {}: Address family for hostname not supported",
                program_name, host
            );
            return;
        }
    };

    // Check if network connection is available
    if let Err(_) = TcpStream::connect_timeout(ip, Duration::from_millis(100)) {
        eprintln!("{}: connect: Network is unreachable", program_name);
        return;
    }

    let is_ipv6 = ip.is_ipv6();
    let ip_str = if is_ipv6 {
        format!("[{}]", ip.ip())
    } else {
        ip.ip().to_string()
    };

    // Check if host is an IP address
    let host_is_ip = host.parse::<IpAddr>().is_ok();

    if host_is_ip {
        println!("TCP PING {}:{}", ip_str, port);
    } else {
        println!("TCP PING {} {}:{}", host, ip_str, port);
    }

    let mut delays = Vec::new();
    let mut packets_sent = 0;
    let mut packets_received = 0;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut seq = 0;
    while running.load(Ordering::SeqCst) {
        if let Some(c) = count {
            if seq >= c {
                break;
            }
        }

        let start_time = Instant::now();
        match TcpStream::connect_timeout(ip, Duration::from_millis(1000)) {
            Ok(_) => {
                let delay = start_time.elapsed().as_secs_f64() * 1000.0;
                delays.push(delay);
                packets_sent += 1;
                packets_received += 1;
                println!(
                    "Connected to {}:{}, tcp_seq={} time={:.3} ms",
                    ip_str, port, seq, delay
                );
            }
            Err(e) => {
                packets_sent += 1;
                println!(
                    "Failed to connect to {}:{}, tcp_seq={} {}",
                    ip_str, port, seq, e
                );
            }
        }

        seq += 1;

        thread::sleep(Duration::from_secs(1));
    }

    // Show statistical information
    if !delays.is_empty() {
        let loss_rate = (packets_sent - packets_received) as f64 / packets_sent as f64 * 100.0;
        println!("\n--- {} tcp ping statistics ---", host);
        println!(
            "{} packets transmitted, {} packets received, {:.1}% packet loss",
            packets_sent, packets_received, loss_rate
        );

        let min = delays.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = delays.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = delays.iter().sum::<f64>() / delays.len() as f64;
        let variance =
            delays.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / delays.len() as f64;
        let stddev = variance.sqrt();

        println!(
            "round-trip min/avg/max/stddev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            min, mean, max, stddev
        );
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    match args.get(1).map(|s| s.as_str()) {
        Some("-v") | Some("--version") => {
            let current_version = env!("CARGO_PKG_VERSION");
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            println!("tcpping version {} ({}/{})", current_version, os, arch);

            if let Some((latest_version, is_prerelease)) = get_latest_version() {
                let current_is_prerelease = current_version.contains("-");
                if Version::parse(&latest_version).unwrap()
                    > Version::parse(current_version).unwrap()
                    || (current_is_prerelease && !is_prerelease)
                {
                    println!(
                        "New version available: v{}. run 'tcpping --upgrade' to update",
                        latest_version
                    );
                }
            }
            process::exit(0);
        }
        Some("-h") | Some("--help") => {
            eprintln!(
                "Usage: {} <host> [--port <port>] [--IPv4 | --IPv6] [--count <count>] [--upgrade]",
                program_name
            );
            process::exit(0);
        }
        Some("--upgrade") => {
            if !is_installed() {
                eprintln!(
                    "tcpping not installed. Please run '{} --install' first.",
                    program_name
                );
                process::exit(1);
            }

            let current_version = env!("CARGO_PKG_VERSION");
            if let Some((latest_version, is_prerelease)) = get_latest_version() {
                let current_is_prerelease = current_version.contains("-");
                if Version::parse(&latest_version).unwrap()
                    <= Version::parse(current_version).unwrap()
                    && !(current_is_prerelease && !is_prerelease)
                {
                    println!("This version is already the latest version.");
                    process::exit(0);
                }
            }

            println!("Upgrading tcpping...");

            // Create a temporary file
            let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
            let temp_path = temp_file.path().to_str().unwrap().to_string();

            // Download the install script
            let response =
                get("https://raw.githubusercontent.com/ReyReyy/tcpping/master/shell/install.sh")
                    .expect("Faild to download install script");
            let content = response.text().expect("Failed to read install script");

            // Write the script content to the temporary file
            fs::write(&temp_path, content).expect("Failed to write to temporary file");

            // Run the install script with the --upgrade parameter
            let status = Command::new("sudo")
                .arg("bash")
                .arg(&temp_path)
                .arg("--upgrade")
                .status()
                .expect("Failed to execute install script");

            process::exit(status.code().unwrap_or(1));
        }
        Some("--install") => {
            if is_installed() {
                eprintln!("tcpping already installed. No need to install again.");
                process::exit(1);
            }

            println!("Installing tcpping...");
            match install() {
                Ok(_) => println!("tcpping has been installed successfully"),
                Err(e) => {
                    eprintln!("Installation failed: {}", e);
                    process::exit(1);
                }
            }
            process::exit(0);
        }
        _ => {}
    }

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <host> [--port <port>] [--IPv4 | --IPv6] [--count <count>]",
            program_name
        );
        process::exit(1);
    }

    let host = &args[1];
    let mut port = 80;
    let mut count = None;
    let mut force_ip_version = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                port = args[i + 1].parse().unwrap_or(80);
                i += 2;
            }
            "-c" | "--count" => {
                count = Some(args[i + 1].parse().unwrap_or(0));
                i += 2;
            }
            "-4" | "--ipv4" => {
                force_ip_version = Some(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
                i += 1;
            }
            "-6" | "--ipv6" => {
                force_ip_version = Some(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED));
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                process::exit(1);
            }
        }
    }

    tcp_ping(&program_name, host, port, count, force_ip_version);
}
