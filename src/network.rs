pub mod events {
    use serde_derive::{Serialize, Deserialize};
    use std::net::SocketAddr;
    use crate::game::{Direction, Point, Square};


    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum GameEvent {
        Quit,
        Direction(Direction),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerID {
        pub point: Point,
        pub player: Square,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum NetworkEvent {
        PlayerID(PlayerID),
        Peers(Vec<SocketAddr>),
        PlayerJoin,
        PlayerLeft,
        ID(usize),
    }

    #[derive(Debug)]
    pub struct NetworkData {
        pub amt: usize,
        pub src: SocketAddr,
        pub event: NetworkEvent,
    }
}

pub mod receiver {
    use std::time;
    use std::net::{SocketAddr, UdpSocket};
    use std::io::Result;
    use crate::network::events;


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

        pub fn poll_event(&self) -> Result<events::NetworkData> {
            self.socket.set_read_timeout(None)?;

            let mut buf = [0; 300];
            let (amt, src) = self.socket.recv_from(&mut buf)?;

            let event: events::NetworkEvent = bincode::deserialize(&buf).unwrap();
            Ok(events::NetworkData {
                amt: amt,
                src: src,
                event: event,
            })
        }

        pub fn peek_event(&self, duration: time::Duration) -> Result<events::NetworkData> {
            self.socket.set_read_timeout(Some(duration))?;

            let mut buf = [0; 300];
            let (amt, src) = self.socket.recv_from(&mut buf)?;

            let event: events::NetworkEvent = bincode::deserialize(&buf).unwrap();
            Ok(events::NetworkData {
                amt: amt,
                src: src,
                event: event,
            })
        }
    }
}


pub mod sender {
    use std::net::{SocketAddr, UdpSocket};
    use std::io::Result;
    use crate::network::events;


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
            let bytes = bincode::serialize(&events::NetworkEvent::PlayerJoin).unwrap();
            let interface_addr: SocketAddr = "0.0.0.0:9999".parse().unwrap();
            if interface_addr != self.host_addr {
                self.socket.send_to(&bytes, interface_addr)?;
            }
            self.socket.send_to(&bytes, self.host_addr)?;
            Ok(())
        }

        pub fn register_remote_socket(&mut self, addr: SocketAddr) -> Result<()> {
            let id = events::NetworkEvent::ID(self.peer_addrs.len());
            let id_bytes = bincode::serialize(&id).unwrap();
            self.socket.send_to(&id_bytes, addr)?;
            self.peer_addrs.push(addr);
            let peer_addrs_clone = self.peer_addrs.clone();
            let peer_addrs_bytes = bincode::serialize(&events::NetworkEvent::Peers(peer_addrs_clone)).unwrap();
            for peer_addr in self.peer_addrs.iter() {
                self.socket.send_to(&peer_addrs_bytes, peer_addr)?;
            }
            Ok(())
        }

        pub fn tick(&self, player_id: events::PlayerID) -> Result<()> {
            let bytes = bincode::serialize(&events::NetworkEvent::PlayerID(player_id)).unwrap();
            for peer_addr in self.peer_addrs.iter() {
                self.socket.send_to(&bytes, peer_addr)?;
            }
            Ok(())
        }
    }
}
