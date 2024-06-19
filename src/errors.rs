use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("Cannot find a router address.")]
    CantFindRouterAddress,
    #[error("Cannot find an interface.")]
    CantFindInterface,
    #[error("Cannot find a MAC address.")]
    CantFindMacAddress,
    #[error("Cannot create an Ethernet packet.")]
    CantCreateEthernetPacket,
    #[error("Cannot create an IPv4 packet.")]
    CantCreateIpv4Packet,
    #[error("Cannot create a TCP packet.")]
    CantCreateTcpPacket,
    #[error("Cannot create an ICMP packet.")]
    CantCreateIcmpPacket,
    #[error("This IP protocol version is not supported.")]
    UnsupportedIpVersion,
    #[error("Unexpected TCP flags set.")]
    UnexpectedTcpFlags,
    #[error("Unexpected ICMP response.")]
    UnexpectedIcmpResponse,
}

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Unexpected channel type")]
    UnexpectedChannelType,
    #[error("Send error")]
    SendError,
}
