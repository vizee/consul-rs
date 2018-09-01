extern crate http;
extern crate hyper;
extern crate percent_encoding;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate serde_derive;

pub mod api;
pub mod client;
pub mod types;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Future;

    #[test]
    fn it_works() {
        let c = client::Client::new("http://127.0.0.1:8500", None);
        hyper::rt::run(
            c.kv_get("test", None)
                .map(|v| {
                    println!("{:?}", v);
                }).map_err(|e| {
                    println!("{:?}", e);
                }),
        );
    }
}
