use tokio::fs::File;

pub async fn frame_checker(frame: String, message_file: File, users_file: File) {
    let frame: Vec<String> = frame.split("$").map(|e| e.to_string()).collect();
    
    if frame[0] == "message".to_string() {
        message_file.write_all(frame[0].clone().as_bytes()).await.unwrap();
    } else if frame[0] == "login" {
        let mut line = String::new();
        users_file.read_to_string(&mut line);

    }
}