use std::env;
use std::net::Ipv4Addr;
use std::time::Instant;
use libc::{socket, AF_INET, SOCK_RAW, IPPROTO_ICMP, sockaddr_in, in_addr, sendto, recvfrom};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: ping_tool <target-ip>");
        return;
    }

    let target_ip: Ipv4Addr = args[1].parse().expect("Invalid IP address");
    println!("Pinging {} ...", target_ip);

    let sock = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if sock < 0 {
        panic!("Failed to create raw socket (run with sudo)");
    }

    let icmp_packet: [u8; 8] = [
        8, 0, 0, 0,
        0, 1,
        0, 1
    ];

    let dest = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: 0,
        sin_addr: in_addr {
            s_addr: u32::from(target_ip).to_be(),
        },
        sin_zero: [0; 8],
    };

    let start = Instant::now();

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

    let mut buf = [0u8; 1024];

    let mut src_addr: sockaddr_in = unsafe { std::mem::zeroed() };
    let mut addr_len = std::mem::size_of::<sockaddr_in>() as u32;

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

    if recv_len > 0 {
        let elapsed = start.elapsed();

        let reply_ip =
            Ipv4Addr::from(u32::from_be(src_addr.sin_addr.s_addr));

        println!(
            "Received {} bytes from {} in {:?}",
            recv_len, reply_ip, elapsed
        );
    } else {
        println!("No reply received");
    }
}