extern crate rustbox;

use std::error::Error;
use std::default::Default;

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
    Right,
    Left,
    Down,
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

    fn move_up(&mut self, rustbox: &RustBox) {
        self.move_in_direction(Direction::Up, &rustbox);
    }

    fn move_right(&mut self, rustbox: &RustBox) {
        self.move_in_direction(Direction::Right, &rustbox);
    }

    fn move_down(&mut self, rustbox: &RustBox) {
        self.move_in_direction(Direction::Down, &rustbox);
    }

    fn move_left(&mut self, rustbox: &RustBox) {
        self.move_in_direction(Direction::Left, &rustbox);
    }

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

fn main() {
    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    let mut square = Square {
        side: 3,
        coordinates: Point {x: 0, y: 0},
        color: Color::Blue,
    };

    square.draw(&rustbox);
    rustbox.present();

    loop {
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char('q') => { break; }
                    Key::Up => {
                        square.move_up(&rustbox);
                        rustbox.present();
                    }
                    Key::Right => {
                        square.move_right(&rustbox);
                        rustbox.present();
                    }
                    Key::Left => {
                        square.move_left(&rustbox);
                        rustbox.present();
                    }
                    Key::Down => {
                        square.move_down(&rustbox);
                        rustbox.present();
                    }
                    _ => { }
                }
            },
            Err(e) => panic!("{}", e.description()),
            _ => { }
        }
    }
}
