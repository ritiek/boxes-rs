use rustbox::RustBox;
use rustbox::Key;

use std::default::Default;
use std::io::Result;
use std::error::Error;
use std::env;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

use boxes_rs::game::{Square, Direction, PlayerColor, Point};
use boxes_rs::network::events::{NetworkEvent, GameEvent, PlayerID};
use boxes_rs::network::receiver::Receiver;
use boxes_rs::network::sender::Sender;


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
    match id {
        0 => PlayerColor::Blue,
        1 => PlayerColor::Red,
        2 => PlayerColor::Green,
        3 => PlayerColor::Yellow,
        4 => PlayerColor::Cyan,
        5 => PlayerColor::Magenta,
        6 => PlayerColor::White,
        _ => PlayerColor::Black,
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let bind_interface: &str = "0.0.0.0";

    let receiver_port: u16 = 9999;
    let receiver_addr: SocketAddr = format!("{}:{}", &bind_interface, &receiver_port)
        .parse()
        .unwrap();

    let sender_port: u16 = 9998;
    let sender_addr: SocketAddr = format!("{}:{}", &bind_interface, &sender_port)
        .parse()
        .unwrap();

    let host_addr: SocketAddr = match args.len() {
        1 => receiver_addr,
        _ => args[1]
            .parse()
            .unwrap(),
    };

    let event_receiver = Receiver::new(receiver_addr)?;
    let event_sender = Arc::new(Mutex::new(Sender::new(sender_addr, host_addr)?));
    let event_sender_clone = event_sender.clone();

    let rustbox = Arc::new(Mutex::new(
        RustBox::init(Default::default()).unwrap()
    ));

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
            let data = event_receiver.poll_event().unwrap();

            match data.event {
                NetworkEvent::PlayerJoin => {
                    let remote_receiver_addr: SocketAddr = format!("{}:9999", data.src.ip())
                        .parse()
                        .unwrap();

                    event_sender_clone.lock().unwrap()
                        .register_remote_socket(remote_receiver_addr)
                        .unwrap();
                }
                NetworkEvent::PlayerID(mut v) => {
                    let current_player_id = player_clone.lock().unwrap().id;

                    if current_player_id == v.player.id {
                        player_clone.lock().unwrap().redraw(v.point, &clonebox);
                    } else {
                        v.player.redraw(v.point, &clonebox);
                    }

                    clonebox.lock().unwrap().present();
                }
                NetworkEvent::Peers(mut v) => {
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

    event_sender.lock().unwrap().register_self()?;

    player.lock().unwrap().draw(&rustbox);
    rustbox.lock().unwrap().present();

    loop {
        let poll = rustbox_poll(&mut player, &event_sender, &rustbox);
        match poll {
            Ok(_) => { },
            Err(_) => break,
        };
    }

    Ok(())
}
