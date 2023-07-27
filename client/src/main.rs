use std::{error::Error, io::stdin};

use tokio::{net::TcpStream, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}, sync::mpsc};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let mut socket = TcpStream::connect("localhost:8000").await?;

    let (read, mut write) = socket.split();

    let mut reader = BufReader::new(read);
    let mut line = String::new();

    let (tx, mut rx) = mpsc::channel(10);

    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let mut input = String::new();
        loop {
            stdin().read_line(&mut input).expect("msg");
            tx_clone.send(input.clone()).await.unwrap();
            input.clear();
        }
    });


    loop {
        
        tokio::select! {
            _ = reader.read_line(&mut line) => {
                println!("{}", line.trim());
                line.clear();
            },
            result = rx.recv() => {
                let message = result.unwrap();
                if message.trim() != String::from("") {
                    write.write_all(message.as_bytes()).await.expect("msg");

                }
            }
        };
        
    }

}
