use rustbox::{Color, RustBox};
use serde_derive::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PlayerColor {
    Blue,
    Red,
    Green,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn player_color_to_color(player_color: PlayerColor) -> Color {
    match player_color {
        PlayerColor::Blue => Color::Blue,
        PlayerColor::Red => Color::Red,
        PlayerColor::Green => Color::Green,
        PlayerColor::Yellow => Color::Yellow,
        PlayerColor::Cyan => Color::Cyan,
        PlayerColor::Magenta => Color::Magenta,
        PlayerColor::White => Color::White,
        PlayerColor::Black => Color::Black,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Square {
    pub side: usize,
    pub coordinates: Point,
    pub color: PlayerColor,
    pub id: usize,
}

impl Square {
    pub fn draw(&self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint('█', &rustbox);
    }

    pub fn erase(&self, rustbox: &Arc<Mutex<RustBox>>) {
        self.paint(' ', &rustbox);
    }

    pub fn redraw(&mut self, coordinates: Point, rustbox: &Arc<Mutex<RustBox>>) {
        self.erase(&rustbox);
        self.coordinates = coordinates;
        self.draw(&rustbox);
    }

    pub fn paint(&self, character: char, rustbox: &Arc<Mutex<RustBox>>) {
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

    pub fn move_in_direction(&mut self, direction: Direction) -> Point {
        match direction {
            Direction::Up => {
                if self.coordinates.y > 0 {
                    Point { x: self.coordinates.x, y: self.coordinates.y - 1 }
                } else {
                    Point { x: self.coordinates.x, y: self.coordinates.y }
                }
            }
            Direction::Right => {
                Point { x: self.coordinates.x + 1, y: self.coordinates.y }
            }
            Direction::Left => {
                if self.coordinates.x > 0 {
                    Point { x: self.coordinates.x - 1, y: self.coordinates.y }
                } else {
                    Point { x: self.coordinates.x, y: self.coordinates.y }
                }
            }
            Direction::Down => {
                Point { x: self.coordinates.x, y: self.coordinates.y + 1 }
            }
        }
    }
}
