use serde_derive::{Serialize, Deserialize};
use crate::game::{Direction, Point, Square};
use std::net::{SocketAddr};

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
