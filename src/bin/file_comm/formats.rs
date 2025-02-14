use std::net::TcpStream;
use abes_nice_things::{ToBinary, FromBinary};
use crate::{Settings, Mode};

pub type FormatID = u32;
pub const HIGHEST: FormatID = 0;

pub fn hand_shake(stream: TcpStream, settings: Settings) -> Result<(), ()> {
    println!("Beginning format handshake");
    //////////////////////////////////////////////////////////////
    // The person sending will be the one to suggest the format //
    //////////////////////////////////////////////////////////////
    // We are assuming that backwards compatability exists      //
    //////////////////////////////////////////////////////////////
    match settings.mode {
        Mode::Recv => recv_hand_shake(stream, settings),
        Mode::Send => send_hand_shake(stream, settings)
    }
}
fn send_hand_shake(mut stream: TcpStream, settings: Settings) -> Result<(), ()> {
    println!("Sending suggested format: {}", settings.get_format());
    settings.get_format().to_binary(&mut stream);
    // Format is decided by the reciever
    let format = FormatID::from_binary(&mut stream);
    println!("Decided to use format: {format}\nFormat handshake done");
    match format {
        0 => f0::send(stream, settings),
        _ => return Err(())
    }
    Ok(())
}
fn recv_hand_shake(mut stream: TcpStream, settings: Settings) -> Result<(), ()> {
    println!("Waiting for suggested format");
    let other_highest = FormatID::from_binary(&mut stream);
    println!("Suggestion: {other_highest}");
    let format = {
        // We are able to process their highest format
        if other_highest <= settings.get_format() {
            println!("Accepting suggestion");
            other_highest
        }
        // We cannot match them, so they have to match us
        else {
            println!("Suggestion is impossible, sending alternative");
            settings.get_format()
        }
    };
    format.to_binary(&mut stream);
    println!("Format handshake done");
    match format {
        0 => f0::recv(stream),
        _ => return Err(())
    };
    Ok(())
}
mod f0 {
    // Format:
    // Length of name(u32)
    // name
    // data
    use std::{
        net::TcpStream,
        fs::File,
        path::PathBuf,
        io::{Read, Write}
    };
    use crate::Settings;
    use abes_nice_things::{input, input_yn, ToBinary, FromBinary};

    static mut TO_SEND: Option<PathBuf> = None;
    pub fn send(mut stream: TcpStream, settings: Settings) {
        let file;
        #[allow(static_mut_refs)]// compiler is wrong
        let path = match unsafe { TO_SEND.clone() } {
            Some(path) => {
                file = File::open(&path).unwrap();
                path
            },
            None => {
                loop {
                    println!("What file do you want to send?");
                    let path = input();
                    match File::open(&path) {
                        Ok(file_in) => {
                            println!("Valid file");
                            file = file_in;
                            if let Some(_) = settings.host {
                                if input_yn("Do you want to use this for subsequent requests?y/n") {
                                    unsafe { TO_SEND = Some(PathBuf::from(&path)) }
                                }
                            }
                            break PathBuf::from(path)
                        }
                        Err(error) => eprintln!("Failed to identify file validity: {error}")
                    }
                }
            }
        };
        let path = path.file_name().expect("Failed to get file name").to_str().unwrap();
        let len = file.metadata().expect("Failed to get file metadata").len();

        println!("Sending metadata");
        (path.len() as u32).to_binary(&mut stream);
        stream.write_all(path.as_bytes()).unwrap();
        println!("Sending file");
        transfer(file, stream, len, 1000);
        println!("File sent")
    }
    pub fn recv(mut stream: TcpStream) {
        println!("Getting metadata");
        let name_len = u32::from_binary(&mut stream);
        let mut buf = vec![0; name_len as usize];
        stream.read_exact(&mut buf).unwrap();
        let name = String::from_utf8(buf).unwrap();
        if !match stream.peer_addr() {
            Ok(addr) => input_yn(
                &format!(
                    "Are you sure you want to accept {name} from {addr}?y/n"
                )
            ),
            Err(_) => input_yn(
                &format!(
                    "Are you sure you want to accept {name} from unknown?y/n"
                )
            )
        } {
            return
        }
        let mut buf = Vec::new();
        println!("Getting data");
        stream.read_to_end(&mut buf).unwrap();
        println!("Writing to file");
        std::fs::write(name, buf).unwrap();
        println!("Done")
        
    }
    fn transfer(mut from: impl Read, mut to: impl Write, mut len: u64, interval: usize) {
        while len > interval as u64 {
            let mut buf = vec![0_u8; interval];
            from.read_exact(&mut buf).unwrap();
            to.write_all(&buf).unwrap();
            len -= interval as u64;
        }
        let mut buf = vec![0; len.try_into().unwrap()];
        from.read_exact(&mut buf).unwrap();
        to.write_all(&buf).unwrap();
    }
}