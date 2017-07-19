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

struct OSCCodec;

impl UdpCodec for OSCCodec {
    type In = (SocketAddr, OscMsg);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let mut msg = bytes::BytesMut::with_capacity(4 + buf.len());
        // Should probably guard against overflow...
        msg.put_u32::<BigEndian>(buf.len() as u32);
        msg.extend_from_slice(&buf[..]);
        serde_osc::from_slice(&msg[..])
            .map(|osc_msg| (*addr, osc_msg))
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
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
    let (sock_sink, mut sock_stream) = socket.framed(OSCCodec).split();

    let sock_stream = sock_stream.boxed().for_each(|(addr, msg)| {
        match msg {
            OscMsg::Freq((), (new_freq,)) => {
                println!("new_freq: {:?}", &new_freq);
            }
            _ => { 
                println!("message: {:?}", msg); 
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
