extern crate tokio;
extern crate futures;
extern crate hyper;
extern crate env_logger;

use std::thread::spawn;
use futures::{Future, Stream, oneshot};
use futures::future::{lazy, ok};
use hyper::{Client, Body, Response};
use hyper::server::{Http, const_service, service_fn};
use tokio::reactor::Handle;



fn main() {
    env_logger::init();

    tokio::run(lazy(||{
        let (shutdown, shutdown_rx) = oneshot::<()>();
        let reply_service = const_service(service_fn(|_req|{
            let reply = "Hello World";
            Ok(Response::new(Body::from(reply)))
        }));


        let addr = "127.0.0.1:1234".parse().unwrap();
        let server = Http::new()
            .bind(&addr, reply_service)
            .unwrap();

        let shutdown_signal = shutdown_rx.map_err(|_|());
        let f = server
            .run_until(shutdown_signal)
            .map_err(|e| println!("Server Error: {}", e))
            .inspect(|_| println!("Server finished"));
        tokio::spawn(f);

        let client = Client::new(&Handle::current());
        let call_future = client.get("http://localhost:1234/".parse().unwrap())
            .and_then(|response| {
                response.into_parts().1.into_stream().concat2()
            })
            .and_then(move |chunk| {
                drop(shutdown);
                println!("Received: {:?}", chunk);
                ok(())
            })
            .map_err(|e| {
                println!("Client error: {}", e);
            });

        call_future
    }));
}
