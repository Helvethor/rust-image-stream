use std::io::prelude::*;
use image::RgbImage;

use message::*;


pub struct Client<S: Read + Write> {
    messenger: Messenger<S>,
    dimensions: (u32, u32)
}

impl<S: Read + Write> Client<S> {
    pub fn new(stream: S) -> Result<Client<S>, String> {
        let messenger = Messenger::new(stream);

        let dimensions = match messenger.recv() {
            Ok(m) => match m {
                Message::Dimensions(d) => d,
                _ => return Err("Unexpected Message, expected Capabilities Message".to_string())
            },
            Err(e) => return Err(e)
        };

        Ok(Client { 
            messenger,
            dimensions
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    pub fn send(&self, image: RgbImage) -> Result<usize, String> {
        let image_data = image.into_raw();
        let message = Message::RgbImage(image_data);
        self.messenger.send(&message)
    }
}
