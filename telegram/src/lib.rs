extern crate byteorder;
extern crate tokio_core as tokio;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate error_chain;
extern crate extprim;
#[macro_use]
extern crate telegram_derive;

pub mod ser;
// pub mod de;
pub mod error;
mod client;
mod request;

pub use client::Client;
pub use error::{Error, Result};
pub use request::Request;

#[allow(non_camel_case_types)]
pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/schema.rs"));

    pub mod mtproto {
        include!(concat!(env!("OUT_DIR"), "/mtproto_schema.rs"));
    }
}
