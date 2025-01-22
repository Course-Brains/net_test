use std::net::Ipv4Addr;
use tokio::net;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::*;
#[tokio::main]
async fn main() {
    let listener = create_listener();
    let mut mother_ship = MotherShip::new();
    let listener = listener.await;
    println!("Listening");
    loop {
        let connection = listener.accept().await;
        match connection {
            Ok((stream, addr)) => {
                println!("New connection by: {addr}");
                let (read_child, write_child) = mother_ship.produce().into_split();
                let (read, write) = stream.into_split();
                tokio::spawn(input(read, write_child));
                tokio::spawn(output(write, read_child));
            }
            Err(error) => {
                eprintln!("Failed to make connection: {error}")
            }
        }
    }
}
async fn input(mut tcp: net::tcp::OwnedReadHalf, send: WriteChild<[u8; 30]>) {
    loop {
        let mut buf = [0; 30];
        match tcp.read(&mut buf).await {
            Ok(num) => {
                if num == 0 {
                    println!("Connection closed");
                    return;
                }
            }
            Err(error) => {
                eprintln!("Failed to get input from tcp: {error}");
                return;
            }
        }
        //println!("sending data to others: {}", String::from_utf8_lossy(&buf));
        if let Err(error) = send.send(buf) {
            eprintln!("Failed to distribute recieved data: {error}");
            continue;
        }
    }
}
async fn output(mut tcp: net::tcp::OwnedWriteHalf, mut recv: ReadChild<[u8; 30]>) {
    loop {
        match recv.recv().await {
            Ok(data) => {
                if let Err(error) = tcp.write_all(&data).await {
                    eprintln!("Failed to send data over TCP: {error}\x1b[");
                    return;
                }
            }
            Err(error) => {
                match error {
                    RecvError::FromSelf => {
                        continue;
                    }
                    RecvError::Error(error) => {
                        eprintln!("Failed to recieve data from other threads: {error}");
                        continue;
                    }
                }
            }
        }
    }
}
async fn create_listener() -> net::TcpListener {
    loop {
        println!("What port do you want it hosted on?");
        let port;
        match abes_nice_things::input().parse::<u16>() {
            Ok(num) => {
                port = num;
            }
            Err(error) => {
                eprintln!("That is an invalid number: {error}");
                continue;
            }
        }
        match tokio::join!(net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))).0 {
            Ok(listen) => {
                return listen
            }
            Err(error) => {
                eprintln!("failed to bind port: {error}");
            }
        }
    }
}
struct MotherShip<T: Clone> {
    send: Sender<(usize, T)>,
    num: usize
}
impl<T: Clone> MotherShip<T> {
    fn new() -> Self {
        MotherShip {
            send: channel(5).0,
            num: 0
        }
    }
    fn produce(&mut self) -> Child<T> {
        self.num += 1;
        Child {
            recv: self.send.subscribe(),
            send: self.send.clone(),
            id: self.num-1
        }
    }
}
struct Child<T: Clone> {
    recv: Receiver<(usize, T)>,
    send: Sender<(usize, T)>,
    id: usize
}
impl<T: Clone> Child<T> {
    /*async fn recv(&mut self) -> Result<T, RecvError> {
        let (id, data) = self.recv.recv().await?;
        if self.id == id {
            return Err(RecvError::FromSelf)
        }
        Ok(data)
    }
    fn send(&self, data: T) -> Result<usize, error::SendError<(usize, T)>> {
        self.send.send((self.id, data))
    }*/
    fn into_split(self) -> (ReadChild<T>, WriteChild<T>) {
        (
            ReadChild {
                recv: self.recv,
                id: self.id
            },
            WriteChild {
                send: self.send,
                id: self.id
            }
        )
    }
}
struct ReadChild<T: Clone> {
    recv: Receiver<(usize, T)>,
    id: usize
}
impl<T: Clone> ReadChild<T> {
    async fn recv(&mut self) -> Result<T, RecvError> {
        let (id, data) = self.recv.recv().await?;
        if self.id == id {
            return Err(RecvError::FromSelf)
        }
        Ok(data)
    }
}
struct WriteChild<T> {
    send: Sender<(usize, T)>,
    id: usize
}
impl<T> WriteChild<T> {
    fn send(&self, data: T) -> Result<usize, error::SendError<(usize, T)>> {
        self.send.send((self.id, data))
    }
}
#[derive(Debug)]
enum RecvError {
    FromSelf,
    Error(error::RecvError)
}
impl From<error::RecvError> for RecvError {
    fn from(value: error::RecvError) -> Self {
        RecvError::Error(value)
    }
}