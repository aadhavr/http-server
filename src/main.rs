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
                let mut headers = request.lines();
                
                let request_line = headers.next().unwrap();
                let url_path = request_line.split_whitespace().nth(1).unwrap();

                if url_path == "/" {
                    let response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
                    stream.write_all(response.as_bytes()).unwrap();
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

                    // Write the response to the stream
                    stream.write_all(response.as_bytes()).unwrap();
                } else if url_path.starts_with("/echo/") {
                    let response_str = &url_path[6..]; // Extract the string after "/echo/"
                    let content_length = response_str.len();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                        content_length, response_str
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                } else {
                    // Respond with 404 Not Found for other paths
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n".to_string();
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
