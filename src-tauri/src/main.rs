#![deny(warnings)]
#![warn(rust_2018_idioms)]
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod commands;
pub mod rest;

use commands::send_request;

fn main() {
    println!("Hello world");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
