extern crate rustbox;

use std::error::Error;
use std::default::Default;
use std::net::UdpSocket;
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::{thread, time};

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
enum Event {
    Quit,
    Direction(Direction),
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
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

    fn move_in_direction(&mut self, direction: Direction, rustbox: &Arc<Mutex<RustBox>>) {
        let new_coordinates: Point = match direction {
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
        self.redraw(new_coordinates, &rustbox);
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
                        square.move_in_direction(direction, &rustbox);
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

fn receive_data(bind_socket: &str, rustbox: &Arc<Mutex<RustBox>>) {
    let mut square = Square {
        side: 3,
        coordinates: Point { x: 12, y: 12 },
        color: Color::Red,
    };
    /* receive_data(receiver_socket, &rustbox); */
    let socket = UdpSocket::bind(&bind_socket)
        .expect(format!("Couldn't bind to {}", bind_socket).as_str());
    let mut buf = [0; 3];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)
            .expect("Failed to receive data");
        /* println!("{:?}", &buf[0 .. amt]); */
        square.draw(&rustbox);
        rustbox.lock().unwrap().present();
    }
}

fn send_data(from_socket: &str, to_socket: &str) {
    let socket = UdpSocket::bind(&from_socket)
        .expect(format!("Couldn't bind to {}", from_socket).as_str());
}

fn main() {
    let bind_interface = "127.0.0.1";

    let bind_port = 9999;
    let bind_socket = format!("{}:{}", bind_interface, bind_port);

    /* let sender_port = 9998; */
    /* let sender_socket = format!("{}:{}", interface, sender_port); */

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };


    let clonebox = rustbox.clone();
    thread::spawn(move || {
        receive_data(&bind_socket, &clonebox);
    });

    let mut square = Square {
        side: 3,
        coordinates: Point { x: 0, y: 0 },
        color: Color::Blue,
    };

    square.draw(&rustbox);
    rustbox.lock().unwrap().present();

    loop {
        match rustbox_poll(&mut square, &rustbox) {
            Ok(_) => { },
            Err(_) => break,
        }
    }
}
