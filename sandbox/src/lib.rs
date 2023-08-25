use std::{error::Error, net::SocketAddr};

use tokio::{net::{tcp::{ReadHalf, WriteHalf}, TcpStream}, fs::{File, OpenOptions}, sync::broadcast::{Sender, Receiver}, io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}};

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
        // users data ??????

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
                    //////?????    frame checker
                    println!("{}", self.read_line);
                    if result.unwrap() == 0 {
                        break Ok(());
                    }
                    self.tx.send((self.read_line.clone(), self.addr))?;
    
                    //writing history data
                    self.message_data.write_all(self.read_line.as_bytes()).await?;
                    println!("file edited");
    
                    self.read_line.clear();
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
