#![feature(test)]

#[macro_use]
extern crate lazy_static;
extern crate pnet;
extern crate rips;
extern crate test;

use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ip::IpNextHeaderProtocols;
use rips::MacAddr;
use rips::Payload;
use rips::ethernet::{EthernetBuilder, BasicEthernetPayload};
use rips::ipv4::{Ipv4Builder, BasicIpv4Payload};
use rips::udp::UdpBuilder;
use std::net::{Ipv4Addr, SocketAddrV4};
use test::Bencher;

lazy_static! {
    static ref MAC: MacAddr = MacAddr::new(00, 11, 22, 33, 44, 55);
    static ref IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 3);

    static ref ETHERNET_PAYLOAD: BasicEthernetPayload<'static> = BasicEthernetPayload::new(EtherTypes::Arp, &[]);
    static ref IPV4_PAYLOAD: BasicIpv4Payload<'static> = BasicIpv4Payload::new(IpNextHeaderProtocols::Udp, &[]);
}

#[bench]
fn ethernet(b: &mut Bencher) {
    let mut buffer = vec![0; 1024];
    b.iter(|| {
        let mut builder = EthernetBuilder::new(*MAC, *MAC, (*ETHERNET_PAYLOAD).clone());
        builder.build(&mut buffer)
    });
}

#[bench]
fn ipv4(b: &mut Bencher) {
    let mut buffer = vec![0; 1024];
    b.iter(|| {
        let mut builder = Ipv4Builder::new(*IP, *IP, 0, (*IPV4_PAYLOAD).clone());
        builder.build(&mut buffer)
    });
}

#[bench]
fn udp(b: &mut Bencher) {
    let mut buffer = vec![0; 1024];
    let socket_addr = SocketAddrV4::new(*IP, 0);
    b.iter(|| {
        let mut builder = UdpBuilder::new(socket_addr, socket_addr, &[]);
        builder.build(&mut buffer)
    });
}
