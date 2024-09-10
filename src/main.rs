use std::env;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn tcp_ping(host: &str, port: u16, count: Option<u32>, force_ip_version: Option<IpAddr>) {
    let addr = format!("{}:{}", host, port);
    let socket_addrs: Vec<_> = match addr.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    if socket_addrs.is_empty() {
        eprintln!("tcpping: cannot resolve {}: Unknown host", host);
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
            eprintln!("tcpping: connect: Network is unreachable");
            return;
        }
    };

    // Check if network connection is available
    if let Err(e) = TcpStream::connect_timeout(ip, Duration::from_millis(100)) {
        match e.kind() {
            ErrorKind::ConnectionRefused | ErrorKind::AddrNotAvailable | ErrorKind::TimedOut => {
                eprintln!("tcpping: connect: Network is unreachable");
                return;
            }
            _ => {
                eprintln!("tcpping: connect: {}", e);
                return;
            }
        }
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
    match args.get(1).map(|s| s.as_str()) {
        Some("-v") | Some("--version") => {
            let version = env!("CARGO_PKG_VERSION");
            let arch = std::env::consts::ARCH;
            let os = std::env::consts::OS;
            println!("tcpping version {} ({}/{})", version, os, arch);
            process::exit(0);
        }
        Some("-h") | Some("--help") => {
            eprintln!(
                "Usage: {} <host> [--port <port>] [--IPv4 | --IPv6] [--count <count>]",
                args[0]
            );
            process::exit(0);
        }
        _ => {}
    }

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <host> [--port <port>] [--IPv4 | --IPv6] [--count <count>]",
            args[0]
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

    tcp_ping(host, port, count, force_ip_version);
}
