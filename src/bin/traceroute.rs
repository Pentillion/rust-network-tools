use std::{env};
use std::net::Ipv4Addr;
use std::time::{Instant};
use libc::{sockaddr_in, recvfrom};
use rust_network_tools::{ICMP_ECHO_REPLY, ICMP_ECHO_REQUEST, ICMP_ID, calculate_checksum, create_dest, create_socket, send_packet, set_ttl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: traceroute <target-ip>");
        return Ok(());
    }

    let target_ip: Ipv4Addr = args[1].parse()?;

    let sock = create_socket()?;

    let dest = create_dest(target_ip);

    println!("traceroute to {}, 30 hops max", target_ip);

    for ttl in 1..=30 {
        set_ttl(sock, ttl)?;
        let mut icmp_packet: [u8; 8] = [ICMP_ECHO_REQUEST, 0, 0, 0, 0, ICMP_ID, 0, ttl];
        let checksum = calculate_checksum(&icmp_packet);
        icmp_packet[2..4].copy_from_slice(&checksum.to_be_bytes());

        let request_start = Instant::now();

        send_packet(sock, &icmp_packet, &dest)?;

        let mut buf = [0u8; 1024];

        let mut src_addr: sockaddr_in = unsafe { std::mem::zeroed() };
        let mut addr_len = std::mem::size_of::<sockaddr_in>() as u32;

        loop {
            unsafe {
                recvfrom(
                    sock,
                    buf.as_mut_ptr() as *mut _,
                    buf.len(),
                    0,
                    &mut src_addr as *mut _ as *mut _,
                    &mut addr_len as *mut _,
                )
            };

            let recv_type: u8 = buf[20];
            let recv_id = u16::from_be_bytes([buf[24], buf[25]]);

            let elapsed = request_start.elapsed();
            let reply_ip = Ipv4Addr::from(u32::from_be(src_addr.sin_addr.s_addr));
            println!(" {:2}  {}  {:.3?}", ttl, reply_ip, elapsed);

            if recv_id == ICMP_ID as u16 && recv_type == ICMP_ECHO_REPLY {
                return Ok(());
            }
            break;
        }       
    }

    return Ok(());
}