extern crate hyper;
extern crate url;

use std::io::Read;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use std::fmt;

use self::hyper::Client;
use self::hyper::status::StatusCode;
use self::url::{ParseResult, Url, UrlParser};

use parse;

const TIMEOUT: u64 = 10;

#[derive(Debug, Clone)]
pub enum UrlState {
    Accessible(Url),
    BadStatus(Url, StatusCode),
    ConnectionFailed(Url),
    TimedOut(Url),
    Malformed(String),
}

impl fmt::Display for UrlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UrlState::Accessible(url) => format!(" !! {}", url).fmt(f),
            UrlState::BadStatus(url, status) => format!(" !! {} ({})", url, status).fmt(f),
            UrlState::ConnectionFailed(url) => format!(" !! {} (connection failed)", url).fmt(f),
            UrlState::TimedOut(url) => format!(" !! {} (timed out)", url).fmt(f),
            UrlState::Malformed(url) => format!(" !! {} (malformed)", url).fmt(f),
        }
    }
}

fn build_url(domain: &str, path: &str) -> ParseResult<Url> {
    let base_url_string = format!("http://{}", domain);
    let base_url = Url::parse(&base_url_string).unwrap();

    let mut raw_url_parser = UrlParser::new();
    let url_parser = raw_url_parser.base_url(&base_url);

    url_parser.parse(path)
}

pub fn url_status(domain: &str, path: &str) -> UrlState {
    match build_url(domain, path) {
        Ok(url) => {
            let (tx, rx) = channel();
            let req_tx = tx.clone();
            let u = url.clone();

            // Thread to send request to the url
            thread::spawn(move | | {
                let client = Client::new();
                let url_string = url.serialize();
                let response = client.get(&url_string).send();

                let url_state = match response {
                    Ok(response) => {
                        if let StatusCode::Ok = response.status {
                            UrlState::Accessible(url)
                        } else {
                            UrlState::BadStatus(url, response.status)
                        }
                    },
                    Err(_) => UrlState::ConnectionFailed(url),
                };

                let _ = req_tx.send(url_state);
            });

            // Thread to decide that the url has timed out 
            thread::spawn(move | | {
                thread::sleep(Duration::from_secs(TIMEOUT));
                let _ = tx.send(UrlState::TimedOut(u));
            });

            rx.recv().unwrap()
        },
        Err(_) => UrlState::Malformed(path.to_owned()),
    }
}

pub fn fetch_url(url: &Url) -> String {
    let client = Client::new();
    let url_string = url.serialize();

    let mut response = client
        .get(&url_string)
        .send()
        .ok()
        .expect("could not fetch URL");

    let mut body = String::new();

    match response.read_to_string(&mut body) {
        Ok(_) => body,
        Err(_) => String::new(),
    }
}

pub fn fetch_all_urls(url: &Url) -> Vec<String> {
    let html_src = fetch_url(url);
    let dom = parse::parse_html(&html_src);

    parse::get_urls(dom.document)
}
