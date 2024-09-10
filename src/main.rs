use semver::Version;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::Write;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{self, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn get_latest_version(
    include_prerelease: bool,
) -> Result<(String, bool), Box<dyn std::error::Error>> {
    let url = "https://api.github.com/repos/ReyReyy/tcpping/releases";
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "tcpping-updater")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Faild to request API {}", response.status()).into());
    }

    let releases: Vec<Value> = response.json()?;
    if releases.is_empty() {
        return Err("Unable to find any versions".into());
    }

    let latest_release = releases
        .into_iter()
        .filter(|release| include_prerelease || !release["prerelease"].as_bool().unwrap_or(false))
        .max_by(|a, b| {
            let a_version = Version::parse(
                a["tag_name"]
                    .as_str()
                    .unwrap_or("0.0.0")
                    .trim_start_matches('v'),
            )
            .unwrap_or_else(|_| Version::new(0, 0, 0));
            let b_version = Version::parse(
                b["tag_name"]
                    .as_str()
                    .unwrap_or("0.0.0")
                    .trim_start_matches('v'),
            )
            .unwrap_or_else(|_| Version::new(0, 0, 0));
            a_version.cmp(&b_version)
        })
        .ok_or("Unable to find the fitting version")?;

    let latest_version = latest_release["tag_name"]
        .as_str()
        .ok_or("Unable to resolve version tags")?
        .trim_start_matches('v')
        .to_string();
    let is_prerelease = latest_release["prerelease"].as_bool().unwrap_or(false);

    Ok((latest_version, is_prerelease))
}

fn is_installed() -> bool {
    Path::new("/usr/local/bin/tcpping").exists()
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

fn perform_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Downloading installation script...");

    let install_script_url =
        "https://raw.githubusercontent.com/ReyReyy/tcpping/master/shell/install.sh";
    let response = reqwest::blocking::get(install_script_url)?;
    let script_content = response.text()?;

    let temp_script_path = "/tmp/tcpping_install.sh";
    let mut file = File::create(temp_script_path)?;
    file.write_all(script_content.as_bytes())?;

    // println!("Running installation script...");

    let status = Command::new("sudo")
        .arg("bash")
        .arg(temp_script_path)
        .arg("--upgrade")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err("Faild to upgrade".into())
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

            let current_is_prerelease = current_version.contains("-");
            match get_latest_version(current_is_prerelease) {
                Ok((latest_version, is_prerelease)) => {
                    // println!("Latest version: {}", latest_version);
                    if let (Ok(current), Ok(latest)) = (
                        Version::parse(current_version),
                        Version::parse(&latest_version),
                    ) {
                        if latest > current || (current_is_prerelease && !is_prerelease) {
                            println!(
                                "New version: {}. Run '{} --upgrade' to upgrade",
                                latest_version, program_name
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Unable to get latest version: {}", e);
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
            // println!("Current version: {}", current_version);

            let current_is_prerelease = current_version.contains("-");
            match get_latest_version(current_is_prerelease) {
                Ok((latest_version, is_prerelease)) => {
                    // println!("Latest version: {}", latest_version);

                    if let (Ok(current), Ok(latest)) = (
                        Version::parse(current_version),
                        Version::parse(&latest_version),
                    ) {
                        if latest > current || (current_is_prerelease && !is_prerelease) {
                            println!("New version: {}, upgrading...", latest_version);
                            match perform_upgrade() {
                                Ok(_) => {
                                    // println!("Upgrade completed");
                                    process::exit(0);
                                }
                                Err(e) => {
                                    eprintln!("Faild to upgrade: {}", e);
                                    process::exit(1);
                                }
                            }
                        } else {
                            println!("Current version is the latest.");
                            process::exit(0);
                        }
                    } else {
                        eprintln!("Version parsing error.");
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Unable to get latest version: {}.", e);
                    println!("Continue upgrade? (y/N)");
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read input");
                    if input.trim().to_lowercase() != "y" {
                        println!("Upgrade cancelled.");
                        process::exit(0);
                    }
                }
            }
        }
        Some("--install") => {
            if is_installed() {
                eprintln!("tcpping is already installed. No need to install again.");
                process::exit(1);
            }

            println!("Installing tcpping...");
            let status = Command::new("sudo")
                .arg("cp")
                .arg(env::current_exe().expect("Unable to get current executable path"))
                .arg("/usr/local/bin/tcpping")
                .status();

            match status {
                Ok(exit_status) if exit_status.success() => {
                    println!("tcpping installed successfully");
                    process::exit(0);
                }
                Ok(_) => {
                    eprintln!("Faild to install tcpping");
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("An error occurred while installing tcpping: {}", e);
                    process::exit(1);
                }
            }
        }
        Some("--uninstall") => {
            if !is_installed() {
                eprintln!("tcpping is not installed. No need to uninstall.");
                process::exit(1);
            }

            println!("Uninstalling tcpping...");
            match Command::new("sudo")
                .arg("rm")
                .arg("-f")
                .arg("/usr/local/bin/tcpping")
                .status()
            {
                Ok(status) if status.success() => {
                    println!("tcpping uninstalled successfully");
                    process::exit(0);
                }
                Ok(_) => {
                    eprintln!("Faild to uninstall tcpping");
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("An error occurred while uninstalling tcpping: {}", e);
                    process::exit(1);
                }
            }
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
