extern crate rustbox;

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

#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Clone, Copy, Debug)]
struct Square {
    side: usize,
    coordinates: Point,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum Event {
    Quit,
    Direction(Direction),
}

impl Square {
    fn draw(self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint('█', &rustbox);
    }

    fn erase(self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint(' ', &rustbox);
    }

    fn redraw(&mut self, coordinates: Point, rustbox: &Arc<Mutex<RustBox>>) {
        self.erase(&rustbox);
        self.coordinates = coordinates;
        self.draw(&rustbox);
    }

    fn paint(self, character: char, rustbox: &Arc<Mutex<RustBox>>) {
        let square_row = &character.to_string().repeat(self.side*2);
        for increment in 0..self.side {
            rustbox.lock().unwrap().print(
                self.coordinates.x,
                self.coordinates.y + increment,
                rustbox::RB_BOLD,
                self.color,
                Color::Black,
                square_row,
            );
        }
    }

    fn move_in_direction(&mut self, direction: Direction) -> Point {
        match direction {
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
        }
    }
}

#[derive(Debug)]
struct LocalSocket {
    sender: UdpSocket,
    receiver: UdpSocket,
}

#[derive(Debug)]
struct Communicate {
    local_socket: LocalSocket,
    host_addr: Option<SocketAddr>,
    peer_sockets: Vec<SocketAddr>,
}

impl Communicate {
    fn new(sender_addr: SocketAddr, receiver_addr: SocketAddr, host_addr: Option<SocketAddr>) -> Communicate {
        let sender_socket = UdpSocket::bind(&sender_addr)
            .expect(format!("Couldn't bind to {} for sending UDP data", sender_addr).as_str());

        let receiver_socket = UdpSocket::bind(&receiver_addr)
            .expect(format!("Couldn't bind to {} for receiving UDP data", receiver_addr).as_str());

        let local_socket = LocalSocket {
            sender: sender_socket,
            receiver: receiver_socket,
        };

        Communicate {
            local_socket: local_socket,
            host_addr: host_addr,
            peer_sockets: Vec::new(),
        }
    }

    fn register_self(&self) {
        match self.host_addr {
            Some(addr) => self.local_socket.sender
                .send_to(&[0x01], addr)
                .expect("Failed to register self"),
            None => panic!("Cannot self register without any known host"),
        };
    }

    fn register_remote_socket(&mut self, addr: SocketAddr) {
        self.local_socket.sender
            .send_to(
                &(self.peer_sockets.len() as u8).to_be_bytes(),
                addr
            )
            .expect("Failed to register remote socket");

        self.peer_sockets.push(addr);
    }

    fn send_position() {
        
    }

    fn receive_data(&self, socket: &UdpSocket, rustbox: &Arc<Mutex<RustBox>>) {
        let mut players: Vec<Square> = Vec::new();
        let mut buf = [0; 3];
        loop {
            let (amt, src) = socket.recv_from(&mut buf)
                .expect("Failed to receive data");
            let remote_bind_socket = format!("{}:9997", src.ip());
            self.process_data(amt, src, &buf);
            rustbox.lock().unwrap().present();
            /* square.draw(&rustbox); */
            /* rustbox.lock().unwrap().present(); */
        }
    }

    fn process_data(&self, amt: usize, src: SocketAddr, buf: &[u8]) {
        match amt {
            1 => {
                match buf[0] {
                    255 => {
                        /* socket.send_to(&(players.len() as u8).to_be_bytes(), remote_bind_socket); */
                    }
                    0 => {
                        let mut player = Square {
                            side: 3,
                            coordinates: Point { x: 0, y: 0 },
                            color: Color::Blue,
                        };
                        /* player.draw(&rustbox); */
                        /* players.push(player); */
                    }
                    _ => { }
                }
            }
            2 => {
                
            }
            _ => { },
        }
    }
}

fn match_event(key: Key) -> Option<Event> {
    match key {
        Key::Char('q') => Some(Event::Quit),
        Key::Up        => Some(Event::Direction(Direction::Up)),
        Key::Down      => Some(Event::Direction(Direction::Down)),
        Key::Left      => Some(Event::Direction(Direction::Left)),
        Key::Right     => Some(Event::Direction(Direction::Right)),
        _              => None
    }
}

fn rustbox_poll(square: &mut Square, rustbox: &Arc<Mutex<RustBox>>) -> Result<()> {
    let delay = time::Duration::from_millis(10);
    let pe = rustbox.lock().unwrap().peek_event(delay, false);
    /* let pe = rustbox.lock().unwrap().poll_event(false); */
    match pe {
        Ok(rustbox::Event::KeyEvent(key)) => {
            let event = match_event(key);
            if let Some(event) = event {
                match event {
                    Event::Direction(direction) => {
                        let position = square.move_in_direction(direction);
                        square.redraw(position, &rustbox);
                        rustbox.lock().unwrap().present();
                    }
                    Event::Quit => {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Received exit signal"));
                    }
                }
            }
        }
        Err(e) => panic!("{}", e.description()),
        _ => { },
    }
    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_interface: &str = "127.0.0.1";

    let receiver_port: u16 = 9999;
    let receiver_address: SocketAddr = format!("{}:{}", &bind_interface, &receiver_port)
        .parse()
        .expect("Unable to parse socket address");

    let sender_port: u16 = 9998;
    let sender_address: SocketAddr = format!("{}:{}", &bind_interface, &sender_port)
        .parse()
        .expect("Unable to parse socket address");

    let host_address: Option<SocketAddr> = match args.len() {
        1 => None,
        _ => Some(args[0]
            .parse()
            .expect("Unable to parse socket address")
            ),
    };

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };

    let buf = &[0x00];

    /* thread::sleep(time::Duration::from_millis(1000)); */
    /* local_socket.send_to(buf, receiver_address) */
    /*     .expect("Failed to send data"); */

    let game = Communicate::new(receiver_address, sender_address, host_address);
    match game.host_addr {
        Some(v) => {
            // We are a client to someother host
        }
        None => {
            // We are the host
            game.register_self();
        }
    }

    let clonebox = rustbox.clone();
    /* thread::spawn(move || { */
    /*     receive_data(&local_socket, &clonebox); */
    /* }); */

    /* let mut square = Square { */
    /*     side: 3, */
    /*     coordinates: Point { x: 0, y: 0 }, */
    /*     color: Color::Blue, */
    /* }; */

    /* square.draw(&rustbox); */
    rustbox.lock().unwrap().present();

    loop {
        /* match rustbox_poll(&mut player, &rustbox) { */
        /*     Ok(_) => { }, */
        /*     Err(_) => break, */
        /* } */
    }
}
