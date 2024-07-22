use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;

fn handle_connection(mut stream: std::net::TcpStream, directory: &str) {
    // Buffer to store the request
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    // Convert buffer to string
    let request = String::from_utf8_lossy(&buffer[..]);
    let mut headers = request.lines();

    // Extract the request line
    let request_line = headers.next().unwrap();
    let url_path = request_line.split_whitespace().nth(1).unwrap();

    // Determine the response
    if url_path.starts_with("/files/") {
        let filename = &url_path[7..]; // Extract the filename after "/files/"
        let filepath = format!("{}/{}", directory, filename);

        if Path::new(&filepath).exists() {
            let mut file = File::open(&filepath).unwrap();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            let content_length = contents.len();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                content_length
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&contents).unwrap();
        } else {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
            stream.write_all(response.as_bytes()).unwrap();
        }
    } else if url_path == "/user-agent" {
        // Initialize the User-Agent header value
        let mut user_agent = "";

        // Loop through the headers to find the User-Agent header
        for header in headers {
            if header.to_lowercase().starts_with("user-agent:") {
                user_agent = header.split(':').nth(1).unwrap().trim();
                break;
            }
        }

        // Construct the response
        let content_length = user_agent.len();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            content_length, user_agent
        );

        stream.write_all(response.as_bytes()).unwrap();
    } else if url_path.starts_with("/echo/") {
        let response_str = &url_path[6..]; // Extract the string after "/echo/"
        let content_length = response_str.len();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            content_length, response_str
        );
        stream.write_all(response.as_bytes()).unwrap();
    } else if url_path == "/" {
        let response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        // Respond with 404 Not Found for other paths
        let response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    // Parse the --directory flag from command-line arguments
    let args: Vec<String> = env::args().collect();
    let directory = match args.iter().position(|arg| arg == "--directory") {
        Some(index) => &args[index + 1],
        None => {
            eprintln!("Warning: --directory flag not provided. Using current directory as default.");
            "."
        }
    };

    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let directory = directory.to_string();
                // Spawn a new thread to handle the connection
                thread::spawn(move || {
                    handle_connection(stream, &directory);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
