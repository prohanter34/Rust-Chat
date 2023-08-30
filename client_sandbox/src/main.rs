#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{ScrollArea, Vec2, Pos2};
use std::{
    sync::{mpsc::{Sender, Receiver}, self},
    thread, net::TcpStream, io::{prelude::*, BufReader},
};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Rust chat",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    login: String,
    password: String,
    message_input: String,
    messages: Vec<String>,
    login_flag: bool,
    connection_error: bool,
    tx: Sender<String>,
    rx: Receiver<String>
    
}

impl Default for MyApp {
    fn default() -> Self {

        let (tx, rx) = sync::mpsc::channel::<String>();
        let (tx2, rx2) = sync::mpsc::channel();

        let _ = thread::spawn(move || {

            let mut socket = TcpStream::connect("127.0.0.1:8000").expect("connection error");
            let socket_clone = socket.try_clone().unwrap();

            let _ = thread::spawn(move || {
                let mut reader = BufReader::new(socket_clone);
                let mut buf = String::new();

                loop {
                    
                    let result = reader.read_line(&mut buf).unwrap();
                    if result != 0 {
                        tx2.send(buf.clone()).unwrap();
                    }
                    buf.clear();
                }
            });

            loop {
                
                let result = rx.try_recv();

                match result {
                    Ok(frame) => {
                        let message = frame.trim().to_string() + &"\n".to_string();
                        socket.write_all(message.as_bytes()).expect("writing error");
                    },
                    Err(_) => {continue;}
                };

            }

        });

        Self {
            login: String::new(),
            password: String::new(),
            message_input: String::new(),
            messages: Vec::new(),
            login_flag: true,
            connection_error: false,
            tx,
            rx: rx2,
            
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.login_flag {
            egui::Window::new("Login please")
                .collapsible(false)
                .resizable(false)
                .current_pos(Pos2::new(120.0, 110.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        
                        ui.vertical(|ui| {
                            ui.text_edit_singleline(&mut self.login);
                            ui.text_edit_singleline(&mut self.password);
                            
                        });
                        if ui.button("Ok").clicked() {
                            if self.login != "" && self.password != "" {
    
                                self.login_flag = false;
                            }
                        }
                    });
                });
        } else if self.connection_error {
            egui::Window::new("Connection error")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Connection error");
                    });
                });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                // ui.vertical_centered(|ui| {
                let _scroll = ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for i in self.messages.iter() {
                        ui.label(i);
                    }
                    ui.allocate_space(ui.available_size());
                });
                

                ui.allocate_space(ui.available_size() - Vec2::new(1.0, 20.0));

                ui.horizontal(|ui| {
                    let _input_line = ui.text_edit_singleline(&mut self.message_input);
                    if ui.button("Send").clicked() {
                        //create frame and send it ???? 
                        self.tx.send(self.message_input.clone()).unwrap();
                        self.messages.push(self.message_input.clone() + "\n");
                        self.message_input.clear();                              
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        //create frame and send it ????
                        self.tx.send(self.message_input.clone()).unwrap();
                        self.messages.push(self.message_input.clone() + "\n");
                        self.message_input.clear();
                    }
                    if self.messages.len() > 10  {
                        self.messages.remove(0);
                    }
                });

                // });
            });
        }

        let result = self.rx.try_recv();
        match result {
            Ok(data) => self.messages.push(data),
            Err(_) => {},
        }

    }
}

fn frame_creator(frame: FrameType) -> String {
    match frame {
        FrameType::message(data) => {
            return format!("message|{}", data);
        },
        FrameType::login(login, password) => {
            return format!("login|{}|{}", login, password);
        },
    }
}

enum FrameType {
    message(String),
    login(String, String),
}