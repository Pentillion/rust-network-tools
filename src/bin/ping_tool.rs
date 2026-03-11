use std::{env, thread};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use libc::{sockaddr_in, recvfrom};
use rust_network_tools::{calculate_checksum, create_dest, create_socket, send_packet};

const ICMP_ECHO_REQUEST: u8 = 8;
const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_ID: u8 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ping_tool <target-ip>");
        return Ok(());
    }

    let target_ip: Ipv4Addr = args[1].parse()?;

    let sock = create_socket()?;

    let dest = create_dest(target_ip);

    println!("PING {}", target_ip);

    let session_start = Instant::now();
    let mut rtts: Vec<Duration> = Vec::new();

    let mut total_packets_transmitted: u8 = 0;
    let mut total_packets_received: u8 = 0;

    for sequence_id in 1..=3 {
        let mut icmp_packet: [u8; 8] = [ICMP_ECHO_REQUEST, 0, 0, 0, 0, ICMP_ID, 0, sequence_id];
        let checksum = calculate_checksum(&icmp_packet);
        icmp_packet[2..4].copy_from_slice(&checksum.to_be_bytes());

        let request_start = Instant::now();

        send_packet(sock, &icmp_packet, &dest)?;

        total_packets_transmitted += 1;

        let mut buf = [0u8; 1024];

        let mut src_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut addr_len = std::mem::size_of::<sockaddr_in>() as u32;

        loop {
            let recv_len = unsafe {
                recvfrom(
                    sock,
                    buf.as_mut_ptr() as *mut _,
                    buf.len(),
                    0,
                    &mut src_addr as *mut _ as *mut _,
                    &mut addr_len as *mut _,
                )
            };

            if recv_len < 0 {
                println!("Request timed out.");
                break;
            }

            let recv_type: u8 = buf[20];
            let recv_id = u16::from_be_bytes([buf[24], buf[25]]);
            let ttl: u8 = buf[8];

            if recv_id == ICMP_ID as u16 && recv_type == ICMP_ECHO_REPLY {
                let elapsed = request_start.elapsed();
                rtts.push(elapsed);
                total_packets_received += 1;
                let reply_ip = Ipv4Addr::from(u32::from_be(src_addr.sin_addr.s_addr));
                println!("{} bytes from {}: icmp_seq={} ttl={} time={:?}", recv_len, reply_ip, sequence_id, ttl, elapsed);
                break;
            }
        }       
        thread::sleep(Duration::from_secs(1));
    }

    let packet_loss = (1.0 - (total_packets_received as f64 / total_packets_transmitted as f64)) * 100.0;

    println!("--- {} ping statistics ---", target_ip);
    println!("{} packets transmitted, {} packets received, {}% packet loss, time {}ms", 
        total_packets_transmitted, 
        total_packets_received, 
        packet_loss, 
        session_start.elapsed().as_millis()
    );

    if !rtts.is_empty() {
        let min = rtts.iter().min().unwrap();
        let max = rtts.iter().max().unwrap();
        let avg = rtts.iter().sum::<Duration>() / rtts.len() as u32;

        println!(
            "rtt min/avg/max = {:.3}/{:.3}/{:.3} ms", 
            min.as_secs_f64() * 1000.0, 
            avg.as_secs_f64() * 1000.0, 
            max.as_secs_f64() * 1000.0
        );
    }

    return Ok(());
}