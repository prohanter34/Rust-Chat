use std::{error::Error, net::SocketAddr, sync::Arc};
use rust_chat::App;
use tokio::{
    fs::{OpenOptions, File},
    net::{TcpListener, TcpStream},
    sync::{broadcast::{self, Receiver, Sender}, Mutex},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;

    let (tx, _rx) = broadcast::channel(20);

    let messages_data = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("rust_chat/src/data.txt")
        .await?;
    

    let users_data = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("rust_chat/src/users.txt")
        .await?;

    let messages_m = Arc::new(Mutex::new(messages_data));
    let users_m = Arc::new(Mutex::new(users_data));

    loop {

        let messages_clone = Arc::clone(&messages_m);
        let users_clone = Arc::clone(&users_m);

        let (socket, addr) = listener.accept().await?;

        let tx_clone = tx.clone();
        let rx_clone = tx_clone.subscribe();

        let _join_handle = tokio::spawn(async move {
            let _ = proceed(socket, addr, tx_clone, rx_clone, messages_clone, users_clone).await;
        });
    }

}

async fn proceed(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    rx: Receiver<(String, SocketAddr)>,
    messages_data: Arc<Mutex<File>>,
    users_data: Arc<Mutex<File>>,
) -> Result<(), Box<dyn Error>> {

    println!("thread spawn");
    let (read, write) = socket.split();
    let mut app = App::init(read, write, addr, tx, rx, messages_data, users_data).await?;
    app.start_event_loop().await?;
    Ok(())
}

