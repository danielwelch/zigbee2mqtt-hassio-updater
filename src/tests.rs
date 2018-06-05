use super::*;
use actix_web::test::TestServer;
use std::env;

#[test]
#[should_panic]
fn test_auth_str_no_envvar() {
    env::remove_var("TRAVIS_TOKEN");
    auth_str();
}

#[test]
fn test_auth_str() {
    env::set_var("TRAVIS_TOKEN", "ablkjdfsoiuwre");
    assert_eq!(auth_str(), "token ablkjdfsoiuwre".to_owned());
}

#[test]
fn test_deserialize_github_payload() {
    use std::fs::File;
    let payload = File::open("./resources/github_webhook_payload.json").unwrap();
    let data: PushEvent = serde_json::from_reader(payload).unwrap();
}

#[test]
fn test_is_valid_signature() {
    let signature = "51633b546c869c7de65ce2f44d0c5eb49c0e5101";
    let body = "hello_world";
    let secret = "SUPERS3CR3T";

    assert!(is_valid_signature(signature, body, secret));
}

#[test]
fn test_verify_sig_middleware() {
    fn index(req: HttpRequest) -> &'static str {
        "Hello world!"
    }

    env::remove_var("GITHUB_SECRET");
    env::set_var("WEBHOOK_SECRET", "SUPERS3CR3T");

    let secret = "SUPERS3CR3T";
    let body = "hello_world";

    let digest = Sha1::new();
    let mut hmac = Hmac::new(digest, secret.as_bytes());
    hmac.input(body.as_bytes());
    let mut expected_signature: Vec<u8> = Vec::new();
    expected_signature.extend_from_slice(hmac.result().code());

    let mut header: Vec<String> = Vec::new();
    header.push(String::from("sha1="));
    header.push(bytes_to_hex(expected_signature));
    let h = header.join("");

    let mut srv = TestServer::new(|app| app.middleware(VerifySignature).handler(index));
    let _req = srv.post()
        .header("X-Hub-Signature", h)
        .header(http::header::CONTENT_TYPE, "application/json")
        .finish()
        .expect("error setting headers");

    let resp = srv.execute(_req.send()).expect("error sending request");
    assert!(resp.status().is_success());
}
