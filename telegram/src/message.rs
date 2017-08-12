use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_mtproto::{self, Boxed, Identifiable};

use error;


#[derive(Debug, Serialize, Deserialize)]
pub struct Message<T> {
    auth_key_id: u64,
    message_id: u64,
    body: Boxed<T>,
}

impl<T: Identifiable> Message<T> {
    pub fn new(body: T) -> Message<T> {
        // Generate a "unique" message id
        // > Exact unixtime * 2^32
        // FIXME: This can't fail. Attempt to replace this with something from std that
        //        understands that so we don't have an `.unwrap` here
        let now_d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let now_s = now_d.as_secs();
        let message_id = ((now_s as u64) << 32) + (now_d.subsec_nanos() as u64);

        Message {
            auth_key_id: 0,
            message_id: message_id,
            body: Boxed::new(body),
        }
    }
}

impl<T: Serialize> Message<T> {
    pub fn to_writer<W: io::Write>(&self, mut writer: W) -> error::Result<()> {
        // TODO: Handle the auth_key_id in the request
        // auth_key_id
        serde_mtproto::to_writer(&mut writer, &self.auth_key_id)?;
        // message_id
        serde_mtproto::to_writer(&mut writer, &self.message_id)?;

        // Prepare inner message to compute message_length
        let temp = serde_mtproto::to_bytes(&self.body)?;
        // message_length
        serde_mtproto::to_writer(&mut writer, &(temp.len() as u32))?;
        // Write the message to the writer following the message_length
        writer.write(&temp)?;

        Ok(())
    }

    pub fn to_bytes(&self) -> error::Result<Vec<u8>> {
        let mut result = Vec::new();

        self.to_writer(&mut result)?;

        Ok(result)
    }
}

impl<T: DeserializeOwned> Message<T> {
    pub fn from_reader<R: io::Read>(mut reader: R) -> error::Result<Message<T>> {
        // auth_key_id
        let auth_key_id = serde_mtproto::from_reader(&mut reader, None)?;
        // message_id
        let message_id = serde_mtproto::from_reader(&mut reader, None)?;

        // TODO: use deserialized message_length to check if it equals the predicted
        // message_length for the deserialized value
        // message_length
        let _message_length: u32 = serde_mtproto::from_reader(&mut reader, None)?;
        // inner message
        let body: Boxed<T> = serde_mtproto::from_reader(&mut reader, None)?;

        Ok(Message {
            auth_key_id: auth_key_id,
            message_id: message_id,
            body: body,
        })
    }
}

impl<'a, T: Deserialize<'a>> Message<T> {
    pub fn from_bytes(bytes: &'a [u8]) -> error::Result<Message<T>> {
        // auth_key_id
        let auth_key_id = serde_mtproto::from_bytes(bytes, None)?;
        // message_id
        let message_id = serde_mtproto::from_bytes(bytes, None)?;

        // TODO: use deserialized message_length to check if it equals the predicted
        // message_length for the deserialized value
        // message_length
        let _message_length: u32 = serde_mtproto::from_bytes(bytes, None)?;
        // inner message
        let body: Boxed<T> = serde_mtproto::from_bytes(bytes, None)?;

        Ok(Message {
            auth_key_id: auth_key_id,
            message_id: message_id,
            body: body,
        })
    }
}
