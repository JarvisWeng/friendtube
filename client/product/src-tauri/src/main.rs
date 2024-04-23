// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dashmap::DashMap;
use shared::metadata::{ VideoChunk, VideoId, ChunkId };
use std::sync::{ Arc, Mutex };
use std::thread;
use local_ip_address::local_ip;
use std::net::IpAddr;
use rand::Rng;

mod shared;
mod server_connection_manager;
mod buffer_manager;
mod seed_manager;
mod leech_manager;
mod video_server;

struct PortState {
    port: Mutex<u16>,
}

//get IP of the video server
#[tauri::command]
fn get_port(port_state: tauri::State<'_, PortState>) -> u16 {
    println!("Running: {}", "get_ip");
    let p = port_state.port.lock().unwrap();
    return *p;
}

fn main() {
    let mut rng = rand::thread_rng();
    let port = rng.gen_range(3031..10000);

    let seed_port = rng.gen_range(3001..3030);

    // Instantiating buffer manager
    let state = PortState {
        port: Mutex::new(port.clone()),
    };

    let bm = Arc::new(buffer_manager::BufferManager::new());

    //start seed_manager -> only needs to interact with the buffer manager
    let t = bm.clone();
    let res1 = thread::Builder::new().name("seed manager".to_string()).spawn(move || {
        let mut sm = seed_manager::SeedManager::new(local_ip().unwrap(), seed_port, t);
        sm.run();
    }).unwrap();

    //start video server - > needs leech manager for getting videos from peer
    //                   - > needs server connection manager for getting videos from server
    //                   - > needs buffer manager to get videos from local cache
    let t = bm.clone();
    let res2 = thread::Builder::new().name("video server".to_string()).spawn(move || {
            let addr = "127.0.0.1".parse::<IpAddr>().unwrap();
            let mut vs = video_server::VideoServer::new(addr, t, port, seed_port);
            vs.run();
        }).unwrap();

    tauri::Builder
        ::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    res1.join();
    res2.join();
}
