#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;


use std::env;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::collections::HashMap;
use glium::Surface;
use conrod_core::{Color, Colorable, text, Widget};
use conrod_core::position::Dimension;

mod support;


// graphics

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must provide one and only one url");
    }
    let url = &args[1].as_str();

    browse(url);

    render();

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

fn render() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Tundra")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new(ui.widget_id_generator());

    // Poll events from the window.
    let mut event_loop = support::EventLoop::new();
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&mut events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        // Instantiate all widgets in the GUI.
        set_widgets(ui.set_widgets(), ids);

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

fn set_widgets(ref mut ui: conrod_core::UiCell, ids: &mut Ids) {
    use conrod_core::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    // Construct our main `Canvas` tree.
    widget::Canvas::new().color(color::WHITE).set(ids.master, ui);

    widget::Rectangle::outline([10., 40.])
        .color(color::TRANSPARENT)
        .x(0.0)
        .y(0.0)
        .set(ids.rectangle, ui);

//    println!("Rect of {:?}", ui.rect_of(ids.master));

    //set the amount of text
    ids.text.resize(4, &mut ui.widget_id_generator());
    widget::Text::new("G")
        .color(color::BLACK)
        .x_y(-200.0, 200.0)
        .set(ids.text[0], ui);
    widget::Text::new("u")
        .color(color::BLACK)
        .x_y(-190.0, 200.0)
        .set(ids.text[1], ui);
    widget::Text::new("y")
        .color(color::BLACK)
        .x_y(-180.0, 200.0)
        .set(ids.text[2], ui);
    widget::Text::new("Watson")
        .color(color::BLACK)
        .x_y(-140.0, 200.0)
        .set(ids.text[3], ui);

    //todo: make a helper method that uses `ui.rect_of(ids.text[3])` in order to
    //  compute the absolute position within the window. The current behavior is
    //  that (0,0) is the center point of the widget, not the top left.
    //  I want absolute -> equivalent relative








}

// Generate a unique `WidgetId` for each widget.
// Generates the boilerplate for all the `button: conrod::widget::Id, ...`
widget_ids! {
    struct Ids {
        master,
        rectangle,
        oval,
        text[],
        text2,
//        master,
//        header,
//        body,
//        left_column,
//        middle_column,
//        right_column,
//        footer,
//        footer_scrollbar,
//        floating_a,
//        floating_b,
//        tabs,
//        tab_foo,
//        tab_bar,
//        tab_baz,
//
//        title,
//        subtitle,
//        top_left,
//        bottom_right,
//        foo_label,
//        bar_label,
//        baz_label,
//        button_matrix,
//        bing,
//        bong,
    }
}