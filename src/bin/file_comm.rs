use std::net::*;
use std::io::{Read, Write};
use std::str::FromStr;
fn input() -> String {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    buf.trim_end().to_string()
}
fn input_yn(msg: &str) -> bool {
    loop {
        println!("{msg}");
        match input().as_str() {
            "y" => return true,
            "n" => return false,
            _ => {}
        }
    }
}
fn main() {
    let settings = Settings::new();
    if let None = settings {
        return
    }
    let settings = settings.unwrap();
    match settings.mode.unwrap() {
        Mode::Recv => recv(settings.host),
        Mode::Send => send(settings.host)
    }
}
fn recv(host: Option<u16>) {
    match host {
        Some(port) => {
            println!("Listening...");
            for connection in make_listener(port).incoming() {
                match connection {
                    Ok(stream) => {
                        println!("Incoming connection");
                        recv_file(stream);
                    }
                    Err(error) => eprintln!("Failed to recieve connection: {error}")
                }
            }
        }
        None => {
            loop {
                println!("what port:addr do you want to receive a file from?");
                match TcpStream::connect(input()) {
                    Ok(stream) => {
                        recv_file(stream);
                        break;
                    }
                    Err(error) => eprintln!("Failed to connect: {error}")
                }
            }
        }
    }
}
fn send(host: Option<u16>) {
    let path;
    let data;
    loop {
        println!("What file do you want to send?");
        let in_path = input();
        match std::fs::exists(&in_path) {
            Ok(exists) => {
                if !exists {
                    eprintln!("That does not exist");
                    continue
                }
            }
            Err(error) => {
                eprintln!("Invalid path: {error}");
                continue;
            }
        }
        match std::fs::read(&in_path) {
            Ok(contents) => {
                data = contents;
                path = in_path;
                break;
            }
            Err(error) => {
                eprintln!("Failed to read file: {error}");
                continue;
            }
        };
    }
    let path = std::path::PathBuf::from_str(&path).unwrap()
        .file_name().unwrap()
        .to_str().unwrap()
        .to_string();
    match host {
        Some(port) => {
            println!("Listening");
            for connection in make_listener(port).incoming() {
                match connection {
                    Ok(mut stream) => {
                        println!("Incoming connection");
                        if !match stream.peer_addr() {
                            Ok(addr) => input_yn(
                                &format!("Do you want to send {path} to {addr}? y/n")
                            ),
                            Err(_error) => input_yn(
                                &format!("Do you want to send {path} to unknown? y/n")
                            )
                        } {
                            continue;
                        }
                        stream.write_all(&(path.len() as u32).to_le_bytes()).unwrap();
                        stream.write_all(path.as_bytes()).unwrap();
                        stream.write_all(&data).unwrap();
                        println!("File sent");
                    }
                    Err(error) => eprintln!("Failed to recieve connection: {error}")
                }
            }
        }
        None => {
            let mut stream;
            loop {
                println!("What addr:port do you want to connect to?");
                match TcpStream::connect(input()) {
                    Ok(tcp_stream) => {
                        stream = tcp_stream;
                        break;
                    }
                    Err(error) => eprintln!("Failed to connect: {error}")
                }
            }
            stream.write_all(&(path.len() as u32).to_le_bytes()).unwrap();
            stream.write_all(path.as_bytes()).unwrap();
            stream.write_all(&data).unwrap();
        }
    }
}
fn recv_file(mut stream: TcpStream) {
    let mut buf = [0_u8; 4];
    stream.read_exact(&mut buf).unwrap();
    let name_len = u32::from_le_bytes(buf);

    let mut buf = vec![0; name_len as usize];
    stream.read_exact(&mut buf).unwrap();
    let name = String::from_utf8(buf).unwrap();

    if !match stream.peer_addr() {
        Ok(addr) => {
            input_yn(&format!("Would you like to recieve ({name}) from {addr}? y/n"))
        }
        Err(_error) => {
            input_yn(&format!("Would you like to recieve ({name}) from unknown? y/n"))
        }
    } {
        return;
    }

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();

    std::fs::write(name, buf).unwrap();
    println!("File recieved");
}
fn make_listener(port: u16) -> TcpListener {
    return TcpListener::bind(
        (
            Ipv4Addr::UNSPECIFIED,
            port
        )
    ).expect(
        "Failed to bind to a port"
    )
}
struct Settings {
    mode: Option<Mode>,
    host: Option<u16>
}
impl Settings {
    const HELP: &str = 
"This is designed to send files to and from people who are able to communicate with each other over TCP.
The person hosting the TCP listener can be decided at runtime, but only one person in the interaction can host.
Hosting has certain advantages and disadvantages:

When receiving, you can recieve from multiple people easily.
When sending, you can send to multiple people easily.

But:
You cannot initiate the connections.

While there may only be one downside, it contains certain risks.
You must confirm that you are sending or recieving from the right person or be at rist of malware or data theft.
Towards that end, when you are hosting, you must confirm every connection before having it go through.
You must also do this when recieving and not hosting because I am lazy.

Also, and this is important, the data is sent of TCP, not TLS.
For those who don't recognise these, TLS is encrypted, TCP is not.
If you are using something that encrypts your data, or you encrypt the files you are sending, then that will matter less.

Here are the arguments to this binary:
    help
        Prints this out
    --recv
        Indicates that you will be recieving a file
    --send
        Indicates that you will be sending a file
        (You must include either --send or --recv)
    --host [port]
        Indicates you will be hosting and want to use the given port
        You must include a port because I am lazy
        
Using this you may notice that it closes and must be rerun after every successful exchange(unless you are hosting)
That is because I am too lazy to properly implement this, and I don't intend to.";
    fn new() -> Option<Settings> {
        let mut out = Settings::default();
        let mut args = std::env::args();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "help" => {
                    println!("{}", Settings::HELP);
                    return None
                }
                "--recv" => out.mode = Some(Mode::Recv),
                "--send" => out.mode = Some(Mode::Send),
                "--host" => {
                    out.host = Some(
                        args.next().expect("Need a port after --host")
                            .parse::<u16>().expect("Need a port after --host")
                    );
                }
                _ => {}
            }
        }
        if let None = out.mode {
            panic!("Need to specify if you are sending or recieving\n(--recv or --send)")
        }
        return Some(out);
    }
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            mode: None,
            host: None
        }
    }
}
enum Mode {
    Send,
    Recv
}