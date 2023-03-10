use std::convert::identity;
use std::io::{BufRead, Read};
use std::pin::Pin;
use async_trait::async_trait;
use flate2::bufread::GzDecoder;
use hyper::header::CONTENT_TYPE;
use hyper::{client::{Client, HttpConnector}, Response, Body, body::{aggregate, Buf}, Method, header::{CONTENT_ENCODING, ACCEPT, ACCEPT_ENCODING, USER_AGENT}, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use serde::{de::DeserializeOwned, Serialize};
use crate::{Result, ApiRequest};
use futures_util::{Future, FutureExt, TryFutureExt, future::ready};

trait ToJson: Serialize + Send + Sync {
    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }
}

impl<T> ToJson for T where T: Serialize + Send + Sync { }

pub struct ApiClient {
    client: Client<HttpsConnector<HttpConnector>>,
    prefix: String,
}

impl ApiClient {
    pub fn new(prefix: String) -> Self {
        let https = HttpsConnectorBuilder::new().with_webpki_roots().https_or_http().enable_http1().build();
        let client = Client::builder().build(https);
        Self {
            client,
            prefix,
        }
    }

    pub fn uri(&self, uri: &str) -> String {
        format!("{}{}", self.prefix, uri)
    }

    async fn send(&self, request: Request<Body>) -> Result<Box<dyn Read>> {
        self.client.clone().request(request)
        .map_ok(|response| (is_zip_encoded(&response), response))
        .and_then(
            |(is_zip, response)| 
                aggregate(response)
                .map_ok(move |r| wrap_reader(r.reader(), is_zip)
            )
        )
        .err_into()
        .await
    }

}

fn to_object<T>(response: Box<dyn Read>) -> Result<T>
where T: DeserializeOwned
{
    serde_json::from_reader(response)
    .map_err(Into::into)
}

#[async_trait]
impl ApiRequest for ApiClient {
    async fn get<T>(&self, uri: &str) -> Result<T>
    where T: DeserializeOwned
    {
        api_request(
            self.uri(uri),
            Method::GET,
            None
        )
        .and_then(|r| self.send(r))
        .map_ok(to_object)
        .await
        .and_then(identity)
    }

    async fn post<T, U>(&self, uri: &str, object: T) -> Result<U>
    where 
        T: Serialize + Send + Sync,
        U: DeserializeOwned
    {
        api_request(
            self.uri(uri),
            Method::POST,
            Some(object.to_json()?.into())
        )
        .and_then(|r| self.send(r))
        .map_ok(to_object)
        .await
        .and_then(identity)
    }

    async fn delete(&self, uri: &str) -> Result<()> {
        api_request(
            self.uri(uri),
            Method::DELETE,
            None
        )
        .and_then(|r| self.send(r))
        .map_ok(|_| {})
        .await
    }
}

fn api_request(uri: String, method: Method, body: Option<Body>) -> Pin<Box<dyn Future<Output = Result<Request<Body>>> + Send>>
{
    ready(
        match body {
            Some(body) =>
                Request::builder()
                .uri(uri)
                .method(method)
                .header(ACCEPT, "application/json")
                .header(ACCEPT_ENCODING, "gzip")
                .header(CONTENT_TYPE, "application/json")
                .header(USER_AGENT, "Rust hyper")
                .body(body)
                .map_err(Into::into),
            None =>
                Request::builder()
                .uri(uri)
                .method(method)
                .header(ACCEPT, "application/json")
                .header(ACCEPT_ENCODING, "gzip")
                .header(USER_AGENT, "Rust hyper")
                .body(Body::empty())
                .map_err(Into::into),
}
    )
    .boxed()
}

fn is_zip_encoded(response: &Response<Body>) -> bool {
    response.headers()
    .get_all(CONTENT_ENCODING)
    .iter()
    .any(|e| e == "gzip")
}

fn wrap_reader<'a>(reader: impl BufRead + Send + 'a, is_zip: bool) -> Box<dyn Read + 'a> {
    if is_zip {
        Box::new(GzDecoder::new(reader))
    }
    else {
        Box::new(reader)
    }
}
