use futures::{future, Stream, Future};
use hyper::{self, Body};
use hyper::client::HttpConnector;
use tokio::reactor::Core;

use error;
use request::Request;
use ser::Serialize;


pub struct Client {
    core: Core,
    http_client: hyper::Client<HttpConnector, Body>,
}

impl Client {
    /// Create a new Telegram client.
    #[inline]
    pub fn new() -> error::Result<Client> {
        let core = Core::new()?;
        let http_client = hyper::Client::new(&core.handle());

        Ok(Client {
            core: core,
            http_client: http_client,
        })
    }

    /// Send a constructed request using this Client.
    pub fn send<F, T, U>(&mut self, req: Request<T>, on_receive_handler: F) -> error::Result<U>
        where F: FnOnce(Vec<u8>) -> U,
              T: Serialize,
    {
        let future: Box<Future<Item = U, Error = error::Error>> =
            match req.to_http_request() {
                Ok(http_request) => {
                    Box::new(self.http_client
                        .request(http_request)
                        .and_then(|res| res.body().concat2())
                        .map(|data| data.to_vec())
                        .map(on_receive_handler)
                        .map_err(|err| err.into()))
                },
                Err(error) => {
                    Box::new(future::err(error))
                },
            };

        self.core.run(future)
    }
}
