extern crate rustbox;
extern crate bincode;
extern crate serde_derive;
extern crate serde;

use std::env;
use std::error::Error;
use std::default::Default;
use std::net::SocketAddr;
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use rustbox::RustBox;
use rustbox::Key;

use boxes_rs::network::{NetworkEvent, GameEvent, PlayerID};
use boxes_rs::game::{Square, Direction, PlayerColor, Point};
use boxes_rs::receiver::Receiver;
use boxes_rs::sender::Sender;

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
                    Ok(())
                }
            }
        }
        Err(e) => panic!("{}", e.description()),
        _ => {
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
    };
    player_color
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

    let event_receiver = match Receiver::new(receiver_addr) {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    let event_sender = match Sender::new(sender_addr, host_addr) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };


    let event_sender_clone = event_sender.clone();

    let rustbox = match RustBox::init(Default::default()) {
        Ok(v) => Arc::new(Mutex::new(v)),
        Err(e) => panic!("{}", e),
    };

    let clonebox = rustbox.clone();

    let mut player = Arc::new(Mutex::new(Square {
        side: 0,
        coordinates: Point { x: 0, y: 0 },
        color: PlayerColor::Blue,
        id: 0,
    }));

    let player_clone = player.clone();

    thread::spawn(move || {
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

                    match event_sender_clone.lock().unwrap()
                        .register_remote_socket(remote_receiver_addr) {
                            Ok(_) => { },
                            Err(e) => panic!("{}", e),
                    };
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
        Ok(_) => { },
        Err(e) => panic!("{}", e),
    }

    player.lock().unwrap().draw(&rustbox);
    rustbox.lock().unwrap().present();

    loop {
        let poll = rustbox_poll(&mut player, &event_sender, &rustbox);
        match poll {
            Ok(_) => { },
            Err(_) => break,
        };
    }
}
