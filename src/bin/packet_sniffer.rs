use libc::recv;
use rust_network_tools::{EthernetHeader, create_sniffing_socket};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sock = create_sniffing_socket()?;
    let mut buf = [0u8; 2048];

    println!("Listening for packets... (Press Ctrl+C to stop)");

    loop {
        let n = unsafe {
            recv(sock, buf.as_mut_ptr() as *mut _, buf.len(), 0)
        };

        if n > 0 {
            handle_packet(&buf[..n as usize]);
        }
    }
}

fn handle_packet(data: &[u8]) {
    if data.len() < 14 { return; }

    let eth = unsafe { &*(data.as_ptr() as *const EthernetHeader) };
    let proto = u16::from_be(eth.ether_type);

    println!("[ETH] Source: {:02x?} | Dest: {:02x?} | Proto: 0x{:04x}", 
             eth.src_mac, eth.dest_mac, proto);
}