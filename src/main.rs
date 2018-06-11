extern crate futures;
extern crate hyper;
extern crate hyper_proxy;
extern crate stopwatch;
extern crate tokio_core;

use futures::{Future, Stream};
use hyper::client::HttpConnector;
use hyper::{Client};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use tokio_core::reactor::Core;

fn main() {
    let use_proxy = true;
    let proxy_uri: Option<String> = Some("http://localhost:8118".to_owned());

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let mut proxy = None;
    // looking for polymorphic variable that works with both proxyed and unproxyed hyper clients
    let mut client: hyper::Client<hyper::client::HttpConnector, hyper::Body>;

    if use_proxy && proxy_uri.is_some() {
        println!("Using proxy: {}", proxy_uri.unwrap().as_str());
        proxy = Some({
            let proxy_uri = proxy_uri.unwrap().parse().unwrap();
            let mut proxy = Proxy::new(Intercept::All, proxy_uri);
            let connector = HttpConnector::new(4, &handle);
            let proxy_connector = ProxyConnector::from_proxy(connector, proxy).unwrap();
            proxy_connector
        });
        client = Client::configure()
            .connector(proxy.clone().unwrap())
            .build(&handle);
    } else {
        client = Client::configure()
            .connector(HttpConnector::new(4, &handle))
            .build(&handle);
    }

    // use proxy below
}
