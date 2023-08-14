use std::error::Error;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:8000").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        let _join_handle = tokio::spawn(async move {
            let _ = proceed(socket).await;
        });
    }
}

async fn proceed(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("thread spawn");

    let (read, mut writer) = socket.split();

    let mut reader = BufReader::new(read);
    let mut line = String::new();

    loop {
        reader.read_line(&mut line).await?;
        writer.write_all(line.as_bytes()).await?;
        line.clear();
    }
}

