use std::env;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn show_help(program_name: &str) {
    println!("  ");
    println!("Usage:");
    println!("  {} [options] <destination[:port]>", program_name);
    println!("  ");
    println!("Options:");
    println!("  <destination>        DNS name or IP address");
    println!("  -p <port>            specify port (override port in destination)");
    println!("  -4                   use IPv4");
    println!("  -6                   use IPv6");
    println!("  -c <count>           stop after <count> replies");
    println!("  -v                   show version");
    println!("  -h                   show help");
    println!("  ");
}

// Added: resolves destination address and port
fn parse_destination(dest: &str) -> (String, Option<u16>) {
    // Special cases for handling IPv6 addresses
    if dest.starts_with('[') {
        if let Some(end_bracket) = dest.find(']') {
            let ipv6 = &dest[1..end_bracket];
            if dest.len() > end_bracket + 1 && dest.chars().nth(end_bracket + 1) == Some(':') {
                if let Ok(port) = dest[end_bracket + 2..].parse::<u16>() {
                    return (ipv6.to_string(), Some(port));
                }
            }
            return (ipv6.to_string(), None);
        }
    }

    // Handling Common Addresses (IPv4 or Domain Names)
    if let Some(colon_idx) = dest.rfind(':') {
        if let Ok(port) = dest[colon_idx + 1..].parse::<u16>() {
            return (dest[..colon_idx].to_string(), Some(port));
        }
    }

    (dest.to_string(), None)
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

    let is_ipv6 = ip.is_ipv6();
    let ip_str = if is_ipv6 {
        format!("[{}]", ip.ip())
    } else {
        ip.ip().to_string()
    };

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

    if args.len() == 1 {
        show_help(&program_name);
        process::exit(0);
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("-v") | Some("--version") => {
            let current_version = env!("CARGO_PKG_VERSION");
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            println!("tcpping version {} ({}/{})", current_version, os, arch);
            process::exit(0);
        }
        Some("-h") | Some("--help") => {
            show_help(&program_name);
            process::exit(0);
        }
        _ => {}
    }

    // Parse the destination address and possible ports
    let (host, host_port) = parse_destination(&args[1]);
    let mut port = host_port.unwrap_or(80); // 80 is used by default
    let mut count = None;
    let mut force_ip_version = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    // -p parameter takes precedence over the port in the address
                    port = args[i + 1].parse().unwrap_or(port);
                    i += 2;
                } else {
                    eprintln!("Error: Port number required after -p");
                    process::exit(1);
                }
            }
            "-c" | "--count" => {
                if i + 1 < args.len() {
                    count = Some(args[i + 1].parse().unwrap_or(0));
                    i += 2;
                } else {
                    eprintln!("Error: Count number required after -c");
                    process::exit(1);
                }
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

    tcp_ping(&program_name, &host, port, count, force_ip_version);
}
