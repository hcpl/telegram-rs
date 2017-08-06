use std::time::{SystemTime, UNIX_EPOCH};

use hyper;
use serde::Serialize;
use serde_mtproto::{self, Identifiable};

use error;

#[derive(Debug)]
pub struct Request<T: Serialize + Identifiable> {
    message_id: u64,
    message_body: T,
}

impl<T: Serialize + Identifiable> Request<T> {
    /// Create a new Telegram client.
    #[inline]
    pub fn new(body: T) -> Self {
        // Generate a "unique" message id
        // > Exact unixtime * 2^32
        // FIXME: This can't fail. Attempt to replace this with something from std that
        //        understands that so we don't have an `.unwrap` here
        let now_d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let now_s = now_d.as_secs();
        let message_id = ((now_s as u64) << 32) + (now_d.subsec_nanos() as u64);

        Request {
            message_id,
            message_body: body,
        }
    }

    /// Converts this request into a `hyper::Request`.
    pub fn to_http_request(&self) -> error::Result<hyper::Request> {
        let buffer = self.to_vec()?;

        let mut http_request = hyper::Request::new(
            hyper::Method::Post,
            // FIXME: This _cannot_ fail, find a way to do this where the API knows this
            "http://149.154.167.50:443/api".parse().unwrap(),
        );

        http_request
            .headers_mut()
            .set(hyper::header::Connection::keep_alive());

        http_request
            .headers_mut()
            .set(hyper::header::ContentLength(buffer.len() as u64));

        http_request.set_body(buffer);

        Ok(http_request)
    }

    /// Converts this request into a byte vector.
    pub fn to_vec(&self) -> error::Result<Vec<u8>> {
        let mut result = Vec::new();

        // TODO: Handle the auth_key_id in the request
        // auth_key_id
        serde_mtproto::to_writer(&mut result, &0u64)?;

        // message_id
        serde_mtproto::to_writer(&mut result, &self.message_id)?;

        // Prepare inner message to compute message_length
        let message = serde_mtproto::to_bytes_identifiable(&self.message_body)?;

        // message_length
        serde_mtproto::to_writer(&mut result, &(message.len() as u32))?;

        // Push the message onto the buffer following the message_length
        result.extend(message);

        Ok(result)
    }
}
