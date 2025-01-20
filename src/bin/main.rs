use std::{io::{Read, Write}, net};
use abes_nice_things::prelude::*;
fn main() {
    let mut stream;
    loop {
        println!("What addr:port do you want to connect to?");
        match net::TcpStream::connect(input()) {
            Ok(done) => {
                stream = done;
                println!("Connection made");
                break;
            }
            Err(error) => {
                eprintln!("Failed to connect: {error}");
            }
        }
    }
    let mut stream_clone = stream.try_clone().unwrap();
    std::thread::spawn(move || {
        loop {
            let mut stdout = std::io::stdout().lock();
            let mut buf = [0_u8; 16];
            match stream_clone.read(&mut buf) {
                Ok(num) => {
                    if num == 0 {
                        break;
                    }
                }
                Err(error) => {
                    if let std::io::ErrorKind::Interrupted = error.kind() {
                        
                    }
                    panic!("errorses: {error}")
                }
            }
            stdout.write_all(&mut buf).unwrap();
            stdout.flush().unwrap();
        }
    });
    loop {
        let input = input();
        stream.write_all(&[input.as_bytes(), "\n".as_bytes()].concat()).unwrap();
    }
}