use std::{env, thread};
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use libc::{socket, AF_INET, SOCK_RAW, IPPROTO_ICMP, sockaddr_in, in_addr, sendto, recvfrom};

const ICMP_ECHO_REQUEST: u8 = 8;
const ICMP_ECHO_REPLY: u8 = 0;
const ICMP_ID: u8 = 1;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ping_tool <target-ip>");
        return;
    }

    let target_ip: Ipv4Addr = args[1].parse().expect("Invalid IP address");

    let sock = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if sock < 0 {
        panic!("Failed to create raw socket (run with sudo)");
    }

    let timeout = libc::timeval { tv_sec: 1, tv_usec: 0 };
    unsafe {
        libc::setsockopt(
            sock, 
            libc::SOL_SOCKET, 
            libc::SO_RCVTIMEO, 
            &timeout as *const _ as *const _, 
            std::mem::size_of::<libc::timeval>() as u32
        );
    }

    println!("PING {}", target_ip);

    let session_start = Instant::now();

    let mut total_packets_transmitted: u8 = 0;
    let mut total_packets_received: u8 = 0;

    let mut rtts: Vec<Duration> = Vec::new();

    let dest = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: 0,
            sin_addr: in_addr {
                s_addr: u32::from(target_ip).to_be(),
            },
            sin_zero: [0; 8],
        };

    for sequence_id in 1..=3 {
        let mut icmp_packet: [u8; 8] = [
            ICMP_ECHO_REQUEST, 0, 0, 0,
            0, ICMP_ID,
            0, sequence_id
        ];
        let checksum = calculate_checksum(&icmp_packet);
        icmp_packet[2] = (checksum >> 8) as u8;
        icmp_packet[3] = (checksum & 0xFF) as u8;

        let request_start = Instant::now();

        let send_result = unsafe {
            sendto(
                sock,
                icmp_packet.as_ptr() as *const _,
                icmp_packet.len(),
                0,
                &dest as *const _ as *const _,
                std::mem::size_of::<sockaddr_in>() as u32,
            )
        };

        if send_result < 0 {
            panic!("sendto failed");
        }

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
}

fn calculate_checksum(buf: &[u8]) -> u16 {
    let mut sum = 0u32;
    for i in (0..buf.len()).step_by(2) {
        let word = u16::from_be_bytes([buf[i], buf[i + 1]]);
        sum += word as u32;
    }
    while (sum >> 16) > 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !sum as u16
}