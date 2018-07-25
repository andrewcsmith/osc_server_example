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

use osc_address::{OscMessage, OscBundle, OscPacket};

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::error::Error;

use bytes::{Buf, BytesMut, BufMut};
use futures::{Future, BoxFuture, Sink, Stream};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use tokio_io::codec::{Encoder, Decoder};
use tokio_service::Service;

#[derive(OscMessage, Debug, PartialEq)]
enum OscMsg {
    #[osc_address(address="freq")]
    Freq((), (f32,)),

    #[osc_address(address="error")]
    Error((), (String,)),
}

struct OSCCodec;

impl UdpCodec for OSCCodec {
    type In = (SocketAddr, OscBundle<OscMsg>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        let mut msg = bytes::BytesMut::with_capacity(4 + buf.len());
        // Should probably guard against overflow...
        msg.put_i32_be(buf.len() as i32);
        msg.extend_from_slice(&buf[..]);
        println!("msg: {:?}", msg);
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

    let sock_stream = sock_stream
    .or_else::<_, Result<_, io::Error>>(|err| {
        // TODO: We need a way to bundle these errors back into an OscBundle and pass them along
        // println!("error: {:?}", err);
        // Ok((addr.clone(), OscMsg::Error((), (err.description().to_string(),))))
        Err(err)
    })
    .for_each(|(addr, bundle)| {
        for packet in bundle.messages() {
            match packet {
                OscPacket::Message(msg) => {
                    match msg {
                        OscMsg::Freq((), (new_freq,)) => {
                            println!("new_freq: {:?}", &new_freq);
                        }
                        OscMsg::Error((), (err,)) => {
                            println!("error: {}", &err);
                        }
                        _ => { 
                            println!("message: {:?}", &msg); 
                        }
                    }
                }

                OscPacket::Bundle(_) => println!("Sorry, we'll only nest one deep here")
            }
        }

        Ok(())
    });

    core.run(sock_stream);

    Ok(())
}

fn main() {
    go().unwrap();
}
