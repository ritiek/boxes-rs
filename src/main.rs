extern crate rustbox;
extern crate bincode;
extern crate serde_derive;
extern crate serde;

use std::env;
use std::error::Error;
use std::default::Default;
use std::net::{SocketAddr, UdpSocket};
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use std::process;

use rustbox::{Color, RustBox};
use rustbox::Key;
use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/* #[derive(Clone, Copy, Debug, PartialEq)] */
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum PlayerColor {
    Blue,
    Red,
    Green,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

#[derive(Debug, Serialize, Deserialize)]
struct Square {
    side: usize,
    coordinates: Point,
    color: PlayerColor,
    id: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/* #[derive(Clone, Copy, Debug)] */
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/* #[derive(Clone, Copy, Debug)] */
enum GameEvent {
    Quit,
    Direction(Direction),
}

impl Square {
    fn draw(&self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint('█', &rustbox);
    }

    fn erase(&self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint(' ', &rustbox);
    }

    fn redraw(&mut self, coordinates: Point, rustbox: &Arc<Mutex<RustBox>>) {
        self.erase(&rustbox);
        self.coordinates = coordinates;
        self.draw(&rustbox);
    }

    fn paint(&self, character: char, rustbox: &Arc<Mutex<RustBox>>) {
        let square_row = &character.to_string().repeat(self.side*2);
        for increment in 0..self.side {
            rustbox.lock().unwrap().print(
                self.coordinates.x as usize,
                self.coordinates.y as usize + increment,
                rustbox::RB_BOLD,
                player_color_to_color(self.color),
                Color::Black,
                square_row,
            );
        }
    }

    fn move_in_direction(&mut self, direction: Direction) -> Point {
        let coordinates = match direction {
            Direction::Up => {
                Point { x: self.coordinates.x, y: self.coordinates.y - 1 }
            }
            Direction::Right => {
                Point { x: self.coordinates.x + 1, y: self.coordinates.y }
            }
            Direction::Left => {
                Point { x: self.coordinates.x - 1, y: self.coordinates.y }
            }
            Direction::Down => {
                Point { x: self.coordinates.x, y: self.coordinates.y + 1 }
            }
        };
        /* println!("moved {:?}", coordinates); */
        coordinates
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerID {
    point: Point,
    player: Square,
}

#[derive(Debug, Serialize, Deserialize)]
enum NetworkEvent {
    PlayerID(PlayerID),
    Peers(Vec<SocketAddr>),
    PlayerJoin,
    PlayerLeft,
    Nothing,
    ID(usize),
}

#[derive(Debug)]
struct NetworkData {
    amt: usize,
    src: SocketAddr,
    /* buf: [u8; 300] */
    event: NetworkEvent,
}

#[derive(Debug)]
struct Receiver {
    socket: UdpSocket,
}

impl Receiver {
    fn new(addr: SocketAddr) -> Result<Receiver> {
        let socket = UdpSocket::bind(&addr)?;
            /* .expect(format!("Couldn't bind to {} for sending UDP data", sender_addr).as_str()); */

        Ok(Receiver {
            socket: socket,
        })
    }

    fn poll_event(&self) -> Result<NetworkData> {
        self.socket.set_read_timeout(None)?;
            /* .expect("Unset set_read_timeout call failed"); */

        let mut buf = [0; 300];
        let (amt, src) = self.socket.recv_from(&mut buf)?;
        /*     .expect("Failed to receive data"); */

        let event: NetworkEvent = bincode::deserialize(&buf).unwrap();
        /* println!("{:?}", event); */
        Ok(NetworkData {
            amt: amt,
            src: src,
            event: event,
        })
        // self.process_data(amt, src, &buf);
    }

    fn peek_event(&self, duration: time::Duration) -> Result<NetworkData> {
        self.socket.set_read_timeout(Some(duration))?;
            /* .expect(&format!("set_read_timeout call to {:?} failed", duration)); */

        let mut buf = [0; 300];
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        let event: NetworkEvent = bincode::deserialize(&buf).unwrap();
        Ok(NetworkData {
            amt: amt,
            src: src,
            event: event,
        })
    }
}

#[derive(Debug)]
struct Sender {
    socket: UdpSocket,
    host_addr: SocketAddr,
    peer_addrs: Vec<SocketAddr>,
}

impl Sender {
    fn new(addr: SocketAddr, host_addr: SocketAddr) -> Result<Sender> {
        let socket = UdpSocket::bind(&addr)?;
            /* .expect(format!("Couldn't bind to {} for sending UDP data", sender_addr).as_str()); */

        Ok(Sender {
            socket: socket,
            host_addr: host_addr,
            peer_addrs: Vec::new(),
        })
    }

    fn register_self(&self) -> Result<()> {
        let bytes = bincode::serialize(&NetworkEvent::PlayerJoin).unwrap();
        let interface_addr: SocketAddr = "0.0.0.0:9999".parse().unwrap();
        if interface_addr != self.host_addr {
            self.socket.send_to(&bytes, interface_addr)?;
        }
        self.socket.send_to(&bytes, self.host_addr)?;
            /* .expect("Failed to register self"); */
        Ok(())
    }

    fn register_remote_socket(&mut self, addr: SocketAddr) -> Result<()> {
        let id = NetworkEvent::ID(self.peer_addrs.len());
        let id_bytes = bincode::serialize(&id).unwrap();
        self.socket.send_to(&id_bytes, addr)?;
            /* .expect("Failed to register remote socket"); */
        self.peer_addrs.push(addr);
        let peer_addrs_clone = self.peer_addrs.clone();
        let peer_addrs_bytes = bincode::serialize(&NetworkEvent::Peers(peer_addrs_clone)).unwrap();
        for peer_addr in self.peer_addrs.iter() {
            self.socket.send_to(&peer_addrs_bytes, peer_addr)?;
        }
        Ok(())
    }

    fn tick(&self, player_id: PlayerID) -> Result<()> {
        let bytes = bincode::serialize(&NetworkEvent::PlayerID(player_id)).unwrap();
        for peer_addr in self.peer_addrs.iter() {
            self.socket.send_to(&bytes, peer_addr)?;
        }
            /* .expect("Failed to register self"); */
        Ok(())
    }

}

fn match_event(key: Key) -> Option<GameEvent> {
    match key {
        Key::Char('q') => Some(GameEvent::Quit),
        Key::Up        => Some(GameEvent::Direction(Direction::Up)),
        Key::Down      => Some(GameEvent::Direction(Direction::Down)),
        Key::Left      => Some(GameEvent::Direction(Direction::Left)),
        Key::Right     => Some(GameEvent::Direction(Direction::Right)),
        _              => None
    }
}

fn rustbox_poll(square: &mut Arc<Mutex<Square>>, event_sender: &Arc<Mutex<Sender>>, rustbox: &Arc<Mutex<RustBox>>) -> Result<()> {
    let delay = time::Duration::from_nanos(1000);
    let pe = rustbox.lock().unwrap().peek_event(delay, false);
    /* let pe = rustbox.lock().unwrap().poll_event(false); */
    let side = square.lock().unwrap().side;
    let coordinates = square.lock().unwrap().coordinates;
    let color = square.lock().unwrap().color;
    let id = square.lock().unwrap().id;
    let square_dump = Square {
        side: side,
        coordinates: coordinates,
        color: color,
        id: id,
    };
    match pe {
        Ok(rustbox::Event::KeyEvent(key)) => {
            match match_event(key) {
                Some(event) => {
                    match event {
                        GameEvent::Direction(direction) => {
                            let position = square.lock().unwrap().move_in_direction(direction);
                            /* square.lock().unwrap().redraw(position, &rustbox); */
                            /* rustbox.lock().unwrap().present(); */
                            /* println!("{:?}", position); */
                            /* Ok(square.lock().unwrap().coordinates) */
                            /* Ok(position) */
                            let player_id = PlayerID { point: position, player: square_dump };
                            event_sender.lock().unwrap().tick(player_id).unwrap();
                            Ok(())
                        }
                        GameEvent::Quit => {
                            Err(std::io::Error::new(std::io::ErrorKind::Other,
                                    "Received exit signal"))
                        }
                    }
                }
                None => {
                    let point = square.lock().unwrap().coordinates;
                    let player_id = PlayerID { point: point, player: square_dump };
                    Ok(())
                }
            }
        }
        Err(e) => panic!("{}", e.description()),
        _ => {
            let point = square.lock().unwrap().coordinates;
            let player_id = PlayerID { point: point, player: square_dump };
            Ok(())
        },
    }
}

fn id_to_player_color(id: usize) -> PlayerColor {
    let player_color = match id {
        0 => PlayerColor::Blue,
        1 => PlayerColor::Red,
        2 => PlayerColor::Green,
        3 => PlayerColor::Yellow,
        4 => PlayerColor::Cyan,
        5 => PlayerColor::Magenta,
        6 => PlayerColor::White,
        _ => PlayerColor::Black,
        /* _ => panic!("too many players"), */
    };
    player_color
}

fn player_color_to_color(player_color: PlayerColor) -> Color {
    let color = match player_color {
        PlayerColor::Blue => Color::Blue,
        PlayerColor::Red => Color::Red,
        PlayerColor::Green => Color::Green,
        PlayerColor::Yellow => Color::Yellow,
        PlayerColor::Cyan => Color::Cyan,
        PlayerColor::Magenta => Color::Magenta,
        PlayerColor::White => Color::White,
        PlayerColor::Black => Color::Black,
    };
    color
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_interface: &str = "0.0.0.0";

    let receiver_port: u16 = 9999;
    let receiver_addr: SocketAddr = format!("{}:{}", &bind_interface, &receiver_port)
        .parse()
        .expect("Unable to parse socket address");

    let sender_port: u16 = 9998;
    let sender_addr: SocketAddr = format!("{}:{}", &bind_interface, &sender_port)
        .parse()
        .expect("Unable to parse socket address");

    let host_addr: SocketAddr = match args.len() {
        1 => receiver_addr,
        _ => args[1]
            .parse()
            .expect("Unable to parse socket address"),
    };

    let buf = &[0x00];

    let event_receiver = match Receiver::new(receiver_addr) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    let event_sender = match Sender::new(sender_addr, host_addr) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };


    // --------------------
    // SOCKET COMMUNICATION
    // --------------------

    let event_sender_clone = event_sender.clone();

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };

    let clonebox = rustbox.clone();

    /* let mut player: Arc<Mutex<Square>>; */
    let mut player = Arc::new(Mutex::new(Square {
        side: 0,
        coordinates: Point { x: 0, y: 0 },
        color: PlayerColor::Blue,
        id: 0,
    }));

    let mut player_clone = player.clone();

    let registrar = thread::spawn(move || {
        let mut players: Vec<Square> = Vec::new();
        loop {
            let data = match event_receiver.poll_event() {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            };
            /* println!("{:?}", data); */

            match data.event {
                NetworkEvent::PlayerJoin => {
                    /* player_clone.lock().unwrap().side = 4; */
                    let remote_receiver_addr: SocketAddr = format!("{}:9999", data.src.ip())
                        .parse()
                        .unwrap();
                    event_sender_clone.lock().unwrap().register_remote_socket(remote_receiver_addr);
                }
                NetworkEvent::PlayerID(mut v) => {
                    /* println!("{:?}", v); */
                    let current_player_id = player_clone.lock().unwrap().id;

                    if current_player_id == v.player.id {
                        player_clone.lock().unwrap().redraw(v.point, &clonebox);
                    } else {
                        v.player.redraw(v.point, &clonebox);
                    }

                    clonebox.lock().unwrap().present();
                }
                NetworkEvent::Peers(mut v) => {
                    /* println!("{:?}", v); */
                    v[0] = format!("{}:9999", data.src.ip()).parse().unwrap();
                    event_sender_clone.lock().unwrap().peer_addrs = v;
                }
                NetworkEvent::ID(v) => {
                    player_clone.lock().unwrap().id = v;
                    player_clone.lock().unwrap().side = 3;
                    player_clone.lock().unwrap().color = id_to_player_color(v);
                    player_clone.lock().unwrap().draw(&clonebox);
                    clonebox.lock().unwrap().present();
                }
                _ => { },
            }
        }
    });

    match event_sender.lock().unwrap().register_self() {
        Ok(_) => { ; },
        Err(e) => panic!("{}", e),
    }

    player.lock().unwrap().draw(&rustbox);
    rustbox.lock().unwrap().present();

    loop {
        let poll = rustbox_poll(&mut player, &event_sender, &rustbox);
        match poll {
            Ok(v) => {
                /* /1* println!("{:?}", v) *1/ */
                /* event_sender.lock().unwrap().tick(v).unwrap(); */
                /* /1* rustbox.lock().unwrap().present(); *1/ */
            },
            Err(_) => break,
        };
        /* let duration = time::Duration::from_millis(500); */
        /* thread::sleep(duration); */
        /* println!("{:?}", poll); */
    }
}
