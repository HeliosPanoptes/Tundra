extern crate gtk;   // Application/Window
extern crate cairo; // Drawing
extern crate gio;   // IO

use std::env;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::collections::HashMap;
use std::f64::consts::PI;

// graphics
use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;
use cairo::{Context, FontSlant, FontWeight};

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
    let tundra_app = gtk::Application::new(Some("org.heliospanoptes.tundra"),
                                           gio::ApplicationFlags::FLAGS_NONE)
                                      .expect("Application::new failed");

    tundra_app.connect_activate(|app| {
        // We create the main window.
        let win = gtk::ApplicationWindow::new(app);

        // Then we set its size and a title.
        win.set_default_size(320, 200);
        win.set_title("Tundra");
//
        //allocate memory for the canvas
        let canvas = Box::new(DrawingArea::new)();
        win.add(&canvas);

        canvas.connect_draw(|_, cr| {
            cr.set_dash(&[3., 2., 1.], 1.);
            assert_eq!(cr.get_dash(), (vec![3., 2., 1.], 1.));

            cr.scale(500f64, 500f64);

            cr.set_source_rgb(250.0 / 255.0, 224.0 / 255.0, 55.0 / 255.0);
            cr.paint();

            cr.set_line_width(0.05);

            // border
            cr.set_source_rgb(0.3, 0.3, 0.3);
            cr.rectangle(0.0, 0.0, 1.0, 1.0);
            cr.stroke();

            cr.set_line_width(0.03);

            // draw circle
            cr.arc(0.5, 0.5, 0.4, 0.0, PI * 2.);
            cr.stroke();

            // mouth
            let mouth_top = 0.68;
            let mouth_width = 0.38;

            let mouth_dx = 0.10;
            let mouth_dy = 0.10;

            cr.move_to(0.50 - mouth_width / 2.0, mouth_top);
            cr.curve_to(
                0.50 - mouth_dx,
                mouth_top + mouth_dy,
                0.50 + mouth_dx,
                mouth_top + mouth_dy,
                0.50 + mouth_width / 2.0,
                mouth_top,
            );


            cr.stroke();

            let eye_y = 0.38;
            let eye_dx = 0.15;
            cr.arc(0.5 - eye_dx, eye_y, 0.05, 0.0, PI * 2.);
            cr.fill();

            cr.arc(0.5 + eye_dx, eye_y, 0.05, 0.0, PI * 2.);
            cr.fill();

            Inhibit(false)
        });

        // Don't forget to make all widgets visible.
        win.show_all();
        // Foreground
        win.present();


    });



    tundra_app.run(&[]);
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

//pub fn drawable<F>(window: &gtk::ApplicationWindow, draw_fn: F)
//    where
//        F: Fn(&DrawingArea, &Context) -> Inhibit + 'static, {
//    // Allocate on the heap
//
//
//    canvas.connect_draw(draw_fn);
//
//
//}

//fn build_ui(window: &gtk::ApplicationWindow) {
//    drawable(window, |_, cr| {
//        cr.set_dash(&[3., 2., 1.], 1.);
//        assert_eq!(cr.get_dash(), (vec![3., 2., 1.], 1.));
//
//        cr.scale(500f64, 500f64);
//
//        cr.set_source_rgb(250.0 / 255.0, 224.0 / 255.0, 55.0 / 255.0);
//        cr.paint();
//
//        cr.set_line_width(0.05);
//
//        // border
//        cr.set_source_rgb(0.3, 0.3, 0.3);
//        cr.rectangle(0.0, 0.0, 1.0, 1.0);
//        cr.stroke();
//
//        cr.set_line_width(0.03);
//
//        // draw circle
//        cr.arc(0.5, 0.5, 0.4, 0.0, PI * 2.);
//        cr.stroke();
//
//        // mouth
//        let mouth_top = 0.68;
//        let mouth_width = 0.38;
//
//        let mouth_dx = 0.10;
//        let mouth_dy = 0.10;
//
//        cr.move_to(0.50 - mouth_width / 2.0, mouth_top);
//        cr.curve_to(
//            0.50 - mouth_dx,
//            mouth_top + mouth_dy,
//            0.50 + mouth_dx,
//            mouth_top + mouth_dy,
//            0.50 + mouth_width / 2.0,
//            mouth_top,
//        );
//
//        println!("Extents: {:?}", cr.fill_extents());
//
//        cr.stroke();
//
//        let eye_y = 0.38;
//        let eye_dx = 0.15;
//        cr.arc(0.5 - eye_dx, eye_y, 0.05, 0.0, PI * 2.);
//        cr.fill();
//
//        cr.arc(0.5 + eye_dx, eye_y, 0.05, 0.0, PI * 2.);
//        cr.fill();
//
//        Inhibit(false)
//    });
//}
