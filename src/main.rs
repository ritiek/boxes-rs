extern crate rustbox;

use std::error::Error;
use std::default::Default;
use std::net::UdpSocket;
use std::process;
use std::io::Result;

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
    fn draw(self, rustbox: &RustBox) {
        self.paint("█", &rustbox);
    }

    fn erase(self, rustbox: &RustBox) {
        self.paint(" ", &rustbox);
    }

    fn redraw(&mut self, coordinates: Point, rustbox: &RustBox) {
        self.erase(&rustbox);
        self.coordinates = coordinates;
        self.draw(&rustbox);
    }

    fn paint(self, character: &str, rustbox: &RustBox) {
        let square_row = &character.repeat(self.side*2);
        for increment in 0..self.side {
            rustbox.print(
                self.coordinates.x,
                self.coordinates.y + increment,
                rustbox::RB_BOLD,
                self.color,
                Color::Black,
                square_row,
            );
        }
    }

    /* fn move_up(&mut self, rustbox: &RustBox) { */
    /*     self.move_in_direction(Direction::Up, &rustbox); */
    /* } */

    /* fn move_down(&mut self, rustbox: &RustBox) { */
    /*     self.move_in_direction(Direction::Down, &rustbox); */
    /* } */

    /* fn move_left(&mut self, rustbox: &RustBox) { */
    /*     self.move_in_direction(Direction::Left, &rustbox); */
    /* } */

    /* fn move_right(&mut self, rustbox: &RustBox) { */
    /*     self.move_in_direction(Direction::Right, &rustbox); */
    /* } */

    fn move_in_direction(&mut self, direction: Direction, rustbox: &RustBox) {
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

fn rustbox_poll(square: &mut Square, rustbox: &RustBox) -> Result<()> {
    match rustbox.poll_event(false) {
        Ok(rustbox::Event::KeyEvent(key)) => {
            let event = match_event(key);
            if let Some(event) = event {
                match event {
                    Event::Direction(direction) => {
                        square.move_in_direction(direction, &rustbox);
                        rustbox.present();
                    }
                    Event::Quit => {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Received exit signal"));
                    }
                }
            }
        }
        Err(e) => panic!("{}", e.description()),
        _ => { }
    }
    Ok(())
}

fn main() {
    /* let port = 34254; */
    /* let bind_address = format!("127.0.0.1:{}", port); */
    /* let mut socket = UdpSocket::bind(&bind_address) */
    /*     .expect(format!("Couldn't bind to {}", bind_address).as_str()); */

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    let mut square = Square {
        side: 3,
        coordinates: Point {x: 0, y: 0},
        color: Color::Blue,
    };

    square.draw(&rustbox);
    rustbox.present();

    loop {
        match rustbox_poll(&mut square, &rustbox) {
            Ok(_) => { },
            Err(_) => break,
        }
    }
}
