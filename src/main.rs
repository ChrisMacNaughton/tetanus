#[macro_use]
extern crate clap;
use clap::App;
use std::net::TcpListener;
use std::thread;
use std::io::prelude::*;
use std::net::Shutdown;

mod request;

#[derive(Debug)]
struct Args {
    port: String,
    address: String,
    directory: String,
    config: String,
}

impl Args {
    fn new(port: &str, address: &str, directory: &str, config: &str) -> Args {
        Args {
            port: port.to_string(),
            address: address.to_string(),
            directory: directory.to_string(),
            config :config.to_string(),
        }
    }
}

fn handle_args() -> Args{
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();
    Args::new(
        matches.value_of("PORT").unwrap_or("8080"),
        matches.value_of("ADDRESS").unwrap_or("127.0.0.1"),
        matches.value_of("DIRECTORY").unwrap_or("."),
        matches.value_of("CONFIG").unwrap_or("default.conf"),
    )
}

fn handle_client(mut request: request::Request) {
    let resp = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<doctype !html><html><head><title>Hello</title></head><body><h1>Hello, world!</h1></body></html>\r\n";
    request.response = request::Response {
        body: resp.to_string(),
    };
    match request.stream.write(request.response.to_response()) {
        Ok(_) => println!("Response sent!"),
        Err(e) => println!("Failed sending response: {}!", e),
    }
    request.stream.shutdown(Shutdown::Both);
}
fn main() {
    let args = handle_args();
    println!("{:?}", args);
    let listen_on = format!("{}:{}", args.address, args.port);
    println!("About to bind to: {}", listen_on);

    // https://users.rust-lang.org/t/string-type-coercion-in-rust/1439
    let listener = TcpListener::bind(&*listen_on).unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    // let mut stream = stream.unwrap();
                    // stream.write(b"Hello World\r\n");
                    // connection succeeded
                    let request = request::Request::new(stream);
                    handle_client(request);
                });
            }
            Err(_) => { /* connection failed */ }
        }
    }

    // close the socket server
    drop(listener);
}