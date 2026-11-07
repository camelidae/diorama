use std::io::{Read, Write};

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "hello".into());
    match mode.as_str() {
        "hello" => {
            println!("hello from WASI 0.3");
            println!("arg={}", std::env::args().nth(2).unwrap_or_default());
            println!("env={}", std::env::var("P3_TEST").unwrap_or_default());
            eprintln!("p3 stderr");
        }
        "file" => println!("file={}", std::fs::read_to_string("/data/message").unwrap().trim()),
        "exit" => std::process::exit(7),
        "wait" => loop { std::thread::sleep(std::time::Duration::from_secs(1)); },
        "http" => {
            let listener = std::net::TcpListener::bind("0.0.0.0:8080").unwrap();
            println!("listening on 8080");
            for connection in listener.incoming() {
                let mut stream = connection.unwrap();
                let mut buf = [0; 4096];
                let _ = stream.read(&mut buf);
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nwasip3!\n").unwrap();
            }
        }
        _ => panic!("unknown mode: {mode}"),
    }
}
