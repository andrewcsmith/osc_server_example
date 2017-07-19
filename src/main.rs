//! Try running this code and interacting with it from SuperCollider:
//!
//! ```
//! ~addr = NetAddr.new("127.0.0.1", 6667);
//! ~addr.sendMsg("/freq", 440);
//! ~addr.sendMsg("/freq", 441.1);
//! ```
//!

extern crate serde;
extern crate serde_osc;
#[macro_use] extern crate serde_derive;

extern crate bytes;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

extern crate osc_address;
#[macro_use] extern crate osc_address_derive;

use osc_address::OscMessage;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::error::Error;

use bytes::{BytesMut, BufMut, BigEndian, LittleEndian};
use futures::{Future, BoxFuture, Sink, Stream};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use tokio_io::codec::{Encoder, Decoder};
use tokio_service::Service;

#[derive(OscMessage, Debug, PartialEq)]
enum OscMsg {
    #[osc_address(address="freq")]
    Freq((), (f32,)),
}

struct LineCodec;

impl UdpCodec for LineCodec {
    type In = (SocketAddr, Vec<u8>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        into.extend(buf);
        addr
    }
}

fn go() -> Result<(), Box<Error>> {
    let mut core = Core::new()?;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6667);
    let socket = UdpSocket::bind(&addr, &core.handle())?;
    println!("socket: {:?}", &socket.local_addr());
    let (sock_sink, mut sock_stream) = socket.framed(LineCodec).split();

    let sock_stream = sock_stream.boxed().for_each(|(addr, msg)| {
        // println!("len: {:?}", &msg.len());
        // println!("msg: {:?}", &msg);
        let mut buf_len = bytes::BytesMut::with_capacity(4);
        buf_len.put_u32::<BigEndian>(msg.len() as u32);
        let mut new_msg: Vec<u8> = buf_len.to_vec();
        new_msg.extend_from_slice(&msg[..]);
        // println!("new_msg: {:?}", &new_msg);
        let message: Result<OscMsg, _> = serde_osc::from_slice(&new_msg[..]); 
        match message {
            Ok(message) => {
                match message {
                    OscMsg::Freq((), (new_freq,)) => {
                        println!("new_freq: {:?}", &new_freq);
                    }
                    _ => { println!("message: {:?}", message); }
                }
            }
            Err(e) => { 
                println!("wtf: {:?}", e);
            }
        }
        Ok(())
    });

    let fut = sock_stream;

    core.run(fut);

    Ok(())
}

fn main() {
    go().unwrap();
}
