extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate extprim;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate telegram_derive;
extern crate tokio_core as tokio;

pub mod ser;
// pub mod de;
pub mod error;
mod client;
mod request;

pub use client::Client;
pub use request::Request;

#[allow(non_camel_case_types)]
pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/schema.rs"));

    pub mod mtproto {
        include!(concat!(env!("OUT_DIR"), "/mtproto_schema.rs"));
    }
}
