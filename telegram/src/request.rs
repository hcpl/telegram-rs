use hyper;
use serde::Serialize;
use serde_mtproto::Identifiable;

use error;
use message::Message;


#[derive(Debug)]
pub struct Request<T> {
    message: Message<T>,
}

impl<T: Serialize + Identifiable> Request<T> {
    /// Create a new Telegram client.
    #[inline]
    pub fn new(body: T) -> Request<T> {
        Request {
            message: Message::new(body),
        }
    }

    /// Converts this request into a `hyper::Request`.
    pub fn to_http_request(&self) -> error::Result<hyper::Request> {
        let buffer = self.message.to_bytes()?;

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

    pub fn to_bytes(&self) -> error::Result<Vec<u8>> {
        self.message.to_bytes()
    }
}
