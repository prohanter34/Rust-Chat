use std::{error::Error, net::SocketAddr, rc::Rc, borrow::BorrowMut};
use rust_chat::App;
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


    let (read, write) = socket.split();
    let mut app = App::init(read, write, addr, tx, rx).await?;
    app.start_event_loop().await?;
    Ok(())
}

