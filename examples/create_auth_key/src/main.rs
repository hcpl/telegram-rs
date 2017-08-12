extern crate env_logger;
extern crate extprim;
#[macro_use]
extern crate extprim_literals;
extern crate futures;
extern crate hyper;
extern crate log;
extern crate telegram;
extern crate tokio_core;

use futures::Future;
use telegram::{schema, Request, Response};
use tokio_core::reactor::Core;


fn main() {
    env_logger::init().unwrap();

    // Request for (p,q) Authorization
    // https://core.telegram.org/mtproto/samples-auth_key

    // [DEBUG] Step
    println!(" * Request for (p,q) Authorization");

    let req = Request::new(schema::mtproto::req_pq {
        nonce: i128!(0x3E0549828CCA27E966B301A48FECE2FC),
    });

    // [DEBUG] Step
    println!(" - Request");
    println!("{:#?}\n", req);

    // [DEBUG] Step
    println!(" - Serialized request");

    // [DEBUG] Show buffer
    let buffer = req.to_bytes().unwrap();
    pprint(&buffer);

    // [DEBUG] Step
    println!(" - Send {}\n", "http://149.154.167.50:443/api");

    let mut core = Core::new().unwrap();
    let client = telegram::Client::new(&core.handle());
    let promise = client.send(req).map(|data: Response<schema::mtproto::ResPQ>| {
        // [DEBUG] Step
        println!(" - Response");
        pprint(&data.to_bytes().unwrap());

        println!(" - Deserialized response");
        println!("{:#?}\n", data);
    });

    core.run(promise).unwrap();
}

fn pprint(buffer: &[u8]) {
    const CHUNK_SIZE: usize = 0x10;

    for (index, chunk) in buffer.chunks(CHUNK_SIZE).enumerate() {
        print!(" {:04X} |", index * CHUNK_SIZE);

        for byte in chunk {
            print!(" {:02X}", byte);
        }

        println!();
    }

    println!();
}
