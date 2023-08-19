use std::{error::Error, net::SocketAddr};
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    let (tx, _rx) = broadcast::channel(20);

    loop {
        let (socket, addr) = listener.accept().await?;

        let tx_clone = tx.clone();
        let rx_clone = tx_clone.subscribe();

        let _join_handle = tokio::spawn(async move {
            let _ = proceed(socket, addr, tx_clone, rx_clone).await;
        });
    }

}

async fn proceed(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    mut rx: Receiver<(String, SocketAddr)>,
) -> Result<(), Box<dyn Error>> {

    // initialization client

    println!("thread spawn");

    let (read, mut writer) = socket.split();

    //reading history data
    let mut file_data = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("Rust-Chat/src/data.txt")
        .await?;
    let mut data = String::new();
    file_data.read_to_string(&mut data).await?;
    writer.write_all(data.as_bytes()).await?;
    // user data
    let mut users_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("Rust-Chat/src/users.txt")
        .await?;
    let mut users = String::new();
    users_file.read_to_string(&mut users).await.unwrap();
    //
    let mut reader = BufReader::new(read);
    let mut line = String::new();

    // event loop

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                //////?????    frame checker
                println!("{}", line);
                if result.unwrap() == 0 {
                    break Ok(());
                }
                tx.send((line.clone(), addr))?;

                //writing history data
                file_data.write_all(line.as_bytes()).await?;
                println!("file edited");

                line.clear();
                //
            },
            result = rx.recv() => {

                let (message, some_addr) = result.unwrap();
                if some_addr != addr {
                    ////////////???? frame creator
                    writer.write_all(message.as_bytes()).await?;

                }
                line.clear();

            }
        };
    }
}

