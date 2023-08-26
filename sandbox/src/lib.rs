use std::{error::Error, net::SocketAddr};

use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::tcp::{ReadHalf, WriteHalf},
    sync::broadcast::{Receiver, Sender},
};

pub struct App<'a> {
    reader: BufReader<ReadHalf<'a>>,
    read_line: String,
    write: WriteHalf<'a>,
    addr: SocketAddr,
    message_data: File,
    users_data: File,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
}

impl<'a> App<'a> {
    pub async fn init(
        read: ReadHalf<'a>,
        mut write: WriteHalf<'a>,
        addr: SocketAddr,
        tx: Sender<(String, SocketAddr)>,
        rx: Receiver<(String, SocketAddr)>,
    ) -> Result<App<'a>, Box<dyn Error>> {
        let reader = BufReader::new(read);
        let line = String::new();

        // message history
        let mut file_data = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("Rust-Chat/src/data.txt")
            .await?;
        let mut data = String::new();
        file_data.read_to_string(&mut data).await?;
        write.write_all(data.as_bytes()).await?;
        // users data 

        let mut users_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("Rust-Chat/src/users.txt")
            .await?;
        let mut users = String::new();
        users_file.read_to_string(&mut users).await.unwrap();

        //
        let app: App<'a> = App {
            reader,
            read_line: line,
            write,
            addr,
            message_data: file_data,
            users_data: users_file,
            tx,
            rx,
        };
        return Ok(app);
    }

    pub async fn start_event_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            tokio::select! {
                result = self.reader.read_line(&mut self.read_line) => {

                    println!("{}", self.read_line);
                    if result.unwrap() == 0 {
                        break Ok(());
                    }

                    //frame handler
                    let frame = Frame::new(self.read_line.clone());
                    self.read_line.clear();
                    self.handle(frame).await?;
                    //
                },
                result = self.rx.recv() => {

                    let (message, some_addr) = result.unwrap();
                    if some_addr != self.addr {
                        ////////////???? frame creator
                        self.write.write_all(message.as_bytes()).await?;

                    }
                    self.read_line.clear();

                }
            };
        }
    }

    async fn handle(&mut self, frame: Frame) -> Result<(), Box<dyn Error>> {
        match frame.data {
            FrameData::Message(data) => {
                self.tx.send((data.clone(), self.addr))?;
                // message history
                self.message_data.write_all(data.as_bytes()).await?;
                //
            }
            FrameData::Login(data) => {
                let mut line = String::new();
                self.users_data.read_to_string(&mut line).await?;
                for line in line.lines() {
                    if line == (data.login.clone() + &data.password) {
                        self.write
                            .write_all(String::from("Successful login").as_bytes())
                            .await?;
                        break;
                    }
                }
                self.write
                    .write_all(String::from("Login failed").as_bytes())
                    .await?;
            }
            FrameData::None => {}
        }
        return Ok(());
    }
}

enum FrameType {
    Message,
    Login,
    Register,
    Error,
}

struct Login {
    login: String,
    password: String,
}

enum FrameData {
    Message(String),
    Login(Login),
    None,
}

pub struct Frame {
    frame_type: FrameType,
    data: FrameData,
}

impl Frame {
    pub fn new(input: String) -> Frame {
        let frame_vec: Vec<String> = input.split("|").map(|e| e.to_string()).collect();
        let frame: Frame;
        match frame_vec[0].as_str() {
            "message" => {
                frame = Frame {
                    frame_type: FrameType::Message,
                    data: FrameData::Message(frame_vec[1].clone()),
                };
            }
            "login" => {
                frame = Frame {
                    frame_type: FrameType::Login,
                    data: FrameData::Login(Login {
                        login: frame_vec[1].clone(),
                        password: frame_vec[2].clone(),
                    }),
                }
            }
            "register" => {
                frame = Frame {
                    frame_type: FrameType::Register,
                    data: FrameData::Login(Login {
                        login: frame_vec[1].clone(),
                        password: frame_vec[2].clone(),
                    }),
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
}
