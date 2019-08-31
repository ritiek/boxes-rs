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

#[derive(Debug)]
struct Square {
    side: usize,
    coordinates: Point,
    color: Color,
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
                self.color,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum NetworkEvent {
    Point(Point),
    PlayerJoin,
    PlayerLeft,
    Nothing,
}

#[derive(Clone, Copy, Debug)]
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
        Ok(NetworkData {
            amt: amt,
            src: src,
            event: bincode::deserialize(&buf).unwrap(),
        })
        // self.process_data(amt, src, &buf);
    }

    fn peek_event(&self, duration: time::Duration) -> Result<NetworkData> {
        self.socket.set_read_timeout(Some(duration))?;
            /* .expect(&format!("set_read_timeout call to {:?} failed", duration)); */

        let mut buf = [0; 300];
        let (amt, src) = self.socket.recv_from(&mut buf)?;

        Ok(NetworkData {
            amt: amt,
            src: src,
            event: bincode::deserialize(&buf).unwrap(),
        })
    }
}

#[derive(Debug)]
struct Sender {
    socket: UdpSocket,
    host_addr: SocketAddr,
    peer_addr: Vec<SocketAddr>,
}

impl Sender {
    fn new(addr: SocketAddr, host_addr: SocketAddr) -> Result<Sender> {
        let socket = UdpSocket::bind(&addr)?;
            /* .expect(format!("Couldn't bind to {} for sending UDP data", sender_addr).as_str()); */

        Ok(Sender {
            socket: socket,
            host_addr: host_addr,
            peer_addr: Vec::new(),
        })
    }

    fn register_self(&self) -> Result<()> {
        let bytes = bincode::serialize(&NetworkEvent::PlayerJoin).unwrap();
        self.socket.send_to(&bytes, self.host_addr)?;
            /* .expect("Failed to register self"); */
        Ok(())
    }

    fn register_remote_socket(&mut self, addr: SocketAddr) {
        self.socket
            .send_to(
                &(self.peer_addr.len() as u8).to_be_bytes(),
                addr
            )
            .expect("Failed to register remote socket");

        self.peer_addr.push(addr);
    }

    fn tick(&self, position: Point) -> Result<()> {
        let bytes = bincode::serialize(&NetworkEvent::Point(position)).unwrap();
        self.socket.send_to(&bytes, self.host_addr)?;
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

fn rustbox_poll(square: &mut Arc<Mutex<Square>>, event_sender: &Arc<Mutex<Sender>>, rustbox: &Arc<Mutex<RustBox>>) -> Result<Point> {
    let delay = time::Duration::from_millis(10);
    let pe = rustbox.lock().unwrap().peek_event(delay, false);
    /* let pe = rustbox.lock().unwrap().poll_event(false); */
    match pe {
        Ok(rustbox::Event::KeyEvent(key)) => {
            match match_event(key) {
                Some(event) => {
                    match event {
                        GameEvent::Direction(direction) => {
                            let position = square.lock().unwrap().move_in_direction(direction);
                            event_sender.lock().unwrap().tick(position).unwrap();
                            /* square.lock().unwrap().redraw(position, &rustbox); */
                            /* rustbox.lock().unwrap().present(); */
                            /* println!("{:?}", position); */
                            Ok(square.lock().unwrap().coordinates)
                            /* Ok(position) */
                        }
                        GameEvent::Quit => {
                            Err(std::io::Error::new(std::io::ErrorKind::Other,
                                    "Received exit signal"))
                        }
                    }
                }
                None => {
                    Ok(square.lock().unwrap().coordinates)
                }
            }
        }
        Err(e) => panic!("{}", e.description()),
        _ => Ok(square.lock().unwrap().coordinates),
    }
}

/* fn rustbox_poll(square: &mut Square, rustbox: &Arc<Mutex<RustBox>>) -> Result<()> { */
    /* let delay = time::Duration::from_millis(10); */
    /* let pe = rustbox.lock().unwrap().peek_event(delay, false); */
    /* /1* let pe = rustbox.lock().unwrap().poll_event(false); *1/ */
    /* match pe { */
    /*     Ok(rustbox::Event::KeyEvent(key)) => { */
    /*         let event = match_event(key); */
    /*         if let Some(event) = event { */
    /*             match event { */
    /*                 GameEvent::Direction(direction) => { */
    /*                     let position = square.move_in_direction(direction); */
    /*                     square.redraw(position, &rustbox); */
    /*                     rustbox.lock().unwrap().present(); */
    /*                 } */
    /*                 GameEvent::Quit => { */
    /*                     return Err(std::io::Error::new(std::io::ErrorKind::Other, "Received exit signal")); */
    /*                 } */
    /*             } */
    /*         } */
    /*     } */
    /*     Err(e) => panic!("{}", e.description()), */
    /*     _ => { }, */
    /* } */
    /* Ok(()) */
/* } */


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

    let host_addr: SocketAddr = match args.len() {
        1 => receiver_addr,
        _ => args[0]
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

    let registrar = thread::spawn(move || {
        println!("Waiting for connection...");
        /* let duration = time::Duration::from_millis(5000); */
        /* event_receiver.peek_event(duration) */
        (event_receiver.poll_event(), event_receiver)
    });

    match event_sender.lock().unwrap().register_self() {
        Ok(_) => { ; },
        Err(e) => panic!("{}", e),
    }

    let (event, event_receiver) = registrar.join().unwrap();

    /* let duration = time::Duration::from_millis(4000); */
    /* thread::sleep(duration); */

    let data: NetworkData = match event {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    event_sender.lock().unwrap().peer_addr.push(data.src);

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };

    /* println!("buhaha"); */
    /* match event { */
    /*     Ok(event) => { */
    /*     } */
    /*     Err(e) => { */
    /*         panic!("Could not receive player id from host server"); */
    /*     } */
    /* } */
    /* println!("we peeked it"); */

    let mut player = Arc::new(Mutex::new(Square {
        side: 3,
        coordinates: Point { x: 0, y: 0 },
        color: Color::Blue,
    }));

    player.lock().unwrap().draw(&rustbox);
    /* rustbox.lock().unwrap().present(); */

    let clonebox = rustbox.clone();

    let player2 = player.clone();
    thread::spawn(move || {
        loop {
            let event = event_receiver.poll_event();
            let data: NetworkData = match event {
                Ok(v) => v,
                Err(e) => panic!("{}", e),
            };
            match data.event {
                NetworkEvent::Point(v) => {
                    /* let point = Point { x: 10, y: 10 }; */
                    /* let duration = time::Duration::from_millis(500); */
                    /* thread::sleep(duration); */
                    /* println!("processing {:?}", v); */
                    /* player2.lock().unwrap().redraw(point, &clonebox); */
                    player2.lock().unwrap().redraw(v, &clonebox);
                    clonebox.lock().unwrap().present();
                }
                _ => { ; }
            }
        }
    });

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
