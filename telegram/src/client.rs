use futures::{future, Stream, Future};
use hyper::{self, Body};
use hyper::client::HttpConnector;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_mtproto::Identifiable;
use tokio::reactor::Handle;

use error;
use request::Request;
use response::Response;

pub struct Client {
    http_client: hyper::Client<HttpConnector, Body>,
}

impl Client {
    /// Create a new Telegram client.
    #[inline]
    pub fn new(handle: &Handle) -> Client {
        Client {
            http_client: hyper::Client::new(handle),
        }
    }

    // Send a constructed request using this Client.
    pub fn send<T, U>(
        &self,
        req: Request<T>
    ) -> Box<Future<Item = Response<U>, Error = error::Error>>
        where T: Serialize + Identifiable,
              U: 'static + DeserializeOwned + Identifiable,
    {
        let http_request = match req.to_http_request() {
            Ok(req) => req,
            Err(error) => return Box::new(future::err(error)),
        };

        Box::new(
            self.http_client
                .request(http_request)
                .and_then(|res| res.body().concat2())
                .map(|data| Response::from_reader(&*data))
                .flatten()
                .map_err(|err| err.into())
        )
    }
}
