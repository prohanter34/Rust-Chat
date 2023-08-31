use std::{error::Error, net::SocketAddr, sync::Arc};

use tokio::{net::tcp::{ReadHalf, WriteHalf}, fs::{File, OpenOptions}, sync::{broadcast::{Sender, Receiver}, Mutex}, io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}};

pub struct App<'a> {
    reader: BufReader<ReadHalf<'a>>,
    read_line: String,
    write: WriteHalf<'a>,
    addr: SocketAddr,
    message_data: Arc<Mutex<File>>,
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
        message_data: Arc<Mutex<File>>,
    ) -> Result<App<'a>, Box<dyn Error>> {
   
        let reader = BufReader::new(read);
        let line = String::new();
        
        // message history
        let mut messages_file = File::open("rust_chat/src/data.txt").await?;
        let mut data = String::new();
        messages_file.read_to_string(&mut data).await?;
        write.write_all(data.as_bytes()).await?;
        // users data
        let users_file = OpenOptions::new()
            .read(true)
            .append(true)
            .open("rust_chat/src/users.txt").await?;
        //
        let app: App<'a> = App {
            reader,
            read_line: line,
            write,
            addr,
            message_data,
            tx,
            rx,
            users_data: users_file,
        };
        return Ok(app);
    }

    pub async fn handle(&mut self, frame: Frame) -> Result<(), Box<dyn Error>> {
        match frame.data {
            FrameData::Message(data) => {
                    self.tx.send((data.clone(), self.addr))?;
    
                    //writing history data
                    (self.message_data.lock().await).write_all(data.as_bytes()).await.unwrap();
                    println!("file edited");
    
                    self.read_line.clear();
                    Ok(())
            },
            FrameData::Login(data) => {
                let mut line = String::new();
                let mut flag = true;
                self.users_data.read_to_string(&mut line).await?;
                for i in line.lines() {
                    if i == data.login.clone() + "|" + &data.password {
                        self.write.write_all("Login successful\n".as_bytes()).await?;
                        flag = false;
                        break;
                    }

                }
                if flag {
                    self.write.write_all("Login failed\n".as_bytes()).await?;
                }
                Ok(())
            },
            FrameData::None => {
                println!("frame error");
                Ok(())
            },
        }
    }

    pub async fn start_event_loop(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            tokio::select! {
                result = self.reader.read_line(&mut self.read_line) => {
                    self.read_line = self.read_line.trim().to_string() + &"\n".to_string();
                    if result.unwrap() == 0 {
                        break Ok(());
                    }
                    //////?????    frame checker
                    let frame = frame_check(self.read_line.clone());
                    self.read_line.clear();
                    println!("{:?}", frame);
                    self.handle(frame).await?;
                },
                result = self.rx.recv() => {
    
                    let (message, some_addr) = result.unwrap();
                    if some_addr != self.addr {
                        ////////////???? frame creator
                        println!("{}", message);
                        self.write.write_all(message.as_bytes()).await?;
    
                    }
    
                }
            };
        }
    }
}


#[derive(Debug)]
enum FrameType {
    Message,
    Login,
    Register,
    Error
}

#[derive(Debug)]
struct Login {
    login: String,
    password: String
}

#[derive(Debug)]
enum FrameData {
    Message(String),
    Login(Login),
    None
}

#[derive(Debug)]
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
            let password = frame_vec[2].clone();
            frame = Frame {
                frame_type: FrameType::Login,
                data: FrameData::Login(Login { login: frame_vec[1].clone(), password: password.replace("\n", "") }),
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
