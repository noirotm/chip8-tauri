// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bitvec::vec::BitVec;
use chip8_system::display::DisplayMessage;
use chip8_system::port;
use chip8_system::port::InputPort;
use chip8_system::system::System;
use crossbeam_channel::Sender;
use std::thread;
use tauri::api::dialog::FileDialogBuilder;
use tauri::{AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Submenu};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
/*#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}*/

struct AppState {
    chip8: System,
    screen: Screen,
    //kb: Keyboard,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    pixels: Vec<bool>,
}

struct Screen {
    display_sender: Sender<DisplayMessage>,
}

impl Screen {
    fn new(h: AppHandle) -> Self {
        let (ds, dr) = crossbeam_channel::bounded(128);

        thread::spawn(move || {
            while let Ok(msg) = dr.recv() {
                if let Some(w) = h.get_window("main") {
                    match msg {
                        DisplayMessage::Clear => {
                            w.emit("clear", ()).unwrap();
                        }
                        DisplayMessage::Update(b) => {
                            w.emit(
                                "draw",
                                Payload {
                                    pixels: bitvec_to_pixels(&b),
                                },
                            )
                            .unwrap();
                        }
                    }
                }
            }
        });

        Self { display_sender: ds }
    }
}

fn bitvec_to_pixels(b: &BitVec) -> Vec<bool> {
    b.iter().by_vals().collect()
}

impl InputPort<DisplayMessage> for Screen {
    fn input(&self) -> Sender<DisplayMessage> {
        self.display_sender.clone()
    }
}

//struct Keyboard;

fn build_menu() -> Menu {
    Menu::new().add_submenu(Submenu::new(
        "File",
        Menu::new()
            .add_item(CustomMenuItem::new("load".to_string(), "Load..."))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
    ))
}

fn main() {
    tauri::Builder::default()
        .menu(build_menu())
        .on_menu_event(|event| match event.menu_item_id() {
            "load" => {
                FileDialogBuilder::new()
                    .set_title("Load CHIP-8 ROM")
                    .set_parent(event.window())
                    .add_filter("CHIP-8 ROMs", &["c8", "ch8"])
                    .pick_file(move |p| {
                        if let Some(p) = p {
                            let mut app = AppState {
                                chip8: System::new(),
                                screen: Screen::new(event.window().app_handle()),
                                //kb: Keyboard,
                            };

                            port::connect(&app.chip8.display, &app.screen);

                            if app.chip8.load_image(&p).is_ok() {
                                _ = app.chip8.run();
                            }
                        }
                    });
            }
            "quit" => {
                event.window().close().unwrap();
            }
            _ => {}
        })
        //.invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
