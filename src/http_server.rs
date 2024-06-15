use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;

#[derive(Debug)]
struct Response {
    status_line: String,
    content_type: ContentType,
    content: Vec<u8>,
}

impl Response {
    fn new() -> Response {
        Response {
            status_line: "".to_string(),
            content_type: ContentType::Html,
            content: Vec::new(),
        }
    }

    fn as_bytes(self) -> Vec<u8> {
        let mut response = Vec::new();
        let status_line = self.status_line.as_bytes();
        for b in status_line {
            response.push(*b);
        }
        response.push(b'\n');
        let binding = self.content_type.to_string();
        let content_type = binding.as_bytes();
        for b in content_type {
            response.push(*b)
        }
        response.push(b'\n');
        let content_length = format!("Content-Length: {}\r\n\r\n", self.content.len());
        let content_length = content_length.as_bytes();
        for b in content_length {
            response.push(*b);
        }
        for b in self.content {
            response.push(b);
        }
        for b in "\r\n\r\n".as_bytes() {
            response.push(*b);
        }
        response
    }
}

#[derive(Debug)]
enum ContentType {
    Html,
    Pdf,
    Css,
}

impl ContentType {
    fn to_string(&self) -> String {
        match self {
            ContentType::Html => "Content-Type: text/html".to_string(),
            ContentType::Pdf => "Content-Type: application/pdf".to_string(),
            ContentType::Css => "Content-Type: text/css".to_string(),
        }
    }
}

enum StatusCode {
    Ok,
    NotFound,
}

impl StatusCode {
    fn as_str(self) -> String {
        match self {
            StatusCode::Ok => "HTTP/1.1 200 OK".to_string(),
            StatusCode::NotFound => "HTTP/1.1 404 Not Found".to_string(),
        }
    }
}
struct FileContents {
    content_type: ContentType,
    contents: Vec<u8>,
}

fn fetch_file(path: &Path) -> io::Result<FileContents> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => {
            let mut file = File::open(path)?;
            let mut contents: Vec<u8> = Vec::new();
            file.read_to_end(&mut contents)?;
            let file_contents = FileContents {
                content_type: ContentType::Pdf,
                contents,
            };
            Ok(file_contents)
        }
        Some("html") => {
            let mut file = File::open(path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            let file_contents = FileContents {
                content_type: ContentType::Html,
                contents,
            };
            Ok(file_contents)
        }
        Some("css") => {
            let mut file = File::open(path)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            let file_contents = FileContents {
                content_type: ContentType::Css,
                contents,
            };
            Ok(file_contents)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unsupported file extension",
        )),
    }
}

pub fn handle_request(mut stream: TcpStream, folder_path: &String) {
    let reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let req_path = &http_request[0]
        .split_ascii_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    let abs_path = folder_path.to_owned() + req_path;
    let response_data = fetch_file(Path::new(&abs_path));
    let mut response = Response::new();
    match response_data {
        Ok(contents) => {
            response.status_line = StatusCode::Ok.as_str();
            response.content = contents.contents;
            response.content_type = contents.content_type;
        }
        Err(_) => {
            response.status_line = StatusCode::NotFound.as_str();
            response.content = get_404_page(folder_path);
            response.content_type = ContentType::Html;
        }
    }
    let response = response.as_bytes();
    stream.write_all(&response).unwrap()
}

fn get_404_page(folder_path: &String) -> Vec<u8> {
    let path = folder_path.to_owned() + "notfound.html";
    let path = Path::new(&path);
    let fetched = fetch_file(path).unwrap();
    fetched.contents
}
