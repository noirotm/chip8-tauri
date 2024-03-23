// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use chip8_system::display::DisplayMessage;
use chip8_system::keyboard::KeyboardMessage;
use chip8_system::keyboard_map::KeyboardMap;
use chip8_system::port;
use chip8_system::port::{InputPort, OutputPort};
use chip8_system::system::System;
use crossbeam_channel::{Receiver, Sender};
use sound_cpal::Beeper;
use std::path::Path;
use std::thread;
use tauri::api::dialog::FileDialogBuilder;
use tauri::{AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Runtime, State, Submenu, Window};

#[derive(Clone, serde::Serialize)]
struct DrawEventPayload {
    pixels: Vec<bool>,
}

struct Screen {
    display_sender: Sender<DisplayMessage>,
}

impl Screen {
    fn new<R: Runtime>(app: AppHandle<R>) -> Self {
        let (ds, dr) = crossbeam_channel::bounded(128);

        thread::spawn(move || {
            while let Ok(msg) = dr.recv() {
                let w = app.get_window("main").expect("main window");
                match msg {
                    DisplayMessage::Clear => {
                        w.emit("clear", ()).unwrap();
                    }
                    DisplayMessage::Update(b) => {
                        w.emit(
                            "draw",
                            DrawEventPayload {
                                pixels: b.iter().by_vals().collect(),
                            },
                        )
                        .unwrap();
                    }
                }
            }
        });

        Self { display_sender: ds }
    }
}

impl InputPort<DisplayMessage> for Screen {
    fn input(&self) -> Sender<DisplayMessage> {
        self.display_sender.clone()
    }
}

struct Keyboard {
    keyboard_receiver: Receiver<KeyboardMessage>,
}

impl OutputPort<KeyboardMessage> for Keyboard {
    fn output(&self) -> Receiver<KeyboardMessage> {
        self.keyboard_receiver.clone()
    }
}

#[tauri::command]
fn key_down(key: &str, state: State<AppState>) {
    if let Some(key) = state.keyboard_map.key(key) {
        _ = state.keyboard_sender.try_send(KeyboardMessage::down(key));
    }
}

#[tauri::command]
fn key_up(key: &str, state: State<AppState>) {
    if let Some(key) = state.keyboard_map.key(key) {
        _ = state.keyboard_sender.try_send(KeyboardMessage::up(key));
    }
}

fn build_menu() -> Menu {
    Menu::new().add_submenu(Submenu::new(
        "File",
        Menu::new()
            .add_item(CustomMenuItem::new("load".to_string(), "Load..."))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("quit".to_string(), "Quit")),
    ))
}

fn load_image<R: Runtime>(kb_receiver: Receiver<KeyboardMessage>, window: &Window<R>) {
    let app = window.app_handle();
    FileDialogBuilder::new()
        .set_title("Load CHIP-8 ROM")
        .set_parent(window)
        .add_filter("CHIP-8 ROMs", &["c8", "ch8"])
        .pick_file(move |p| {
            if let Some(p) = p {
                let r = run_chip8(p, kb_receiver, app);

                if r.is_err() {}
            }
        });
}

fn run_chip8<P: AsRef<Path>, R: Runtime>(
    path: P,
    kb_receiver: Receiver<KeyboardMessage>,
    app: AppHandle<R>,
) -> anyhow::Result<()> {
    let mut chip8 = System::new();

    let screen = Screen::new(app);
    port::connect(&chip8.display, &screen);

    let beep = Beeper::new().map_err(|e| anyhow!("{}", e))?;
    port::connect(&chip8.sound_timer, &beep);

    let keyboard = Keyboard {
        keyboard_receiver: kb_receiver,
    };
    port::connect(&keyboard, &chip8.keyboard);

    chip8.load_image(path)?;
    chip8.run().map_err(|e| e.into())
}

struct AppState {
    keyboard_sender: Sender<KeyboardMessage>,
    keyboard_map: KeyboardMap,
}

fn main() {
    let (kb_sender, kb_receiver) = crossbeam_channel::bounded::<KeyboardMessage>(128);

    tauri::Builder::default()
        .menu(build_menu())
        .manage(AppState {
            keyboard_sender: kb_sender,
            keyboard_map: Default::default(),
        })
        .on_menu_event(move |event| match event.menu_item_id() {
            "load" => load_image(kb_receiver.clone(), event.window()),
            "quit" => {
                event.window().close().unwrap();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![key_down, key_up])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
