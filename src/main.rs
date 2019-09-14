#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
//#[macro_use]
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
use conrod_core::{color, widget, Colorable, Widget, Positionable, Sizeable};

mod support;


struct WindowUi {
    ui : conrod_core::Ui,
    events_loop: glium::glutin::EventsLoop,
    display: support::GliumDisplayWinitWrapper
}
// graphics

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must provide one and only one url");
    }
    let url = &args[1].as_str();

    browse(url);
}

/// A convenience method that combines all of the steps for the browser to
/// display the page
///
/// At this point it holds all the state... maybe not the best, but we'll work with it
/// until it doesn't work anymore
fn browse(url: &str) {
    // construct our `Ui`.
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut window_ui = set_up_window();

    let (host, port, path, _fragment) = parse_address(url);
    let (_headers, body) = request(&host, &port, &path);
    let text = lex(body);
    let display_list = layout(&text, window_ui.ui.set_widgets());

    //move the ui value to render after setup
    render(window_ui, &display_list);
//    show(body);
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

fn lex(body: String) -> Vec<String> {
    let mut in_angle = false;
    let mut in_body = false;
    let mut tag_label = "".to_string();
    let mut text = Vec::new();
    for c in body.chars() {
        if c == '<' {
            in_angle = true;
            //reset the tag label
            tag_label = "".to_string();
            continue;
        } else if c == '>' {
            in_angle = false;
            continue;
        }
        //check for body tag
        if in_angle {
            tag_label.push(c);
        }
        if "body" == tag_label {
            //toggle the body bool
            in_body = !in_body;
        }

        if in_body && !in_angle {
            text.push(c.to_string());
        }
    }
    return text;
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

/// Takes text and returns a display-list of the format (x, y, text)
fn layout<'a>(text: &'a Vec<String>, ref mut ui: conrod_core::UiCell) -> Vec<(f64, f64, &'a String)> {
    let mut display_list = Vec::new(); // (x, y, text)
    //base position
    let mut x = 13.0;
    let mut y = 13.0;

    for character in text.iter(){
        if character == "\n" {
            x = 13.0;
            y += 25.0;
            continue;
        }

        display_list.push((x, y, character));

        //update for the next character
        x += 13.0;
        if x > ui.win_w {
            y += 18.0;
            x = 13.0;
        }
    }

    return display_list;
}

fn set_up_window() -> WindowUi {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    // Build the window.
    let events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Tundra")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();
    let font_path = assets.join("fonts/PingFang-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    return WindowUi {
        ui,
        events_loop,
        display
    }
}

fn render(mut window_ui: WindowUi, display_list: &Vec<(f64, f64, &String)>) {

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&window_ui.display.0).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new(window_ui.ui.widget_id_generator());

    // Poll events from the window.
    let mut event_loop = support::EventLoop::new();
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&mut window_ui.events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &window_ui.display) {
                window_ui.ui.handle_event(event);
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
        set_text(window_ui.ui.set_widgets(), ids, display_list);

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = window_ui.ui.draw_if_changed() {
            renderer.fill(&window_ui.display.0, primitives, &image_map);
            let mut target = window_ui.display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&window_ui.display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

fn set_text(ref mut ui: conrod_core::UiCell, ids: &mut Ids, display_list: &Vec<(f64, f64, &String)>) {
    // Construct our main `Canvas` tree.
    // The canvas here is just acting like a glorified background.
    // Normally we would use them to lay out the ui, and anchor elements to them, but
    // we're doing everything on our own, so nahhhh
    widget::Canvas::new().color(color::WHITE).set(ids.master, ui);
//    println!("Rect of {:?}", ui.rect_of(ids.master));
    //set the amount of text
    ids.text.resize(display_list.len(), &mut ui.widget_id_generator());

    let mut i = 0;
    for (x, y, text) in display_list {
        let w = widget::Text::new(text);
        let w_wh = w.get_wh(ui).unwrap();
        let rel_pos = rel(ui, w_wh, [*x, *y]);
        w.xy(rel_pos)
            .color(color::BLACK)
            .set(ids.text[i], ui);
        i += 1;
    }
}


/// The positioning behavior of conrad is that 0,0 is the middle of the widget.
/// This function, when given a ui cell, the widget, and a desired absolute position, will
/// return the necessary relative position to put the widget's top left corner at
/// the given coordinates
///
/// (0, 0) refers to the top left pixel
fn rel(ui: &conrod_core::UiCell, widget_wh: conrod_core::Dimensions, abs: conrod_core::Point)
    -> conrod_core::Point {
    //get the window bounds and offsets to apply
    let window_dim = ui.window_dim();
    let window_offset_x = -(window_dim[0] / 2.0);
    let window_offset_y = window_dim[1] / 2.0;

    //get the widget offsets
    let widget_offset_x = widget_wh[0] / 2.0;
    let widget_offset_y = -(widget_wh[1] / 2.0);

    let rel_x = abs[0] + window_offset_x + widget_offset_x;
    let rel_y = -(abs[1]) + window_offset_y + widget_offset_y;

    return [rel_x, rel_y] as conrod_core::Point;
}

// Generate a unique `WidgetId` for each widget.
// Generates the boilerplate for all the `button: conrod::widget::Id, ...`
widget_ids! {
    struct Ids {
        master,
        rectangle,
        oval,
        text[],
    }
}