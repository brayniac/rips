use {Payload, Tx, TxResult};

use pnet::packet::MutablePacket;
use pnet::packet::ethernet::{EtherType, EthernetPacket, MutableEthernetPacket};
use pnet::util::MacAddr;

use std::cmp;

/// Trait for anything wishing to be the payload of an Ethernet frame.
pub trait EthernetPayload: Payload {
    fn ether_type(&self) -> EtherType;
}


/// Basic reference implementation of an `EthernetPayload`.
/// Can be used to construct Ethernet frames with arbitrary payload from a
/// vector.
pub struct BasicEthernetPayload {
    ether_type: EtherType,
    offset: usize,
    payload: Vec<u8>,
}

impl BasicEthernetPayload {
    pub fn new(ether_type: EtherType, payload: Vec<u8>) -> Self {
        BasicEthernetPayload {
            ether_type: ether_type,
            offset: 0,
            payload: payload,
        }
    }
}

impl EthernetPayload for BasicEthernetPayload {
    fn ether_type(&self) -> EtherType {
        self.ether_type
    }
}

impl Payload for BasicEthernetPayload {
    fn len(&self) -> usize {
        self.payload.len()
    }

    fn build(&mut self, buffer: &mut [u8]) {
        let start = self.offset;
        let end = cmp::min(start + buffer.len(), self.payload.len());
        self.offset = end;
        buffer[0..end - start].copy_from_slice(&self.payload[start..end]);
    }
}

#[cfg(test)]
mod basic_ethernet_payload_tests {
    use Payload;
    use pnet::packet::ethernet::EtherTypes;
    use super::*;

    #[test]
    fn ether_type() {
        let testee = BasicEthernetPayload::new(EtherTypes::Ipv6, vec![]);
        assert_eq!(EtherTypes::Ipv6, testee.ether_type());
    }

    #[test]
    fn len_zero() {
        let testee = BasicEthernetPayload::new(EtherTypes::Arp, vec![]);
        assert_eq!(0, testee.len());
    }

    #[test]
    fn len_three() {
        let testee = BasicEthernetPayload::new(EtherTypes::Arp, vec![5, 6, 7]);
        assert_eq!(3, testee.len());
    }

    #[test]
    fn build_without_data() {
        let mut testee = BasicEthernetPayload::new(EtherTypes::Arp, vec![]);
        let mut buffer = vec![99; 1];
        testee.build(&mut buffer);
        assert_eq!(99, buffer[0]);
    }

    #[test]
    fn build_with_data() {
        let mut testee = BasicEthernetPayload::new(EtherTypes::Arp, vec![5, 6, 7]);
        let mut buffer = vec![0; 1];
        testee.build(&mut buffer[0..0]);

        testee.build(&mut buffer);
        assert_eq!(5, buffer[0]);
        testee.build(&mut buffer);
        assert_eq!(6, buffer[0]);
        testee.build(&mut buffer);
        assert_eq!(7, buffer[0]);

        testee.build(&mut buffer[0..0]);
    }

    #[test]
    fn build_with_larger_buffer() {
        let mut testee = BasicEthernetPayload::new(EtherTypes::Arp, vec![5, 6]);
        let mut buffer = vec![0; 3];
        testee.build(&mut buffer);
        assert_eq!(&[5, 6, 0], &buffer[..]);
    }
}


pub trait EthernetTx {
    fn src(&self) -> MacAddr;
    fn dst(&self) -> MacAddr;
    fn send<P>(&mut self, packets: usize, size: usize, payload: P) -> TxResult
        where P: EthernetPayload;
}

pub struct EthernetTxImpl<T: Tx> {
    src: MacAddr,
    dst: MacAddr,
    tx: T,
}

impl<T: Tx> EthernetTxImpl<T> {
    pub fn new(tx: T, src: MacAddr, dst: MacAddr) -> Self {
        EthernetTxImpl {
            src: src,
            dst: dst,
            tx: tx,
        }
    }
}

impl<T: Tx> EthernetTx for EthernetTxImpl<T> {
    fn src(&self) -> MacAddr {
        self.src
    }

    fn dst(&self) -> MacAddr {
        self.dst
    }

    /// Send ethernet packets to the network.
    ///
    /// For every packet, all `header_size+size` bytes will be sent, no
    /// matter how small payload is provided to the `MutableEthernetPacket` in
    /// the call to `builder`. So in total `packets * (header_size+size)` bytes
    /// will be sent. This is  usually not a problem since the IP layer has the
    /// length in the header and the extra bytes should thus not cause any
    /// trouble.
    fn send<P>(&mut self, packets: usize, size: usize, payload: P) -> TxResult
        where P: EthernetPayload
    {
        let builder = EthernetBuilder::new(self.src, self.dst, payload);
        let total_size = size + EthernetPacket::minimum_packet_size();
        self.tx.send(packets, total_size, builder)
    }
}

/// Struct building Ethernet frames
pub struct EthernetBuilder<P: EthernetPayload> {
    src: MacAddr,
    dst: MacAddr,
    payload: P,
}

impl<P: EthernetPayload> EthernetBuilder<P> {
    /// Creates a new `EthernetBuilder` with the given parameters
    pub fn new(src: MacAddr, dst: MacAddr, payload: P) -> Self {
        EthernetBuilder {
            src: src,
            dst: dst,
            payload: payload,
        }
    }
}

impl<P: EthernetPayload> Payload for EthernetBuilder<P> {
    fn len(&self) -> usize {
        EthernetPacket::minimum_packet_size() + self.payload.len()
    }

    /// Modifies `pkg` to have the correct header and payload
    fn build(&mut self, buffer: &mut [u8]) {
        let mut pkg = MutableEthernetPacket::new(buffer).unwrap();
        pkg.set_source(self.src);
        pkg.set_destination(self.dst);
        pkg.set_ethertype(self.payload.ether_type());
        self.payload.build(pkg.payload_mut());
    }
}
