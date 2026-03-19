use std::io::{BufRead, BufReader, Error, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub struct HttpResponse {
    pub status: u16,
}

pub fn run(port: u16, timeout: Duration) -> Result<(), Error> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("Server listening on port {port}");

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(move || {
            handle_connection(stream, timeout)
        });
    }

    Ok(())
}

pub fn handle_connection(stream: TcpStream, timeout_duration: Duration) -> Result<(), Error> {
    eprintln!("Connection from {}", stream.peer_addr()?);
    stream.set_read_timeout(Some(timeout_duration))?;
    //stream.set_write_timeout(Some(timeout_duration))?;

    let mut reader = BufReader::new(&stream);
    let mut request = String::new();

    for line in reader.lines() {
        let line = line?;
        request.push_str(&line);
        request.push('\n');

        if line.is_empty() {
            break;
        }
    }

    parse_request(request)?;

    Ok(())
}

/*
Received request:
GET / HTTP/1.1
Host: localhost:8000
Connection: keep-alive
Accept-Encoding: gzip, deflate, br, zstd
Accept-Language: tr
Sec-Fetch-Dest: empty
Sec-Fetch-Mode: cors
Sec-Fetch-Site: cross-site
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Scalar/0.1.252 Chrome/140.0.7339.133 Electron/38.2.1 Safari/537.36
accept: /
sec-ch-ua: "Not=A?Brand";v="24", "Chromium";v="140"
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "macOS"
*/

pub fn parse_request(request: String) -> Result<(HttpRequest), Error> {
    println!("Received request:\n{request}");
    let mut lines = request.lines();
    eprintln!("Request:\n {}", lines.next().unwrap());
    let response = HttpRequest {
        method: String::new(),
        path: String::new(),
        headers: vec![(String::new(), String::new())],
        body: String::new(),
    };

    Ok(response)
}

pub fn get_method(line: String) -> Result<String, Error> {
    fn first_word(s: &str) -> &str {
        let bytes = s.as_bytes(); // turn into byte array

        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] == b' ' { // space found
                return &s[..index];   // return slice
            }

            index += 1;
        }

        &s[..] // no space → whole string
    }

    let method = first_word(&line).to_string();

    Ok(method)
}
