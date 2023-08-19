use std::net::SocketAddr;

use tokio::{net::{tcp::{ReadHalf, WriteHalf}, TcpStream}, fs::File, sync::broadcast::{Sender, Receiver}};

struct App {
    read: ReadHalf<'static>,
    write: WriteHalf<'static>,
    addr: SocketAddr,
    message_data: File,
    users_data: File,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
}

impl App {
    pub fn init(
        mut socket: TcpStream,
        addr: SocketAddr,
        tx: Sender<(String, SocketAddr)>,
        mut rx: Receiver<(String, SocketAddr)>,
    ) -> App {
   
        let (read, write) = socket.split();

        let app = App {
            read,
            write,
            addr,
            message_data: todo!(),
            users_data: todo!(),
            tx,
            rx,
        };
        return app;
    }
}


enum FrameType {
    Message,
    Login,
    Register,
    Error
}

struct Login {
    login: String,
    password: String
}

enum FrameData {
    Message(String),
    Login(Login),
    None
}

pub struct Frame {
    frame_type: FrameType,
    data: FrameData
}

impl Frame {
    pub fn handle(self) {

    }
}

pub fn frame_check(input: String) -> Frame {

    let frame_vec: Vec<String> = input.split("|").map(|e| e.to_string()).collect();
    let mut frame: Frame;
    match frame_vec[0].as_str() {
        "message" => {
            frame = Frame {
                frame_type: FrameType::Message,
                data: FrameData::Message(frame_vec[1].clone()),
            };
        },
        "login" => {
            frame = Frame {
                frame_type: FrameType::Login,
                data: FrameData::Login(Login { login: frame_vec[1].clone(), password: frame_vec[2].clone() }),
            }
        }
        "register" => {
            frame = Frame {
                frame_type: FrameType::Register,
                data: FrameData::Login(Login { login: frame_vec[1].clone(), password: frame_vec[2].clone() }),
            }
        }
        _ => {
            frame = Frame {
                frame_type: FrameType::Error,
                data: FrameData::None,
            }
        }
    }

    return frame;
}
