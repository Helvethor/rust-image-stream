use std::io::prelude::*;
use std::cell::RefCell;
use bincode::{serialize, deserialize, Infinite, Bounded};
use bufstream::BufStream;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Dimensions((u32, u32)),
    RgbImage(Vec<u8>),
} 

#[derive(Debug)]
pub struct Messenger<S: Read + Write> {
    stream: RefCell<BufStream<S>>
}

impl<S: Read + Write> Messenger<S> {
    pub fn new(stream: S) -> Messenger<S> {
        Messenger {
            stream: RefCell::new(BufStream::new(stream))
        }
    }

    pub fn send(&self, message: &Message) -> Result<usize, String> {
        let mut stream = self.stream.borrow_mut();
        let data = match serialize(message, Infinite) {
            Ok(d) => d,
            Err(e) => return Err(format!("Couldn't serialize message: {}", e))
        };

        let len_data = match serialize(&data.len(), Bounded(8)) {
            Ok(d) => d,
            Err(e) => return Err(format!("Couldn't serialize data length: {}", e))
        };

        let mut len = match stream.write(&len_data) {
            Ok(l) => l,
            Err(e) => return Err(format!("Couldn't send data: {}", e))
        };

        len += match stream.write(&data) {
            Ok(l) => l,
            Err(e) => return Err(format!("Couldn't send data: {}", e))
        };

        match stream.flush() {
            Ok(()) => Ok(len),
            Err(e) => Err(format!("Couldn't flush output stream: {}", e))
        }
    }

    pub fn recv(&self) -> Result<Message, String> {
        let mut stream = self.stream.borrow_mut();
        let mut len_data = [0u8; 8];
        if let Err(e) = stream.read_exact(&mut len_data) {
            return Err(format!("Couldn't read data length: {}", e))
        }

        let len = match deserialize(&len_data) {
            Ok(l) => l,
            Err(e) => return Err(format!("Couldn't deserialize data length: {}", e))
        };

        let mut data = vec![0u8; len];
        if let Err(e) = stream.read_exact(&mut data) {
            return Err(format!("Couldn't read data: {}", e));
        }

        match deserialize(&data[..]) {
            Ok(m) => Ok(m),
            Err(e) => Err(format!("Couldn't deserialize data: {}", e))
        }
    }
}
