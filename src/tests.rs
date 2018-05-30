use super::*;
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

// #[test]
// fn test_travis_request() {
//     let test_travis_repo =
//     let res = travis_request()
// }
