use bincode;
use std::net::{SocketAddr, UdpSocket};
use crate::network;
use std::io::Result;

#[derive(Debug)]
pub struct Sender {
    pub socket: UdpSocket,
    pub host_addr: SocketAddr,
    pub peer_addrs: Vec<SocketAddr>,
}

impl Sender {
    pub fn new(addr: SocketAddr, host_addr: SocketAddr) -> Result<Sender> {
        let socket = UdpSocket::bind(&addr)?;
        Ok(Sender {
            socket: socket,
            host_addr: host_addr,
            peer_addrs: Vec::new(),
        })
    }

    pub fn register_self(&self) -> Result<()> {
        let bytes = bincode::serialize(&network::NetworkEvent::PlayerJoin).unwrap();
        let interface_addr: SocketAddr = "0.0.0.0:9999".parse().unwrap();
        if interface_addr != self.host_addr {
            self.socket.send_to(&bytes, interface_addr)?;
        }
        self.socket.send_to(&bytes, self.host_addr)?;
        Ok(())
    }

    pub fn register_remote_socket(&mut self, addr: SocketAddr) -> Result<()> {
        let id = network::NetworkEvent::ID(self.peer_addrs.len());
        let id_bytes = bincode::serialize(&id).unwrap();
        self.socket.send_to(&id_bytes, addr)?;
        self.peer_addrs.push(addr);
        let peer_addrs_clone = self.peer_addrs.clone();
        let peer_addrs_bytes = bincode::serialize(&network::NetworkEvent::Peers(peer_addrs_clone)).unwrap();
        for peer_addr in self.peer_addrs.iter() {
            self.socket.send_to(&peer_addrs_bytes, peer_addr)?;
        }
        Ok(())
    }

    pub fn tick(&self, player_id: network::PlayerID) -> Result<()> {
        let bytes = bincode::serialize(&network::NetworkEvent::PlayerID(player_id)).unwrap();
        for peer_addr in self.peer_addrs.iter() {
            self.socket.send_to(&bytes, peer_addr)?;
        }
        Ok(())
    }
}
