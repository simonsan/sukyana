use super::osi_layers::{Layer, NetworkLayer, TransportLayer};
use anyhow::Result;
use log::debug;
use pnet::packet::{
    self,
    ip::IpNextHeaderProtocols,
    ipv4::{self, Ipv4Flags, Ipv4Packet, MutableIpv4Packet},
    tcp::{MutableTcpPacket, TcpFlags},
};
use rand::Rng;
use std::{net::Ipv4Addr, time::Duration};

const IPV4_HEADER_SIZE: usize = 20;
const TCP_HEADER_SIZE: usize = 20;
const TCP_DATA_SIZE: usize = 0;
const TTL: u8 = 64;

pub struct Tcp;

impl Tcp {
    /// Constructs IPv4 and TCP headers for a TCP SYN packet.
    ///
    /// Sets:
    /// - Source IP address.
    /// - Source port.
    /// - Destination IP address.
    /// - Destination port.
    ///
    /// Returns a byte array containing the IPv4 and TCP headers.
    pub fn build_syn_packet(
        src_ip: Ipv4Addr,
        src_port: u16,
        dest_ip: Ipv4Addr,
        dest_port: u16,
    ) -> [u8; IPV4_HEADER_SIZE + TCP_HEADER_SIZE + TCP_DATA_SIZE] {
        let mut rng = rand::thread_rng();
        let mut tcp_packet = [0u8; IPV4_HEADER_SIZE + TCP_HEADER_SIZE + TCP_DATA_SIZE];

        let mut ip_header = MutableIpv4Packet::new(&mut tcp_packet).unwrap();
        ip_header.set_version(4);
        ip_header.set_header_length(5);
        ip_header.set_source(src_ip);
        ip_header.set_destination(dest_ip);
        ip_header.set_total_length((IPV4_HEADER_SIZE + TCP_HEADER_SIZE + TCP_DATA_SIZE) as u16);
        ip_header.set_identification(rng.gen());
        ip_header.set_flags(Ipv4Flags::DontFragment);
        ip_header.set_ttl(TTL);
        ip_header.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
        let ip_checksum = ipv4::checksum(&ip_header.to_immutable());
        ip_header.set_checksum(ip_checksum);

        let mut tcp_header = MutableTcpPacket::new(&mut tcp_packet[IPV4_HEADER_SIZE..]).unwrap();
        tcp_header.set_source(src_port);
        tcp_header.set_destination(dest_port);
        tcp_header.set_sequence(rng.gen());
        tcp_header.set_acknowledgement(rng.gen());
        tcp_header.set_reserved(0);
        tcp_header.set_flags(TcpFlags::SYN);
        tcp_header.set_urgent_ptr(0);
        tcp_header.set_window(1024);
        tcp_header.set_data_offset(5);
        let tcp_checksum =
            packet::tcp::ipv4_checksum(&tcp_header.to_immutable(), &src_ip, &dest_ip);
        tcp_header.set_checksum(tcp_checksum);

        tcp_packet
    }

    /// Sends a TCP SYN packet and parses the response.
    ///
    /// The packet is handed over to the network layer.
    pub fn send_syn_packet(
        value: u8,
        src_ip: Ipv4Addr,
        src_port: u16,
        dest_ip: Ipv4Addr,
        dest_port: u16,
    ) -> Result<(Option<Vec<u8>>, Option<Duration>)> {
        // Build the TCP SYN packet.
        let packet = Tcp::build_syn_packet(src_ip, src_port, dest_ip, dest_port);

        // Create the match data for layer 3.
        let network_layer = NetworkLayer {
            datalink_layer: None,
            src_addr: Some(dest_ip.into()),
            dest_addr: Some(src_ip.into()),
        };

        // Create the match data for layer 4.
        let transport_layer = TransportLayer {
            network_layer: Some(network_layer),
            src_port: Some(dest_port),
            dest_port: Some(src_port),
        };

        // Matches from layer 4 to layer 2.
        let layer = Layer::Four(transport_layer);

        // Send the packet over the network layer.
        // The packet is handed over to the network layer.
        let (response, rtt) =
            NetworkLayer::send_and_receive(src_ip, dest_ip, &packet, layer, value)?;

        // Parse the IPv4 response.
        match response {
            Some(packet) => {
                match Ipv4Packet::new(&packet) {
                    Some(ip_packet) => {
                        debug!("TCP response: {:?}", ip_packet);
                        // TODO: Parse the TCP response.
                    }
                    None => debug!("No TCP response."),
                }
                Ok((Some(packet), rtt))
            }
            None => Ok((None, None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pnet::packet::tcp::TcpPacket;

    #[test]
    fn test_build_syn_packet() {
        let src_ip = Ipv4Addr::new(192, 168, 1, 1);
        let src_port = 12345;
        let dest_ip = Ipv4Addr::new(192, 168, 1, 2);
        let dest_port = 80;

        // Build a SYN packet.
        let packet = Tcp::build_syn_packet(src_ip, src_port, dest_ip, dest_port);

        // Create the IP packet.
        let ip_packet = Ipv4Packet::new(&packet).unwrap();

        // Verify the IP packet.
        assert_eq!(ip_packet.get_version(), 4);
        assert_eq!(ip_packet.get_source(), src_ip);
        assert_eq!(ip_packet.get_destination(), dest_ip);
        assert_eq!(
            ip_packet.get_next_level_protocol(),
            IpNextHeaderProtocols::Tcp
        );

        // Create the TCP packet.
        let tcp_packet = TcpPacket::new(&packet[IPV4_HEADER_SIZE..]).unwrap();

        // Verify the TCP packlet.
        assert_eq!(tcp_packet.get_source(), src_port);
        assert_eq!(tcp_packet.get_destination(), dest_port);
        assert_eq!(tcp_packet.get_flags(), TcpFlags::SYN);
    }

    #[test]
    fn test_send_syn_packet() -> Result<()> {
        // Local IP address.
        let src_ip = Ipv4Addr::new(192, 168, 178, 26);
        let src_port = 12345;

        // Google's IP address.
        let dest_ip = Ipv4Addr::new(142, 251, 209, 131);
        let dest_port = 80;

        // Send a SYN packet. Calls subsequently the network and data link layer.
        let (packet, rtt) = Tcp::send_syn_packet(1, src_ip, src_port, dest_ip, dest_port)?;

        // Ensure we have received a response packet.
        assert!(packet.is_some());

        // Ensure we have received a round-trip time.
        assert!(rtt.is_some());

        Ok(())
    }
}