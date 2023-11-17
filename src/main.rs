use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use std::{env, io};
use url::Url;

const ENV_CONFIG: &str = "SD_PROXY_ORIGIN";

#[derive(Deserialize, Debug)]
struct IncomingRequest {
    method: String,
    path_query: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    body: Option<String>,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

const fn default_timeout() -> u64 {
    10
}

#[derive(Serialize, Debug)]
struct OutgoingResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

fn main() {
    // Get the hostname from the environment
    let origin = match env::var(ENV_CONFIG) {
        Ok(val) => val,
        Err(err) => panic!("couldn't read {ENV_CONFIG}: {err}"),
    };
    dbg!(&origin);
    // Read incoming request from stdin
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    dbg!(&buffer);
    let incoming_request: IncomingRequest =
        serde_json::from_str(&buffer).unwrap();
    dbg!(&incoming_request);
    // We construct the URL by first parsing the origin and then appending the
    // path query. This forces the path query to be part of the path and prevents
    // it from getting itself into the hostname.
    let url = Url::parse(&origin).unwrap();
    let url = url.join(&incoming_request.path_query).unwrap();

    dbg!(&url);

    let client = Client::new();
    let mut req = client
        .request(Method::from_str(&incoming_request.method).unwrap(), url);
    let header_map = HeaderMap::try_from(&incoming_request.headers).unwrap();
    req = req
        .headers(header_map)
        .timeout(Duration::from_secs(incoming_request.timeout));
    if let Some(body) = incoming_request.body {
        req = req.body(body);
    }
    let resp = req.send().unwrap();
    let mut headers = HashMap::new();
    for (name, value) in resp.headers() {
        headers.insert(name.to_string(), value.to_str().unwrap().to_string());
    }
    let outgoing_response = OutgoingResponse {
        status: resp.status().as_u16(),
        headers,
        body: resp.text().unwrap(),
    };
    println!("{}", serde_json::to_string(&outgoing_response).unwrap())
}
