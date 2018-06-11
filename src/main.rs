extern crate futures;
extern crate hyper;
extern crate hyper_proxy;
extern crate tokio_core;
extern crate tokio_io;

use futures::{Future, Stream};
use hyper::{Chunk, Method, Request, Uri, client::{Config, HttpConnector, Service}};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use std::io;
use tokio_core::reactor::Core;
use tokio_io::{AsyncRead, AsyncWrite};

trait AsyncRw: AsyncWrite + AsyncRead {}
impl<T> AsyncRw for T
where
    T: AsyncWrite + AsyncRead,
{
}

#[derive(Clone)]
enum ProxyOrNotConnector {
    Proxy(ProxyConnector<HttpConnector>),
    Not(HttpConnector),
}

impl Service for ProxyOrNotConnector {
    type Request = Uri;
    type Response = Box<AsyncRw>;
    type Error = io::Error;

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match self {
            ProxyOrNotConnector::Proxy(p) => {
                let x = p.call(req);
                let y = x.map(|y| Box::new(y) as Box<AsyncRw>);
                Box::new(y)
            }
            ProxyOrNotConnector::Not(n) => {
                let x = n.call(req);
                let y = x.map(|y| Box::new(y) as Box<AsyncRw>);
                Box::new(y)
            }
        }
    }
}

fn main() {
    let proxy_uri = Some("http://localhost:8118");
    let use_proxy = true;
    let uri: Uri = "http://httpbin.org/ip".parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let http_connector = HttpConnector::new(4, &handle);

    let connector = match (proxy_uri, use_proxy) {
        (Some(proxy_uri), true) => {
            println!("Using proxy: {}", proxy_uri);
            let proxy_uri = proxy_uri.parse().unwrap();
            let proxy = Some(Proxy::new(Intercept::All, proxy_uri));
            let proxy_connector =
                ProxyConnector::from_proxy(http_connector, proxy.unwrap()).unwrap();
            ProxyOrNotConnector::Proxy(proxy_connector)
        }
        _ => ProxyOrNotConnector::Not(http_connector),
    };

    let client = Config::default().connector(connector.clone()).build(&handle);
    let mut req: hyper::Request;
    match use_proxy {
        true => {
            req = Request::new(Method::Get, uri.clone());
            if let ProxyOrNotConnector::Proxy(x) = connector.clone() {
                if let Some(headers) = x.http_headers(&uri) {
                    req.headers_mut().extend(headers.iter());
                    req.set_proxy(true);
                }
            }
        }
        false => req = Request::new(Method::Get, uri.clone()),
    }

    let future_http = client
        .request(req)
        .and_then(|res| res.body().concat2())
        .map(move |body: Chunk| ::std::str::from_utf8(&body).unwrap().to_string());

    let x = core.run(future_http).unwrap();
    println!("{:?}", x);
}
