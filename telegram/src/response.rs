use std::io;

use serde::ser::Serialize;
use serde::de::{Deserialize, DeserializeOwned};
use serde_mtproto::Identifiable;

use error;
use message::Message;


#[derive(Debug)]
pub struct Response<T> {
    message: Message<T>,
}

impl<T: Identifiable> Response<T> {
    pub fn new(body: T) -> Response<T> {
        Response {
            message: Message::new(body),
        }
    }
}

impl<T: Serialize> Response<T> {
    pub fn to_bytes(&self) -> error::Result<Vec<u8>> {
        self.message.to_bytes()
    }
}

impl<T: DeserializeOwned> Response<T> {
    pub fn from_reader<R: io::Read>(reader: R) -> error::Result<Response<T>> {
        let message = Message::from_reader(reader)?;

        Ok(Response {
            message: message,
        })
    }
}

impl<'a, T: Deserialize<'a>> Response<T> {
    pub fn from_bytes(bytes: &'a [u8]) -> error::Result<Response<T>> {
        let message = Message::from_bytes(bytes)?;

        Ok(Response {
            message: message,
        })
    }
}
