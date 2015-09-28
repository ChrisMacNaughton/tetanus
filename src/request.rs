extern crate regex;

use std::net::TcpStream;
use std::io::prelude::*;
use std::str;
use std::collections::HashMap;
use self::regex::Regex;
#[derive(Debug)]
pub struct Request {
    pub stream: TcpStream,
    pub response: Response,
    pub path: String,
    pub method: String,
    pub host: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Response {
    pub body: String,
}

pub struct Header {
    pub name: String,
    pub value: String,
}

impl Request {
    pub fn new(mut stream: TcpStream) -> Request {
        let mut buf = [0; 1024];
        stream.read(&mut buf).unwrap();
        let s = str::from_utf8(&buf).unwrap();
        let headers = parse_headers(s.to_string());
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

fn parse_headers(request: String) -> HashMap<String, String> {
    println!("{}", request);
    // let request_regex = Regex::new(r"\A(.+)$").unwrap();
    // let header_regex = rRegex::new(r"").unwrap;
    let mut headers = HashMap::new();
    for item in request.split('\n') {
        // println!("{}", item);
        // let header = parse_header(item.to_string());
        match parse_header(item.to_string()) {
            Some(header) => {
                headers.insert(header.name, header.value);
            },
            None => println!("failed to parse {}", item),
        };
        // println!("header: {}\nhas value: {}", name, value);
    }
    println!("\n\nHeaders:\n\n{:?}", headers);
    // let parts = request.as_slice().split(',').collect();
    headers
}

fn parse_header(header: String) -> Option<Header> {
    let parts: Vec<&str> = header.split(": ").collect();
    // let tail = parts.tail();
    // println!("{:?}", head);
    let header_name = match parts.first() {
        Some(header_name) => header_name.to_string(),
        None => "".to_string(),
    };

    // let tail = parts.split_first().into_iter();

    // let header_value = match tail.into_iter() {
    //     Some(header_value) => header_value.collect(),
    //     None => "".to_string(),
    // };
    // let header_value = match parts.tail() {
    //     Some(header_value) => {
    //         let header_value = match header_value.into_iter().collect() {
    //             Some(header_value) => header_value.to_string(),
    //             None => "".to_string(),
    //         };
    //     },
    //     None => "".to_string(),
    // };
    let header_value = "".to_string();
    // let header_name = parts.first();
    // let header_value: String = parts.tail().into_iter().collect().unwrap_or("");
    // let (header_name, header_value) = header.split(header.split_whitespace());
    // let parts: vec
    // let header_name = String::new();
    // let header_value = String::new();
    // let (header_name, header_value) = parts.split_first();
    // match parts.split_first() {
    //     Ok(result) => {
    //         let (header_name, header_value) = result;
    //     },
    //     Err(e) => println!("Failed parsing header: {}!", e),
    // }
    println!("header: {}\nhas value: {}", header_name, header_value);
    if header_value != "" {
        Some(Header {name: header_name, value: header_value} )
    } else {
        None
    }
}