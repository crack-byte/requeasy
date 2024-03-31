use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::{OwnedTrustAnchor, RootCertStore};

pub const DEFAULT_PORT: &str = "443";

fn parse_url(url: &str) -> (&str, &str, String) {
    let (protocol, rest) = url.split_once("://").unwrap();
    let (temp_hostname, pathname) = rest.split_once("/").unwrap();
    let (hostname, port) = if temp_hostname.contains(":") {
        temp_hostname.split_once(":").expect("Invalid hostname")
    } else {
        (temp_hostname, DEFAULT_PORT)
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
    res += "Connection: keep-alive\r\nAccept-Charset: UTF-8, ISO-8859-1;q=0.8\r\nContent-Type: application/json; charset=UTF-8\r\n";

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

pub fn get() {
    // argument matching
    let verbose_enabled = true;
    let url = "https://dummyjson.com/products";
    let data = Option::None;
    let method = Option::None;
    let headers: Vec<&str> = Vec::new();
    println!("{url}");
    let (hostname, pathname, socket_addr) = parse_url(url);
    let buffer_str = populate_get_request(hostname, &pathname, data, method, headers);
    let tcp_socket = TcpStream::connect(socket_addr);

    match tcp_socket {
        Ok(mut stream) => {
            if verbose_enabled {
                let lines = buffer_str.lines();
                for line in lines {
                    println!("> {}", line)
                }
            }
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
            let config = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            let server_name = "dummyjson.com".try_into().unwrap();
            let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();
            let mut sock = TcpStream::connect("dummyjson.com:443").unwrap();
            let mut stream = rustls::Stream::new(&mut conn, &mut sock);
            stream
                .write_all(buffer_str.as_bytes())
                .expect("Failed to write data to stream");
            let mut buffer = [0; 4096 * 10];
            let bytes = stream
                .read(&mut buffer)
                .expect("Failed to read from response from host!");
            println!("{bytes}");
            // converts buffer data into a UTF-8 enccoded string (lossy ensures invalid data can be truncated).
            let response = String::from_utf8_lossy(&buffer[..bytes]);

            // dividing the response headers and body
            let (response_header, response_data) = parse_resp(&response);
            if verbose_enabled {
                let lines = response_header.split("\r\n");
                for line in lines {
                    println!("< {}", line)
                }
            }
            println!("-->{}", response_data);
        }
        Err(e) => {
            eprintln!("Failed to establish connection: {}", e);
        }
    }

    // Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        get();
        assert_eq!(4, 4);
    }
}
