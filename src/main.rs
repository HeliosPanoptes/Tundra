#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
//#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;
extern crate regex;
extern crate petgraph;


use std::env;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::collections::HashMap;
use regex::Regex;
use glium::Surface;
use conrod_core::{color, widget, Colorable, Widget, Positionable, Sizeable};
//Trees, who knew right?
use petgraph::prelude;
use petgraph::Graph;
// linked trees are pretty hard due to rust's guarantees

//conrod support functions
mod support;

const SCROLL_STEP: f64 = 20.0;
const INIT_WIDTH: f64 = 800.0;
const INIT_HEIGHT: f64 = 600.0;
const FONT_SIZE: u32 = 16;
const LINE_SPACING: f64 = 1.2;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must provide one and only one url");
    }
    let url = &args[1].as_str();

    let mut tundra = Tundra::new();

    tundra.browse(url);
}

struct WindowUi {
    ui: conrod_core::Ui,
    events_loop: glium::glutin::EventsLoop,
    display: support::GliumDisplayWinitWrapper,
    font: conrod_core::text::font::Id,
    font_b: conrod_core::text::font::Id,
    font_i: conrod_core::text::font::Id,
    font_bi: conrod_core::text::font::Id,
}

enum DisplayListItem {
    Text(DisplayListText),
}

struct DisplayListText {
    x: f64,
    y: f64,
    text: String,
    font: conrod_core::text::font::Id,
    color: conrod_core::color::Color,
    underline: bool,
}

struct ElementNode {
    tag: String,
    attributes: HashMap<String, String>,
}

impl ElementNode {
    fn new(tag: String) -> ElementNode {
        return ElementNode { tag, attributes: HashMap::new() };
    }

    fn add_attr(&mut self, k: String, v: String) {
        self.attributes.insert(k, v);
    }
}

struct TextNode {
    text: String,
}

impl TextNode {
    fn new(text: String) -> TextNode {
        return TextNode{ text }
    }
}

enum LayoutNode {
    Element(ElementNode),
    Text(TextNode),
}

struct LayoutState{
    layout_tree: petgraph::Graph<LayoutNode, i32>,
}

impl LayoutState {
    /// Adds a node into the tree and returns it's index. Useful for setting up a root node. Use add_child to add a node and link to it
    fn add_node(&mut self, node: LayoutNode) -> petgraph::graph::NodeIndex {
        self.layout_tree.add_node(node)
    }
    /// Adds and links a child into the tree. Returns the child's index
    fn add_child(&mut self, parent_index: petgraph::graph::NodeIndex, child: LayoutNode) -> petgraph::graph::NodeIndex {
        let child_id = self.layout_tree.add_node(child);
        self.layout_tree.add_edge(parent_index, child_id, 1);
        return child_id;
    }

    fn parent_of(&self, child_index: petgraph::graph::NodeIndex) -> petgraph::graph::NodeIndex {
        let parent_edge = self.layout_tree.first_edge(child_index, petgraph::Incoming);
        let (parent, _) = tree.edge_endpoints(parent_edge).unwrap();
        return parent;
    }

    fn get_layout_node(&self, node_index: petgraph::graph::NodeIndex) -> &LayoutNode {
        &self.layout_tree[node_index]
    }
}

enum Token {
    Text(String),
    Tag(String),
}

struct Tundra {
    window_height : f64,
    window_width : f64,
    scroll_y : f64,
    tokens: Vec<Token>,
    display_list: Vec<DisplayListItem>,
    layout_state: LayoutState
}

impl Tundra {

    fn new() -> Tundra {
        return Tundra {
            window_height: INIT_HEIGHT,
            window_width: INIT_WIDTH,
            scroll_y: 0.0,
            tokens: Vec::new(),
            display_list: Vec::new(),
            layout_state: LayoutState {layout_tree: Graph::<LayoutNode, i32>::new()}
        };
    }
    /// A convenience method that combines all of the steps for the browser to
/// display the page
///
/// At this point it holds all the state... maybe not the best, but we'll work with it
/// until it doesn't work anymore
/// I may have hit that point with scrolling. I need to store the state, so it needs to be in an object
    fn browse(&mut self, url: &str) {
        // construct our `Ui`.
        let mut window_ui = self.set_up_window();

        let (host, port, path, _fragment) = self.parse_address(url);
        let (_headers, body) = self.request(&host, &port, &path);
        // test case for spaces and bounding rects being applied correctly
        //   correct: tight boxes and a proper space. incorrect: extra space in the boxes and overlap
        //let body = "<p>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa a</p>".to_string();
        self.lex(body);
        self.layout(&mut window_ui);

        self.render(&mut window_ui);
    }

    fn parse_address(&self, url: &str) -> (String, String, String, String) {
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

    fn request(&self, host: &str, port: &str, path: &str) -> (HashMap<String, String>, String) {
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

    fn lex(&mut self, source: String) {
        let mut tokens : Vec<Token> = Vec::new();
        let mut text : String = "".to_string();
        for c in source.chars() {
            if c == '<' {
                //store the text so far and reset
                if !text.is_empty() {
                    tokens.push(Token::Text(text.to_string()));
                }
                text = "".to_string();
            } else if c == '>' {
                //store the tag and reset
                tokens.push(Token::Tag(text.to_string()));
                text = "".to_string();
            }
            else {
                text.push(c);
            }
        }
        self.tokens = tokens;
    }

    /// Takes text and returns a display-list of the format (x, y, text)
    fn layout(&mut self, window_ui: &mut WindowUi) {
        // clear the display list, especially when re-laying out
        self.display_list.clear();

        //base position
        let mut x: f64 = 13.0;
        let mut y: f64 = 13.0;

        //convenience
        let ref ui = window_ui.ui;
        let f = window_ui.font;
        let f_b = window_ui.font_b;
        let f_i = window_ui.font_i;
        let f_bi = window_ui.font_bi;

        let mut current_font = f;
        let mut current_color = color::BLACK;
        let mut bold = false;
        let mut italic = false;
        let mut terminal_space = true;
        let mut in_body = false;
        let mut in_comment = false;
        let mut underline = false;

        let mut current_node: Option<petgraph::graph::NodeIndex> = None;
        let mut tree = &mut self.layout_state.layout_tree;

        let attribute_re: Regex = Regex::new(r#"((\w+)=((\w+)|(".*?")))+"#).unwrap();

        for token in self.tokens.iter() {

            match token{
                Token::Text(text) => {
                    let new_node = LayoutNode::Text(TextNode::new(text.to_owned()));
                    let new_node = tree.add_node(new_node); //get the node index instead
                    match current_node {
                        Some(node) => { tree.add_edge(node, new_node, 1); },
                        None => { panic!("Tried to add a text node to an empty layout graph"); },
                    }
                },
                Token::Tag(tag) => {
                    if !tag.starts_with("/") { //open tag
                        let self_closing;
                        if tag.ends_with("/") {self_closing = true;}
                        else { self_closing = false;}

                        //tag name
                        let tag_parts: Vec<&str> = tag.splitn(2," ").collect();
                        let tag_name = tag_parts[0];
                        let mut new_element = ElementNode::new(tag_name.to_string());
                        if tag_parts.len() > 1 {
                            let attributes = tag_parts[1];
                            for capture in attribute_re.captures_iter(&attributes) {
                                //use the capture groups to split the keys and values
                                new_element.add_attr(capture[2].to_string().to_lowercase(), capture[3].to_string());
                            }
                        }
                        let new_node = tree.add_node(LayoutNode::Element(new_element));
                        match current_node {
                            Some(node) => { tree.add_edge(node, new_node, 1); },
                            None => (),
                        }
                        //enter the current node, unless it's self-closing
                        if !( self_closing || ["br", "link", "meta"].contains(&tag_name)) {
                            current_node = Some(new_node);
                        }
                    } else { //close tag
                        //go up to the parent
                        let parent_edge = tree.first_edge(current_node.unwrap(), petgraph::Direction::Incoming).unwrap();
                        let (parent, _) = tree.edge_endpoints(parent_edge).unwrap();
                        let current_node = parent;
                    }
                }
            }





            //Calculate whitespace of current font
            let w = widget::Text::new(" ")
                .font_id(current_font)
                .font_size(FONT_SIZE);
            let linespace_h = w.get_h(ui).unwrap();
            let whitespace_w = w.get_w(ui).unwrap();



            match token {
                Token::Text(text) => {
                    //only print the body
                    if !in_body {
                        continue;
                    }
                    let words = text.split_whitespace();
                    let wordcount = words.clone().count();

                    if text.starts_with(" ") && !terminal_space {
                        x += whitespace_w;
                    }

                    for (i, word) in words.enumerate() {
                        //make a dummy version to let conrod do the hard work of the layout.
                        // *** MUST SET FONT BEFORE GETTING DIMENSIONS ***
                        let w = widget::Text::new(&word)
                            .font_id(current_font)
                            .font_size(FONT_SIZE)
                            .line_spacing(LINE_SPACING);
                        let w_wh = w.get_wh(ui).unwrap();

                        if (x + w_wh[0]) > (self.window_width - 13.0) {
                            y += w_wh[1] * LINE_SPACING;
                            x = 13.0;
                        };

                        let display_list_text = DisplayListText {
                            x, y,
                            text: word.to_owned(),
                            font: current_font,
                            color: current_color,
                            underline };
                        self.display_list.push(DisplayListItem::Text(display_list_text));

                        let mut whitespace = whitespace_w;
                        if i == (wordcount - 1) {
                            whitespace = 0.0;
                        };
                        x += w_wh[0] + whitespace;
                    }
                    // Add a whitespace if the last character is a space
                    terminal_space = text.ends_with(" ");
                    if terminal_space && wordcount > 0 {
                        x += whitespace_w;
                    }
                },

                Token::Tag(tag) => {
                    //handle comments
                    if tag.starts_with("!--") {
                        in_comment = true;
                    }
                    // Will skip multi-tag comments
                    if tag.ends_with("--") || in_comment {
                        in_comment = false;
                        continue;
                    }
                    //split a tag into tag name and attributes dictionary
                    let mut tag_parts = tag.split_whitespace();
                    let tag_name = tag_parts.nth(0).unwrap();
                    //handle the start of the document
                    if tag_name.contains("?xml") || tag_name.contains("DOCTYPE") {
                        continue; //handle by not handling
                    }

                    let mut attributes: HashMap<&str, &str> = HashMap::new();
                    for attribute in tag_parts {
                        let separated: Vec<_>  = attribute.splitn(2,"=").collect();
                        if separated.len() < 2 {
                            break; //ignore parsing errors, especially in the case of self-closing tags
                        }
                        attributes.insert(separated[0], separated[1]);

                    }
                    match tag_name {
                        "body" => in_body = true,
                        "/body" => in_body = false,
                        "i" => italic = true,
                        "/i" => italic = false,
                        "b" => bold = true,
                        "/b" => bold = false,
                        "/p" | "br" => {
                            terminal_space = true;
                            x = 13.0;
                            y += linespace_h * LINE_SPACING + 16.0;
                        },
                        "h1" => {
                            bold = true;
                            x = 13.0;
                            y += linespace_h * LINE_SPACING + 16.0;
                        },
                        "/h1" => {
                            bold = false;
                            terminal_space = true;
                            x = 13.0;
                            y += linespace_h * LINE_SPACING + 16.0;
                        },
                        "a" => {
                            underline = true;
                            current_color = color::BLUE;
                        },
                        "/a" => {
                            underline = false;
                            current_color = color::BLACK;
                        },

                        _ => ()
                    }

                    //set the font style
                    match (bold, italic) {
                        (false, false) => current_font = f,
                        (true, false)  => current_font = f_b,
                        (false, true)  => current_font = f_i,
                        (true, true)   => current_font = f_bi,
                    }
                }
            };
        };
    }

    fn set_up_window(&self) -> WindowUi {
        let mut ui = conrod_core::UiBuilder::new([INIT_WIDTH as f64, INIT_HEIGHT as f64]).build();
        // Build the window.
        let events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Tundra")
            .with_dimensions((INIT_WIDTH, INIT_HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let display = support::GliumDisplayWinitWrapper(display);

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let font = ui.fonts.insert_from_file(font_path).unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Bold.ttf");
        let font_b = ui.fonts.insert_from_file(font_path).unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Italic.ttf");
        let font_i = ui.fonts.insert_from_file(font_path).unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-BoldItalic.ttf");
        let font_bi = ui.fonts.insert_from_file(font_path).unwrap();
//        let font_path = assets.join("fonts/PingFang-Regular.ttf");
//        ui.fonts.insert_from_file(font_path).unwrap();

        return WindowUi {
            ui,
            events_loop,
            display,
            font,
            font_b,
            font_i,
            font_bi,
        }
    }

    fn render(&mut self, mut window_ui: &mut WindowUi) {
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
                    // scrolling logic here
                    // Since we're doing this ourselves, we can't rely on the scrolling behavior
                    // of the widgets. The button presses have to be intercepted here in order
                    // to affect the global state (In this case scroll_y)
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
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Down),
                                state: glium::glutin::ElementState::Pressed,
                                ..
                            },
                            ..
                        } => self.scroll_down(),
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Up),
                                state: glium::glutin::ElementState::Pressed,
                                ..
                            },
                            ..
                        } => self.scroll_up(),
                        _ => (),
                    },
                    _ => (),
                }
            } //...end events loop

            // Instantiate all widgets in the GUI.
            self.set_text(window_ui.ui.set_widgets(), ids);

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = window_ui.ui.draw_if_changed() {
                renderer.fill(&window_ui.display.0, primitives, &image_map);
                let mut target = window_ui.display.0.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&window_ui.display.0, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }

            // Re-do layout if necessary
            if window_ui.ui.win_w != self.window_width || window_ui.ui.win_h != self.window_height {
                self.window_width = window_ui.ui.win_w;
                self.window_height = window_ui.ui.win_h;
                self.layout(&mut window_ui);
//                window_ui.ui.needs_redraw();
            }
        } //...end draw loop
    }

    fn set_text(&mut self, ref mut ui: conrod_core::UiCell, ids: &mut Ids) {
        // Construct our main `Canvas` tree.
        // The canvas here is just acting like a glorified background.
        // Normally we would use them to lay out the ui, and anchor elements to them, but
        // we're doing everything on our own, so nahhhh
        let _canvas = widget::Canvas::new()
            .color(color::WHITE)
            .set(ids.master, ui);

        //set the amount of text
        //We could be more memory efficient by only taking up space we need, but eh
        ids.text.resize(self.display_list.len(), &mut ui.widget_id_generator());
        ids.underlines.resize(self.display_list.len(), &mut ui.widget_id_generator());
//        ids.rectangles.resize(self.display_list.len(), &mut ui.widget_id_generator());

        //manual loop because I can't figure out how to borrow the display_list text
//        for (i, item) in self.display_list.iter().enumerate() {
        for i in 0..self.display_list.len() {
            let item = &self.display_list[i];
            match item {
                DisplayListItem::Text(text_item) => {
                    let x = text_item.x;
                    let y = text_item.y;
                    let color = text_item.color;
                    let underline = text_item.underline;

                    if y > self.scroll_y && y < self.scroll_y + self.window_height as f64 {
                        let text = &text_item.text.clone();

                        // *** MUST SET FONT BEFORE GETTING DIMENSIONS ***
                        let w = widget::Text::new(text)
                            .color(color)
                            .font_id(text_item.font)
                            .font_size(FONT_SIZE)
                            .line_spacing(LINE_SPACING);
                        let w_wh = w.get_wh(ui).unwrap();
                        let rel_pos = self.rel(ui, w_wh, [x, y - self.scroll_y]);
                        w.xy(rel_pos)
                            .h(w_wh[1] + 2.0) //add two pixels to the bottom to make underlines look good
                            .set(ids.text[i], ui);

                        if underline {
                            //The start and end describe the angle and magnitude
                            let line_start = [0.0, 0.0];
                            let line_end = [w_wh[0], 0.0];
                            let _underline_w = widget::Line::centred(line_start, line_end)
                                .bottom_left_of(ids.text[i])
                                .color(color)
                                .set(ids.underlines[i], ui);
                        }

                        //draw a rectangle around the word widget as well (debug help)
                        //let r = widget::BorderedRectangle::new(w_wh).xy(rel_pos).color(color::TRANSPARENT).set(ids.rectangles[i], ui);
                    }
                },
                _ => (),
            }
        }
    }


    /// The positioning behavior of conrad is that 0,0 is the middle of the widget.
    /// This function, when given a ui cell, the widget, and a desired absolute position, will
    /// return the necessary relative position to put the widget's top left corner at
    /// the given coordinates
    ///
    /// (0, 0) refers to the top left pixel
    fn rel(&mut self, ui: &conrod_core::UiCell, widget_wh: conrod_core::Dimensions, abs: conrod_core::Point)
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

    fn scroll_down(&mut self) {
        self.scroll_y += SCROLL_STEP;

        // Don't scroll past the bottom of the page
        let last_item = self.display_list.last().unwrap();
        let last_y;
        match last_item {
            DisplayListItem::Text(item) => last_y = item.y,
        };

        if self.scroll_y > last_y - self.window_height as f64 {
            self.scroll_y = last_y - self.window_height as f64;
        }
    }

    fn scroll_up(&mut self) {
        self.scroll_y -= SCROLL_STEP;

        if self.scroll_y < 0.0 {
            self.scroll_y = 0.0;
        }
    }
}


// Generate a unique `WidgetId` for each widget.
// Generates the boilerplate for all the `button: conrod::widget::Id, ...`
widget_ids! {
        struct Ids {
            master,
            rectangle,
            oval,
            text[],
            underlines[],
            dummy_text, //for use in laying out text
            rectangles[],
        }
    }