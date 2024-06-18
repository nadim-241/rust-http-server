use std::{
    env,
    net::TcpListener,
    process::{self},
};

use http_server::{handle_request, ThreadPool};

mod http_server;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: webserver.exe WEBSITE_FOLDER_NAME");
        process::exit(1);
    }

    let target_folder = &args[1];
    start_http_server(target_folder, "127.0.0.1:7878".to_string())
}

fn start_http_server(folder_path: &String, address: String) {
    let listener = TcpListener::bind(&address).unwrap();
    let thread_pool = ThreadPool::new(15);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let folder_path = folder_path.to_owned();
        thread_pool.execute(|| handle_request(stream, folder_path));
    }
}
