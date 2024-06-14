use std::{
    env, fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    process::{self},
};

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

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
        handle_connection(stream, folder_path)
    }
}

fn handle_connection(mut stream: TcpStream, folder_path: &String) {
    let reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("{http_request:#?}");
    if http_request.len() == 0 {
        return;
    }
    let req_path = &http_request[0]
        .split_ascii_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    println!("{}", req_path);

    let status_line;
    let contents = match fs::read_to_string(folder_path.to_owned() + req_path) {
        Ok(contents) => {
            status_line = "HTTP 200 OK";
            contents
        }
        Err(_) => {
            status_line = "HTTP 404 Not Found";
            fs::read_to_string(folder_path.to_owned() + "/notfound.html").unwrap()
        }
    };
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
