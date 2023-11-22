use anyhow::Result;
use reqwest::blocking::{Client, Response};
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use std::{env, io};
use url::Url;

const ENV_CONFIG: &str = "SD_PROXY_ORIGIN";

/// Incoming requests (as JSON) received over stdin
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct IncomingRequest {
    method: String,
    path_query: String,
    stream: bool,
    #[serde(default)]
    headers: HashMap<String, String>,
    body: Option<String>,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

/// Default timeout for requests; serde requires this be a function
fn default_timeout() -> u64 {
    10
}

/// Serialization format for non-streamed requests
#[derive(Serialize, Debug)]
struct OutgoingResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

/// Serialization format for errors, always over stderr
#[derive(Serialize, Debug)]
struct ErrorResponse {
    error: String,
}

fn handle_json_response(resp: Response) -> Result<()> {
    let mut headers = HashMap::new();
    for (name, value) in resp.headers() {
        headers.insert(name.to_string(), value.to_str()?.to_string());
    }
    let outgoing_response = OutgoingResponse {
        status: resp.status().as_u16(),
        headers,
        body: resp.text()?,
    };
    println!("{}", serde_json::to_string(&outgoing_response)?);
    Ok(())
}

fn handle_stream_response(mut resp: Response) -> Result<()> {
    let mut stdout = io::stdout().lock();
    resp.copy_to(&mut stdout)?;
    Ok(())
}

fn proxy() -> Result<()> {
    // Get the hostname from the environment
    let origin = env::var(ENV_CONFIG)?;
    // Read incoming request from stdin (must be on single line)
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let incoming_request: IncomingRequest = serde_json::from_str(&buffer)?;
    // We construct the URL by first parsing the origin and then appending the
    // path query. This forces the path query to be part of the path and prevents
    // it from getting itself into the hostname.
    let url = Url::parse(&origin)?;
    // TODO: Consider just allowlisting a number of API paths instead of relying
    // on the url library to join it properly and avoid type confusion
    let url = url.join(&incoming_request.path_query)?;

    let client = Client::new();
    let mut req =
        client.request(Method::from_str(&incoming_request.method)?, url);
    let header_map = HeaderMap::try_from(&incoming_request.headers)?;
    req = req
        .headers(header_map)
        .timeout(Duration::from_secs(incoming_request.timeout));
    if let Some(body) = incoming_request.body {
        req = req.body(body);
    }
    // Fire off the request!
    let resp = req.send()?;
    // We return the output in two ways, either a JSON blob or stream the output.
    // JSON is used for HTTP 4xx, 5xx, and all non-stream requests.
    if !incoming_request.stream
        || resp.status().is_client_error()
        || resp.status().is_server_error()
    {
        handle_json_response(resp)?;
    } else {
        handle_stream_response(resp)?;
    }
    Ok(())
}

fn main() {
    match proxy() {
        Ok(()) => {}
        Err(err) => {
            // Try to serialize into our error format
            let resp = ErrorResponse {
                error: err.to_string(),
            };
            match serde_json::to_string(&resp) {
                Ok(json) => {
                    // Print the error to stderr
                    eprintln!("{json}")
                }
                Err(_) => {
                    // It should be near impossible that an error message
                    // is not JSON serializable, but just handle this corner
                    // case explicitly
                    // TODO: attempt to log underlying error
                    eprintln!(r#"{{"error": "unable to serialize error"}}"#)
                }
            }
        }
    }
}
