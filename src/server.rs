use std::io::prelude::*;
use image::RgbImage;

use message::*;

#[derive(Debug)]
pub struct Server<S: Read + Write> {
    messenger: Messenger<S>,
    dimensions: (u32, u32)
}

impl<S: Read + Write> Server<S> {
    pub fn new(stream: S, dimensions: (u32, u32)) -> Result<Server<S>, String> {
        let messenger = Messenger::new(stream);
       
        match messenger.send(&Message::Dimensions(dimensions)) {
            Err(e) => return Err(e),
            Ok(len) => println!("sent {} bytes", len)
        };

        Ok(Server {
            messenger,
            dimensions
        })
    }

    pub fn recv(&self) -> Result<RgbImage, String> {
        let message = match self.messenger.recv() { 
            Ok(m) => m,
            Err(e) => return Err(e)
        };


        match message {
            Message::RgbImage(raw) => {
                let (width, height) = self.dimensions;
                match RgbImage::from_raw(width, height, raw) {
                    Some(image) => Ok(image),
                    None => Err("Couldn't deserialize RgbImage: Mismatched dimensions".to_string())
                }
            },
            _ => Err("Unexpected Message, expected ImageData".to_string())
        }
    }
}


#[cfg(test)]
mod tests {
    use std::net::{TcpStream, TcpListener};
    use std::time;
    use image::Rgb;
    use rand::random;

    use super::*;
    use client::Client;
    
    fn server_client(dimensions: (u32, u32)) -> (Server<TcpStream>, Client<TcpStream>) {
        let server_addr = "127.0.0.1:31415";
        let listener = TcpListener::bind(server_addr).unwrap();
        let client_stream = TcpStream::connect(server_addr).unwrap();
        let (server_stream, _client_addr) = listener.accept().unwrap();

        let server = Server::new(server_stream, dimensions.clone()).unwrap();
        let client = Client::new(client_stream).unwrap();

        (server, client)
    }

    #[test]
    fn dimensions() {
        let dimensions = (64, 32);
        let (_server, client) = server_client(dimensions);
        assert_eq!(dimensions, client.dimensions());

        let (_server, client) = server_client(dimensions);
        let dimensions = (65, 32);
        assert_ne!(dimensions, client.dimensions());
    }

    #[test]
    fn simple_image() {
        let (width, height) = (64, 32);
        let (server, client) = server_client((width, height));
        let image = RgbImage::from_fn(width, height, |x, y| {
            let (x, y) = (x as u8, y as u8);
            if x.wrapping_mul(y) % 5 == 0 {
                Rgb([x, y, x.wrapping_add(y)])
            }
            else {
                Rgb([255u8.wrapping_sub(y), 255u8.wrapping_sub(x.wrapping_mul(y)), x.wrapping_add(y)])
            }
        });
        client.send(image.clone()).unwrap();
        let recieved_image = server.recv().unwrap();
        assert_eq!(image.into_raw(), recieved_image.into_raw());
    }

    fn framerate(minimum: u8) {
        let (width, height) = (64, 32);
        let (server, client) = server_client((width, height));
        let test_duration = time::Duration::new(2, 0);
        let start = time::Instant::now();
        let mut frames = 0;

        let image = RgbImage::from_fn(width, height, |_, _| {
            Rgb([random::<u8>(), random::<u8>(), random::<u8>()])
        });
        let image_raw = image.clone().into_raw();

        while time::Instant::now() - start < test_duration {
            let t0 = time::Instant::now();
            for _ in 0..10 {
                client.send(image.clone()).unwrap();
            }
            let t1 = time::Instant::now();
            for _ in 0..10 {
                let recieved_image = server.recv().unwrap();
                assert_eq!(image_raw, recieved_image.into_raw());
            }
            let t2 = time::Instant::now();
            frames += 10;
            println!("send:   {:?}", t1 - t0);
            println!("recv:   {:?}", t2 - t1);
            println!("total : {:?}", t2 - t0);
            println!("===============");
        }

        let rate = (frames / test_duration.as_secs()) as u8;
        println!("Framerate: {}, minimum: {}", rate, minimum);
        assert!(rate >= minimum);
    }

    #[test]
    fn framerate_20() {
        framerate(20);
    }
    
    #[test]
    fn framerate_40() {
        framerate(40);
    }

    #[test]
    fn framerate_60() {
        framerate(60);
    }
}
