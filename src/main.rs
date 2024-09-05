use std::env;
use std::net::{TcpStream, ToSocketAddrs};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn tcp_ping(host: &str, port: u16, use_ipv4: bool, count: Option<u32>) {
    let addr = format!("{}:{}", host, port);
    let socket_addrs: Vec<_> = match addr.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    if socket_addrs.is_empty() {
        eprintln!("{}", host);
        return;
    }

    let ip = if use_ipv4 {
        socket_addrs
            .iter()
            .find(|a| a.is_ipv4())
            .unwrap_or_else(|| {
                // eprintln!("Cannot find ipv4 address, use IPv6 instead");
                &socket_addrs[0]
            })
    } else {
        socket_addrs
            .iter()
            .find(|a| a.is_ipv6())
            .unwrap_or(&socket_addrs[0])
    };

    println!("TCP PING {} ({}):{}", host, ip.ip(), port);

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
                    "Connected to {}:{}: tcp_seq={} time={:.3} ms",
                    ip.ip(),
                    port,
                    seq,
                    delay
                );
            }
            Err(e) => {
                packets_sent += 1;
                println!(
                    "Failed to connect to {}:{}: tcp_seq={} {}",
                    ip.ip(),
                    port,
                    seq,
                    e
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
            println!("tcp_ping version 0.1.0");
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
    let mut use_ipv4 = false;
    let mut count = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                port = args[i + 1].parse().unwrap_or(80);
                i += 2;
            }
            "-4" | "--IPv4" => {
                use_ipv4 = true;
                i += 1;
            }
            "-6" | "--IPv6" => {
                use_ipv4 = false;
                i += 1;
            }
            "-c" | "--count" => {
                count = Some(args[i + 1].parse().unwrap_or(0));
                i += 2;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                process::exit(1);
            }
        }
    }

    tcp_ping(host, port, use_ipv4, count);
}
