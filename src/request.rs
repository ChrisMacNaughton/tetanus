use std::net::TcpStream;
use std::io::prelude::*;
use std::str;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub stream: TcpStream,
    pub response: Response,
    pub path: String,
    pub method: String,
    pub host: String,
    pub headers: HashMap<String, String>
}

#[derive(Debug)]
pub struct Response {
    pub body: String
}

impl Request {
    pub fn new(mut stream: TcpStream) -> Request {
        let mut buf = [0; 1024];
        stream.read(&mut buf).unwrap();
        let s = str::from_utf8(&buf).unwrap();
        println!("{}", s);
        let mut headers = HashMap::new();
        Request {
            stream: stream,
            response: Response::new(),
            path: String::new(),
            method: String::new(),
            host: String::new(),
            headers: headers,
        }
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            body: String::new()
        }
    }
    pub fn to_response(&self) -> &[u8] {
        self.body.as_bytes()
    }
}