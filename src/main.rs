use std::io::{
    Read,
    Write
};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buffer = [0; 512];
                stream.read(&mut buffer).unwrap();

                let request = String::from_utf8_lossy(&buffer[..]);
                let request_line = request.lines().next().unwrap();

                let url_path = request_line.split_whitespace().nth(1).unwrap();

                let response = if url_path == "/" {
                    "HTTP/1.1 200 OK\r\n\r\n"
                } else {
                    "HTTP/1.1 404 Not Found\r\n\r\n"
                };

                let _ = stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
