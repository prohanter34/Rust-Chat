use std::{error::Error, net::SocketAddr};

use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("localhost:8000").await?;

    let (tx, _rx) = broadcast::channel(20);

    loop {
        let (socket, addr) = listener.accept().await?;

        let tx_clone = tx.clone();
        let rx_clone = tx_clone.subscribe();

        let _join_handle = tokio::spawn(async move {
            let _ = proceed(socket, addr, tx_clone, rx_clone).await;
        });
    }

    // Ok(())
}

async fn proceed(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    mut rx: Receiver<(String, SocketAddr)>,
) -> Result<(), Box<dyn Error>> {
    println!("thread spawn");

    let (read, mut writer) = socket.split();

    //reading history data
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("Rust-Chat/src/data.txt")
        .await?;
    let mut data = String::new();
    file.read_to_string(&mut data).await?;
    writer.write_all(data.as_bytes()).await?;
    //
    let mut reader = BufReader::new(read);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                println!("{}", line);
                if result.unwrap() == 0 {
                    break Ok(());
                }
                tx.send((line.clone(), addr))?;

                //writing history data
                file.write_all(line.as_bytes()).await?;
                println!("file edited");

                line.clear();
                //
            },
            result = rx.recv() => {

                let (message, some_addr) = result.unwrap();
                if some_addr != addr {
                    
                    writer.write_all(message.as_bytes()).await?;

                }
                line.clear();

            }
        };
    }
}
