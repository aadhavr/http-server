use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;
use flate2::write::GzEncoder;
use flate2::Compression;

// function to handle incoming connections
fn handle_connection(mut stream: std::net::TcpStream, directory: &str) {
    // read the request from the stream
    let request = read_request(&mut stream);
    
    // parse the request line and headers
    let (method, url_path, headers) = parse_request(&request);
    
    // check if gzip is supported by the client
    let accept_gzip = check_gzip_support(&headers);
    
    // generate and send the appropriate response
    match method {
        "GET" => handle_get(&mut stream, url_path, accept_gzip, &headers, directory),
        "POST" => handle_post(&mut stream, url_path, headers, &request, directory),
        _ => send_response(&mut stream, "HTTP/1.1 405 Method Not Allowed\r\n\r\n"),
    }
}

// function to read the request from the stream
fn read_request(stream: &mut std::net::TcpStream) -> String {
    // create a buffer to store the request data
    let mut buffer = [0; 1024];
    // read data from the stream into the buffer
    let bytes_read = stream.read(&mut buffer).unwrap();
    // convert the buffer into a string and return it
    String::from_utf8_lossy(&buffer[..bytes_read]).into_owned()
}

// function to parse the request line and headers
fn parse_request(request: &str) -> (&str, &str, Vec<&str>) {
    // split the request into lines
    let mut lines = request.lines();
    // get the request line
    let request_line = lines.next().unwrap();
    // split the request line into parts
    let mut parts = request_line.split_whitespace();
    // get the method (GET, POST, etc.)
    let method = parts.next().unwrap();
    // get the requested URL path
    let url_path = parts.next().unwrap();
    // collect the headers into a vector
    let headers: Vec<&str> = lines.collect();
    (method, url_path, headers)
}

// function to check if gzip is supported by the client
fn check_gzip_support(headers: &[&str]) -> bool {
    // iterate through the headers
    for header in headers {
        // look for the Accept-Encoding header
        if header.to_lowercase().starts_with("accept-encoding:") {
            // split the header value into encodings
            let encodings: Vec<&str> = header.split(':').nth(1).unwrap().split(',').map(|s| s.trim()).collect();
            // check if gzip is in the list of supported encodings
            if encodings.contains(&"gzip") {
                return true;
            }
        }
    }
    false
}

// function to handle GET requests
fn handle_get(stream: &mut std::net::TcpStream, url_path: &str, accept_gzip: bool, headers: &[&str], directory: &str) {
    // check if the request is for the root URL
    if url_path == "/" {
        send_response(stream, "HTTP/1.1 200 OK\r\n\r\n");
    }
    // check if the request is for the user-agent endpoint
    else if url_path == "/user-agent" {
        // find the user-agent header
        let user_agent = headers.iter()
            .find(|&&header| header.to_lowercase().starts_with("user-agent:"))
            .map(|header| header.split(':').nth(1).unwrap().trim())
            .unwrap_or("");
        
        let response_body = user_agent.as_bytes().to_vec();
        // send the user-agent as the response
        send_response_with_body(stream, "HTTP/1.1 200 OK", "text/plain", accept_gzip, &response_body);
    }
    // check if the request is for a file
    else if url_path.starts_with("/files/") {
        let filename = &url_path[7..]; // extract the filename after "/files/"
        let filepath = format!("{}/{}", directory, filename);

        if Path::new(&filepath).exists() {
            // read the file contents
            let mut file = File::open(&filepath).unwrap();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            
            // send the file contents as the response
            send_response_with_body(stream, "HTTP/1.1 200 OK", "application/octet-stream", accept_gzip, &contents);
        } else {
            send_response(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
        }
    } else if url_path.starts_with("/echo/") {
        let response_str = &url_path[6..]; // extract the string after "/echo/"
        let response_body = response_str.as_bytes().to_vec();
        
        // send the echoed string as the response
        send_response_with_body(stream, "HTTP/1.1 200 OK", "text/plain", accept_gzip, &response_body);
    } else {
        send_response(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}

// function to handle POST requests
fn handle_post(stream: &mut std::net::TcpStream, url_path: &str, headers: Vec<&str>, request: &str, directory: &str) {
    if url_path.starts_with("/files/") {
        let filename = &url_path[7..]; // extract the filename after "/files/"
        let filepath = format!("{}/{}", directory, filename);

        // find the content length from the headers
        let content_length = headers.iter()
            .find(|&&header| header.to_lowercase().starts_with("content-length:"))
            .and_then(|header| header.split(':').nth(1).unwrap().trim().parse().ok())
            .unwrap_or(0);

        // calculate how much of the body is already read
        let body_start = request.find("\r\n\r\n").unwrap() + 4;
        let already_read_body = &request.as_bytes()[body_start..];

        // read the rest of the request body if not fully read
        let mut body = Vec::from(already_read_body);
        if body.len() < content_length {
            let mut remaining_body = vec![0; content_length - body.len()];
            stream.read_exact(&mut remaining_body).unwrap();
            body.extend_from_slice(&remaining_body);
        }

        // write the request body to the file
        let mut file = OpenOptions::new().write(true).create(true).open(&filepath).unwrap();
        file.write_all(&body).unwrap();

        // respond with 201 Created
        send_response(stream, "HTTP/1.1 201 Created\r\n\r\n");
    } else {
        send_response(stream, "HTTP/1.1 405 Method Not Allowed\r\n\r\n");
    }
}

// function to send a response with a body
fn send_response_with_body(stream: &mut std::net::TcpStream, status: &str, content_type: &str, accept_gzip: bool, body: &[u8]) {
    let mut response = format!("{}\r\nContent-Type: {}\r\n", status, content_type);
    let mut response_body = body.to_vec();

    if accept_gzip {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(body).unwrap();
        response_body = encoder.finish().unwrap();
        response.push_str("Content-Encoding: gzip\r\n");
    }

    response.push_str(&format!("Content-Length: {}\r\n\r\n", response_body.len()));
    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&response_body).unwrap();
}

// function to send a simple response without a body
fn send_response(stream: &mut std::net::TcpStream, response: &str) {
    stream.write_all(response.as_bytes()).unwrap();
}

// main function to start the server
fn main() {
    // get the directory from command-line arguments
    let args: Vec<String> = env::args().collect();
    let directory = match args.iter().position(|arg| arg == "--directory") {
        Some(index) => &args[index + 1],
        None => {
            eprintln!("Warning: --directory flag not provided. Using current directory as default.");
            "."
        }
    };

    // bind the listener to the address and port
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    // accept incoming connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                let directory = directory.to_string();
                // spawn a new thread to handle the connection
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
