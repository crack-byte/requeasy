use std::borrow::Cow::{Borrowed, Owned};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore};

const DEFAULT_PORT: &str = "80";
const SSL_PORT: &str = "443";

#[derive(Debug)]
pub struct ApiResponse {
    pub header: HashMap<String, String>,
    pub body: String,
}

fn parse_url(url: &str) -> (&str, &str, String) {
    let (protocol, rest) = url.split_once("://").unwrap();
    let (temp_hostname, pathname) = rest.split_once("/").unwrap();
    let (hostname, port) = if temp_hostname.contains(":") {
        temp_hostname.split_once(":").expect("Invalid hostname")
    } else {
        (temp_hostname, if protocol.eq("http") { DEFAULT_PORT } else { SSL_PORT })
    };
    let socket_addr = format!("{}:{}", hostname, port);
    (hostname, pathname, socket_addr)
}

fn populate_get_request(
    host: &str,
    path: &str,
    data: Option<&String>,
    method: Option<&String>,
    headers: Vec<&str>,
) -> String {
    let default_method = String::from("GET");
    let method = method.unwrap_or(&default_method);
    let mut res = String::new();
    res += &format!("{} /{} HTTP/1.1\r\n", method, path);
    res += &format!("Host: {}\r\n", host);
    res += "Accept: */*\r\n";
    res += "Accept-Language: en-US,en;q=0.5\r\n";
    res += "Connection: close\r\n";
    // res += "Accept-Encoding: gzip, deflate, br\r\n";
    res += "Accept-Charset: UTF-8, ISO-8859-1;q=0.8\r\n";
    res += "Content-Type: application/json; charset=UTF-8\r\n";

    if method == "POST" || method == "PUT" {
        if headers.len() > 0 {
            for head in headers {
                res += head;
            }
            res += "\r\n"
        } else {
            res += "Content-Type: application/json\r\n";
        }
        if let Some(data_str) = data {
            let data_bytes = data_str.as_bytes();
            res += &format!("Content-Length: {}\r\n\r\n", data_bytes.len());
            res += data_str;
            res += "\r\n";
        }
    }

    res += "\r\n";
    res
}

fn parse_resp(resp: &str) -> (&str, &str) {
    let (response_header, response_data) = resp.split_once("\r\n\r\n").unwrap();
    (response_header, response_data)
}
/// dummy get method to execute a GET method call with the provided URL
pub fn get(url: &str) -> ApiResponse {
    // argument matching
    let verbose_enabled = false;
    let data = Option::None;
    let method = Option::None;
    let headers: Vec<&str> = Vec::new();
    let (hostname, pathname, socket_addr) = parse_url(url);
    let buffer_str = populate_get_request(hostname, &pathname, data, method, headers);
    if verbose_enabled {
        let lines = buffer_str.lines();
        for (index, line) in lines.enumerate() {
            println!("{:?} > {}", index, line)
        }
    }

    let server_name = hostname.try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(Arc::new(get_config()), server_name).unwrap();
    let mut sock = TcpStream::connect(socket_addr).unwrap();
    let mut stream = rustls::Stream::new(&mut conn, &mut sock);
    stream
        .write_all(buffer_str.as_bytes())
        .expect("Failed to write data to stream");
    let mut reader = BufReader::new(stream);
    let mut buff: Vec<u8> = vec![];
    reader.read_to_end(&mut buff);
    let response = String::from_utf8_lossy(&buff);

    let mut header_map: HashMap<String, String> = HashMap::new();
    let mut body = String::new();
    match response {
        Borrowed(res) => {
            // dividing the response headers and body
            let (response_header, response_data) = parse_resp(&res);

            let lines = response_header.split("\r\n");
            for (index, line) in lines.enumerate() {
                if index > 0 {
                    let (key, value) = line.split_once(":").unwrap();
                    header_map.insert(key.parse().unwrap(), value.trim().parse().unwrap());
                }
            }
            body = response_data.to_string()
        }
        Owned(res) => {
            let (response_header, response_data) = parse_resp(&res);

            let lines = response_header.split("\r\n");
            for (index, line) in lines.enumerate() {
                if index > 0 {
                    let (key, value) = line.split_once(":").unwrap();
                    header_map.insert(key.parse().unwrap(), value.trim().parse().unwrap());
                }
            }
            body = response_data.to_string();
        }
    }
    return ApiResponse {
        header: header_map,
        body,
    };
}

fn get_config() -> ClientConfig {
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS
            .iter()
            .map(|ta| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    ta.subject,
                    ta.spki,
                    ta.name_constraints,
                )
            }),
    );
    return ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
}
