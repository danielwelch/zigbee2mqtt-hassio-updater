#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hyper;

extern crate serde;
extern crate serde_json;

extern crate actix;
extern crate actix_web;
extern crate crypto;
extern crate reqwest;

#[cfg(test)]
mod tests;

use std::env;
use std::string::ToString;

use actix_web::Result as ActixWebResult;
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized, ParseError};
use actix_web::middleware::{Middleware, Started};
use actix_web::{http, server, App, Error, HttpMessage, HttpRequest, HttpResponse, Json, Responder};

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha1::Sha1;

/// An incoming PushEvent from Github Webhook.
#[derive(Deserialize)]
struct PushEvent {
    #[serde(rename = "ref")]
    reference: String,
}

#[derive(Serialize)]
struct TravisRequest {
    message: String,
    branch: String,
}

#[derive(Serialize)]
struct ServerMessage {
    message: String,

    #[serde(skip_serializing)]
    e: Option<Error>,
}

use hyper::header::Headers;
header! { (TravisAPIVersion, "Travis-API-Version") => [u32] }

impl Responder for ServerMessage {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        if self.e.is_some() {
            return Err(self.e.unwrap());
        } else {
            let body = serde_json::to_string(&self)?;
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(body))
        }
    }
}

impl ServerMessage {
    fn success<T: ToString>(s: T) -> ServerMessage {
        ServerMessage {
            message: s.to_string(),
            e: None,
        }
    }

    fn error(e: Error) -> ServerMessage {
        ServerMessage {
            message: "".to_owned(),
            e: Some(e),
        }
    }
}

struct VerifySignature;

impl<S> Middleware<S> for VerifySignature {
    fn start(&self, req: &mut HttpRequest<S>) -> ActixWebResult<Started> {
        use std::io::Read;

        let r = req.clone();
        let s = r.headers()
            .get("X-Hub-Signature")
            .ok_or(ErrorUnauthorized(ParseError::Header))?
            .to_str()
            .map_err(ErrorUnauthorized)?;
        // strip "sha1=" from the header
        let (_, sig) = s.split_at(5);

        let secret = env::var("GITHUB_SECRET").unwrap();
        let mut body = String::new();
        req.read_to_string(&mut body)
            .map_err(ErrorInternalServerError)?;

        if is_valid_signature(&sig, &body, &secret) {
            Ok(Started::Done)
        } else {
            Err(ErrorUnauthorized(ParseError::Header))
        }
    }
}

fn is_valid_signature(signature: &str, body: &str, secret: &str) -> bool {
    let digest = Sha1::new();
    let mut hmac = Hmac::new(digest, secret.as_bytes());
    hmac.input(body.as_bytes());
    let expected_signature = hmac.result();

    crypto::util::fixed_time_eq(
        bytes_to_hex(expected_signature.code().to_vec()).as_bytes(),
        signature.as_bytes(),
    )
}

fn bytes_to_hex(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

fn travis_request(url: &str) -> ActixWebResult<reqwest::Response> {
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header(reqwest::header::ContentType::json())
        .header(TravisAPIVersion(3))
        .header(reqwest::header::Authorization(auth_str()))
        .json(&TravisRequest {
            message: "API Request triggered by zigbee2mqtt update".to_string(),
            branch: "master".to_string(),
        })
        .send()
        .map_err(ErrorInternalServerError)?;
    Ok(res)
}

fn auth_str() -> String {
    format!("token {}", std::env::var("TRAVIS_TOKEN").unwrap()).to_owned()
}

fn index(push: Json<PushEvent>) -> impl Responder {
    // check if reference string contains master
    // if so, trigger a build by sending post request to travis URL
    let travis_url = env::var("TRAVIS_URL").unwrap();
    if push.reference.ends_with("master") {
        // send request to travis
        match travis_request("https://api.travis-ci.org/repo/19145006/requests") {
            Ok(_) => ServerMessage::success(format!(
                "PushEvent on branch master found, request sent to {}",
                travis_url
            )),
            Err(e) => ServerMessage::error(e),
        }
    } else {
        ServerMessage::success("PushEvent is not for master branch")
    }
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

fn main() {
    use std::net::{SocketAddr, ToSocketAddrs};
    let sys = actix::System::new("updater");
    let addr = SocketAddr::from(([0, 0, 0, 0], get_server_port()));

    server::new(|| {
        App::new()
            .middleware(VerifySignature)
            .resource("/", |r| r.method(http::Method::POST).with(index))
    }).bind(addr)
        .unwrap()
        .start();

    println!("Listening for incoming POST requests to /");
    let _ = sys.run();
}
