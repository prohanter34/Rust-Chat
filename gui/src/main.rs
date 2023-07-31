#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::ScrollArea;
use tokio::{net::TcpStream, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}, sync::mpsc};
use std::{sync::{Mutex, Arc}, time::Duration};


#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 440.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    input: String,
    data_m: Arc<Mutex<Vec<String>>>,
    input_m: Arc<Mutex<String>>,
}

impl Default for MyApp {
    fn default() -> Self {
        let input_m = Arc::new(Mutex::new(String::new()));
        let input_clone = Arc::clone(&input_m); 
        let data_m: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let data_clone = Arc::clone(&data_m);
        tokio::spawn(async move {
            let mut socket = TcpStream::connect("localhost:8000").await.unwrap();

            let (read, mut write) = socket.split();
            let mut reader = BufReader::new(read);
            let mut buf = String::new();
            let mut data = Vec::new();

            let data_mm = Arc::clone(&input_clone);
            let (tx, mut rx) = mpsc::channel(5);
            tokio::spawn(async move {
                loop {
                            
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    if (*(data_mm.lock().unwrap())) != "" {
                        let message = (*(data_mm.lock().unwrap())).clone();
                        tx.send(message).await.unwrap();
                        (*(data_mm.lock().unwrap())).clear();
                    }
                    
                }
            });


            loop {
                tokio::select! {
                    _ = reader.read_line(&mut buf) => {
                        (*(data_clone.lock().unwrap())).push(buf.clone());
                        data.push(buf.clone());
                        buf.clear();
                
                    },
                    result = rx.recv() => {
                        let message = result.unwrap();
                        if message.trim() != String::from("") {
                            let _a = write.write(message.as_bytes()).await.expect("msg");
        
                        }
                    }
                };


            }

        });


        Self {
            input: String::new(),
            data_m,
            input_m,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {




        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.vertical_centered(|ui| {
            let _scrollarea = ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for i in (*self.data_m.lock().unwrap()).iter() {
                    ui.label(i);
                }
            });

            ui.horizontal(|ui| {
                let _input_line = ui.text_edit_singleline(&mut self.input);
                if ui.button("Send").clicked() {
                    self.input += "\n";
                    (*self.data_m.lock().unwrap()).push(self.input.clone());
                    (*(self.input_m.lock().unwrap())) = self.input.clone();
                    self.input = String::new();
                }
            });

            // });
        });
    }
}
