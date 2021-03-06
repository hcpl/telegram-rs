extern crate extprim;
#[macro_use]
extern crate extprim_literals;
extern crate hyper;
extern crate telegram;
extern crate futures;
extern crate tokio_core;

use futures::Future;
use tokio_core::reactor::Core;


fn main() {
    // Request for (p,q) Authorization
    // https://core.telegram.org/mtproto/samples-auth_key

    // [DEBUG] Step
    println!(" * Request for (p,q) Authorization");

    let req = telegram::Request::new(telegram::schema::mtproto::req_pq {
        nonce: i128!(0x3E0549828CCA27E966B301A48FECE2FC),
    });

    // [DEBUG] Step
    println!(" - Message");
    println!("{:?}\n", req);

    // [DEBUG] Step
    println!(" - Serialize");

    // [DEBUG] Show buffer
    let buffer = req.to_vec().unwrap();
    pprint(&buffer);

    // [DEBUG] Step
    println!(" - Send {}\n", "http://149.154.167.50:443/api");

    let mut core = Core::new().unwrap();
    let client = telegram::Client::new(&core.handle());
    let promise = client.request(req).map(|data| {
        // [DEBUG] Step
        println!(" - Receive");

        pprint(&data);
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
