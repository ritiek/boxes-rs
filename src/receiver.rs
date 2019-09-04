use bincode;
use std::net::{SocketAddr, UdpSocket};
use std::io::Result;
use std::time;
use crate::network;

#[derive(Debug)]
pub struct Receiver {
    socket: UdpSocket,
}

impl Receiver {
    pub fn new(addr: SocketAddr) -> Result<Receiver> {
        let socket = UdpSocket::bind(&addr)?;
        Ok(Receiver {
            socket: socket,
        })
    }

    pub fn poll_event(&self) -> Result<network::NetworkData> {
        self.socket.set_read_timeout(None)?;

        let mut buf = [0; 300];
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        let event: network::NetworkEvent = bincode::deserialize(&buf).unwrap();
        Ok(network::NetworkData {
            amt: amt,
            src: src,
            event: event,
        })
    }

    pub fn peek_event(&self, duration: time::Duration) -> Result<network::NetworkData> {
        self.socket.set_read_timeout(Some(duration))?;

        let mut buf = [0; 300];
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        let event: network::NetworkEvent = bincode::deserialize(&buf).unwrap();
        Ok(network::NetworkData {
            amt: amt,
            src: src,
            event: event,
        })
    }
}
