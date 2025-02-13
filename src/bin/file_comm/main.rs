use std::net::*;
use abes_nice_things::input;
mod formats;

fn main() {
    let settings = Settings::new();
    if let None = settings {
        return
    }
    let settings = settings.unwrap();
    match settings.host {
        Some(port) => host(port, settings),
        None => connect(settings),
    }
}
fn host(port: u16, settings: Settings) {
    println!("Listening...");
    for connection in TcpListener::bind(
        (Ipv4Addr::UNSPECIFIED, port)
    ).expect("Failed to bind to port").incoming() {
        println!("Incoming connection");
        match connection {
            Ok(stream) => {
                let _ = formats::hand_shake(stream, settings);
            }
            Err(error) => eprintln!("Failed to connect: {error}")
        }
    }
}
fn connect(settings: Settings) {
    loop {
        match settings.mode.unwrap() {
            Mode::Send => println!("What port:addr do you want to send a file to?"),
            Mode::Recv => println!("What port:addr do you want to recieve a file from?")
        }
        match TcpStream::connect(input()) {
            Ok(stream) => formats::hand_shake(stream, settings).expect("INVALID FORMAT"),
            Err(error) => eprintln!("Failed to connect: {error}")
        }
    }
}
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
enum Mode {
    Send,
    Recv
}