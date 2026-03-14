use std::net::Ipv4Addr;

use libc::{AF_INET, AF_PACKET, IPPROTO_ICMP, SOCK_RAW, in_addr, sendto, sockaddr_in, socket};

pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP_TIME_EXCEEDED: u8 = 11;
pub const ICMP_ID: u8 = 1;

pub struct EthernetHeader {
    pub src_mac: [u8; 6],
    pub dest_mac: [u8; 6],
    pub ether_type: u16
}

pub fn create_socket() -> Result<i32, Box<dyn std::error::Error>> {
    let sock = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if sock < 0 {
        return Err("Failed to create raw socket (run with sudo)".into());
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
    };
    Ok(sock)
}

pub fn create_sniffing_socket() -> Result<i32, Box<dyn std::error::Error>> {
    let sock = unsafe { socket(AF_PACKET, SOCK_RAW, (0x0003u16).to_be() as i32) };
    if sock < 0 {
        return Err("Failed to create raw socket (run with sudo)".into());
    }
    Ok(sock)
}

pub fn set_ttl(sock: i32, ttl: u8) -> Result<(), Box<dyn std::error::Error>> {
    let ttl_val: libc::c_int = ttl as libc::c_int;
    let res = unsafe {
        libc::setsockopt(
            sock, 
            libc::IPPROTO_IP, 
            libc::IP_TTL, 
            &ttl_val as *const _ as *const _, 
            std::mem::size_of::<libc::c_int>() as u32
        )
    };
    if res < 0 {
        return Err("Failed to set TTL".into());
    }
    Ok(())
}

pub fn create_dest(target_ip: Ipv4Addr) -> sockaddr_in {
    sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: 0,
        sin_addr: in_addr {
            s_addr: u32::from(target_ip).to_be(),
        },
        sin_zero: [0; 8],
    }
}

pub fn send_packet(sock: i32, icmp_packet: &[u8], dest: &sockaddr_in) -> Result<isize, Box<dyn std::error::Error>> {
    let send_result = unsafe {
        sendto(
            sock,
            icmp_packet.as_ptr() as *const _,
            icmp_packet.len(),
            0,
            dest as *const _ as *const _,
            std::mem::size_of::<sockaddr_in>() as u32,
        )
    };

    if send_result < 0 {
        return Err("sendto failed".into());
    }

    Ok(send_result)
}

pub fn calculate_checksum(buf: &[u8]) -> u16 {
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