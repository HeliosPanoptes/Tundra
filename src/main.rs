#[macro_use] extern crate conrod_core;
extern crate conrod_glium;
#[macro_use] extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;

use std::env;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::collections::HashMap;

// graphics

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must provide one and only one url");
    }
    let url = &args[1].as_str();

    browse(url);

    start_tundra_app();

}

fn start_tundra_app() {

}

/// A convenience method that combines all of the steps for the browser to
/// display the page
fn browse(url: &str) {
    let (host, port, path, _fragment) = parse_address(url);
    let (_headers, body) = request(&host, &port, &path);
    show(body);
}

fn parse_address(url: &str) -> (String, String, String, String) {
    if !url.starts_with("http://") {
        panic!("Tundra only supports http");
    }

    let scheme_rest: Vec<_> = url.split("://").collect();
    let _scheme: &str = &scheme_rest[0];
    let rest: &str = &scheme_rest[1];

    let mut hostport = rest;
    let mut pathfragment = "/";
    if rest.contains("/") {
        let address: Vec<_> = rest.splitn(2, "/").collect();
        hostport = address[0];
        pathfragment = address[1];
    }

    let mut host: String = hostport.to_string();
    let mut port: String = "80".to_string();
    if hostport.contains(":") {
        let hostport_vec: Vec<_> = hostport.rsplitn(2, ":").collect();
        host = hostport_vec[1].to_string();
        port = hostport_vec[0].to_string();
    }

    let mut path: String = "/".to_string() + pathfragment;
    let mut fragment: String = "".to_string();
    if pathfragment.contains("#") {
        let pathfragment_vec: Vec<_> = pathfragment.rsplitn(2, "#").collect();
        path = "/".to_string() + pathfragment_vec[1];
        fragment = "#".to_string() + pathfragment_vec[0];
    }

    return (host, port, path, fragment)
}

fn request(host: &str, port: &str, path: &str) -> (HashMap<String, String>, String) {
    let address = format!("{}:{}", host, port);
    match TcpStream::connect(address) {
        Ok(mut socket) => {
            let request_string = format!("GET {} HTTP/1.1\r\n\
                                          Host: {}\r\n\
                                          User-Agent: HeliosPanoptes\r\n\
                                          Connection: close\r\n\r\n", path, host);

            socket.write(request_string.as_bytes()).unwrap();

            let mut buf = Vec::new();

            match socket.read_to_end(&mut buf) {
                Ok(_) => {
                    let response = String::from_utf8_lossy(&buf);

                    let response_vec: Vec<_> = response.split("\r\n\r\n").collect();
                    let raw_headers: String = response_vec[0].to_string();
                    let body: String = response_vec[1].to_string();

                    // split the headers into lines
                    let mut header_lines: Vec<_> = raw_headers.split("\r\n").collect();
                    // parse the http status line
                    let http_status_line: Vec<_> = header_lines[0].splitn(3, " ").collect();
                    let _version = http_status_line[0];
                    let status = http_status_line[1];
                    let explanation = http_status_line[2];
                    assert!(status == "200", format!("Server error{}:{}", status, explanation));
                    //remove the http status line from the list of headers
                    header_lines.remove(0);

                    let mut headers = HashMap::new();

                    for header in header_lines {
                        let header_line: Vec<_> = header.splitn(2, ":").collect();
                        headers.insert(header_line[0].to_string().trim().to_lowercase(),
                                       header_line[1].to_string().trim().to_lowercase());
                    };

                    return (headers, body);
                },
                Err(_e) => {
                    println!("Failed to receive data");
                    exit(1);
                }
            }
        }
        Err(_e) => {
            println!("Failed to connect to url");
            exit(1);
        }
    };
}

fn show(body: String) {
    let mut in_angle = false;
    for c in body.chars() {
        if c == '<' {
            in_angle = true;
        } else if c == '>' {
            in_angle = false;
        } else if !in_angle {
            print!("{}", c);
        }
    }
}