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
    host_addr: SocketAddr,
    peer_sockets: Vec<SocketAddr>,
}

impl Communicate {
    fn new(sender_addr: SocketAddr, receiver_addr: SocketAddr, host_addr: SocketAddr) -> Result<Communicate> {
        let sender_socket = UdpSocket::bind(&sender_addr)?;
            /* .expect(format!("Couldn't bind to {} for sending UDP data", sender_addr).as_str()); */

        let receiver_socket = UdpSocket::bind(&receiver_addr)?;
            /* .expect(format!("Couldn't bind to {} for receiving UDP data", receiver_addr).as_str()); */

        let local_socket = LocalSocket {
            sender: sender_socket,
            receiver: receiver_socket,
        };

        Ok(Communicate {
            local_socket: local_socket,
            host_addr: host_addr,
            peer_sockets: Vec::new(),
        })
    }

    fn register_self(&self) {
        self.local_socket.sender
            .send_to(&[0x01], self.host_addr)
            .expect("Failed to register self");
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

    fn send_position(&self, position: Point) {
    }

    fn poll_event(&self) {
        self.local_socket.receiver.set_read_timeout(None)
            .expect("Unset set_read_timeout call failed");

        let mut buf = [0; 3];
        let (amt, src) = self.local_socket.receiver
            .recv_from(&mut buf)
            .expect("Failed to receive data");

        // self.process_data(amt, src, &buf);
    }

    fn peek_event(&self, duration: time::Duration) -> Result<(usize, SocketAddr)> {
        self.local_socket.receiver.set_read_timeout(Some(duration))
            .expect(&format!("set_read_timeout call to {:?} failed", duration));

        let mut buf = [0; 3];
        self.local_socket.receiver
            .recv_from(&mut buf)
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
    let receiver_addr: SocketAddr = format!("{}:{}", &bind_interface, &receiver_port)
        .parse()
        .expect("Unable to parse socket address");

    let sender_port: u16 = 9998;
    let sender_addr: SocketAddr = format!("{}:{}", &bind_interface, &sender_port)
        .parse()
        .expect("Unable to parse socket address");

    let host_addr : SocketAddr = match args.len() {
        1 => receiver_addr,
        _ => args[0]
            .parse()
            .expect("Unable to parse socket address"),
    };

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };

    let buf = &[0x00];

    /* thread::sleep(time::Duration::from_millis(1000)); */
    /* local_socket.send_to(buf, receiver_address) */
    /*     .expect("Failed to send data"); */

    let game = match Communicate::new(receiver_addr, sender_addr, host_addr) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    /* let mut event: Result<(usize, SocketAddr)>; */
    thread::spawn(move || {
        let duration = time::Duration::from_millis(5000);
        let event = game.peek_event(duration);
    });

    game.register_self();

    let clonebox = rustbox.clone();
    /* thread::spawn(move || { */
    /*     game.poll_event(); */
    /* }); */
    /* println!("buhaha"); */
    /* match event { */
    /*     Ok(event) => { */
    /*     } */
    /*     Err(e) => { */
    /*         panic!("Could not receive player id from host server"); */
    /*     } */
    /* } */
    /* println!("we peeked it"); */

    /* let mut square = Square { */
    /*     side: 3, */
    /*     coordinates: Point { x: 0, y: 0 }, */
    /*     color: Color::Blue, */
    /* }; */

    /* square.draw(&rustbox); */
    rustbox.lock().unwrap().present();

    /* loop { */
        /* match rustbox_poll(&mut player, &rustbox) { */
        /*     Ok(_) => { }, */
        /*     Err(_) => break, */
        /* } */
    /* } */
}
