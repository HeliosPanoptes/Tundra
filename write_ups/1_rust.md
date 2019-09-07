## How Browsers Download Web Pages

The primary goal of a web browser is to show the user some information identified by a 
URL. So, how does a browser get information out of a URL like 
http://example.org/index.html? Well, in short (and a network class will cover this in more 
detail), the browser parses the URL, connects to a server over the Internet, sends that 
server a request, receives a reply, and finally shows that to the user. 

This guide is for Rust. Be warned that Rust can be significantly lower level, and that convenient data operations in Python must be done explicitly in Rust. If you're ok with that, let's continue!	

## Parsing the URL

The first thing a browser does with a URL like `http://example.org/index.html` is to *parse* it, which means that the URL is split into parts. Here those parts are:

- The *scheme*, here `http`
- The *host*, here `example.org`
- The *path*, here `/index.html`

These parts play different roles: the host tells the browser who to get the information from, the scheme tells it how, and the path is something the browser tells the host to explain what information it wants. There are also optional parts to the URL. Sometimes, like in `http://localhost:8080/`, there is a *port*, which you can think of as telling you which door to the host's house to use; the default is `80`.[1] Sometimes there is also something tacked onto the end, a *fragment* like `#section` or a *query string* like `?s=term`.

In Python, there's a library called `urllib.parse` that can do this URL parsing for you, and I'm sure that there's a comparable version in Rust. However, I'm trying to avoid using libraries here,[3] so let's write a bad version ourselves. We'll start with the scheme—our browser only supports `http`, so we just need to check that the URL starts with `http://` and then strip that off:

```Rust
if !url.starts_with("http://") {
  panic!("Tundra only supports http");
}

let scheme_rest: Vec<_> = url.split("://").collect();
let rest: &str = &scheme_rest[1];
```

Next, the host and port come before the first `/`, while the path is that slash and everything after it:

```rust
let mut hostport = rest;
let mut pathfragment = "/";
if rest.contains('/') {
  let address: Vec<_> = rest.splitn(2, "/").collect();
  hostport = address[0];
  pathfragment = address[1];
}

let mut host: String = hostport.to_string();
let mut port: String = "80".to_string();
if hostport.contains(':') {
  let hostport_vec: Vec<_> = hostport.rsplitn(2, ":").collect();
  host = hostport_vec[1].to_string();
  port = hostport_vec[0].to_string();
}

let mut path: String = "/".to_string() + pathfragment;
let mut fragment: String = "".to_string();
if pathfragment.contains('#') {
  let pathfragment_vec: Vec<_> = pathfragment.rsplitn(2, "#").collect();
  path = "/".to_string() + pathfragment_vec[1];
  fragment = "#".to_string() + pathfragment_vec[0];
}
```

What was a 3-liner in Python has turned into a 23-liner, the main reason being we don't have access to the same list comprehension and sugar. The logic is the same, but explicitly laid out on more lines. Here, I'm using `splitn` to control how many sections of the vector I make. In this case, I only want to split once, making two sections, so I put in 2. Note that both the ports and the fragments are optional, with defaults being provided.

**Go further**: The syntax of URLs is defined in [RFC 3987](https://tools.ietf.org/html/rfc3986), which is pretty readable. Try to implement the full URL standard, including encodings for reserved characters.

**Go further**: [Data URLs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URIs)are a pretty interesting type of URL that embed the whole reasource into the URL. Try to implement them; most libraries have libraries that handle the `base64`encoding used in Data URLs.[4]

## Communicating with the host

With the URL parsed, a browser must connect to the host, explain what information it wants, and receive the host's reply.

### Connecting to the host

First, a browser needs to find the host on the Internet and make a connection.

Usually, the browser asks the operating system to make the connection for it. The OS then talks to a *DNS server*which converts[5] a host name like `example.org` into a *IP address* like `93.184.216.34`.[6] Then the OS decides which hardware is best for communicating with that IP address (say, wireless or wired) using what is called a *routing table*, and uses that hardware to send a sort of greeting to that IP address, to the specific port at that IP address that the browser indicated. Then there's a driver inside the OS that communicates with that hardware and send signals on a wire or whatever.[7] On the other side of that wire (or those airwaves) is a series of *routers*[8] which each send your message in the direction they think will take it toward that IP address.[9] Anyway, the point of this is that the browser tells the OS, hey, put me in touch with `example.org` on port `80`, and it does.

On many systems, you can set up this kind of connection manually using the `telnet`program, like this:

```
telnet example.org 80
```

You'll get output that looks like this:

```
Trying 93.184.216.34...
Connected to example.org.
Escape character is '^]'.
```



###### Installation

You might need to install `telnet`. Nowadays, it is usually disabled by default; on Windows, for example, you need to [go to Programs and Features / Turn Windows features on or off](https://www.lifewire.com/what-is-telnet-2626026)in the Control panel. On macOS, you can use the `nc` command as a replacement:

```
nc -v example.org 80
```

On Linux the `nc`command is usually available in the repos in a package called `netcat`or similar. The output with `nc`is a little different from `telnet`but it does basically the same thing. You can also install `telnet`on most Linux systems.



This means that the OS converted `example.org` to the IP address of `93.184.216.34`and was able to connect to it.[10]

 You can type text into the console and press enter to talk to example.org.
Requesting information from the host

Once it's been connected, the browser explains to the host what information it is looking for. In our case, the browser must do that explanation using the http protocol, and it must explain to the host that it is looking for /index.html. In HTTP, this request looks like this:

GET /index.html HTTP/1.0
Host: example.org

Here, the word GET means that the browser would like to receive information,[11] then comes the path, and finally there is the word HTTP/1.0 which tells the host that the browser speaks version 1.0 of HTTP.[12] There are several versions of HTTP, at least 0.9, 1.0, 1.1, and 2.0. The later standards add a variety of useful features, like virtual hosts, cookies, referrers, and so on, but in the interest of simplicity our browser won't use them yet. We're also not implementing HTTP 2.0; HTTP 2.0 is much more complex than the 1.X series, and is intended for large and complex web applications, which our browser won't much support, anyway. 

After the first line, each line contains a header, which has a name (like Host) and a value (like example.org). Different headers mean different things; the Host header, for example, tells the host who you think it is.[13] There are lots of other headers one could send, but let's stick to just Host for now.[14] Finally, after the headers are sent, you need to enter one blank line; that tells the host that you are done with headers.

Enter all this into telnet and see what happens. Remember to leave add one more blank line after the line that begins with Host.

Go further: The HTTP/1.0 standard is also known as RFC 1945. The HTTP/1.1 standard is RFC 2616, so if you're interested in Connection and keep-alive, look there.
Our own Telnet

So far we've communicated with another computer using telnet. But it turns out that telnet is quite a simple program, and we can do the same programmatically, without starting another program and typing into it.

To communicate with another computer, the operating system provides a feature called "sockets". When you want to talk to other computers (either to tell them something, or to wait for them to tell you something), you create a socket, and then that socket can be used to send information back and forth. Sockets come in a few different kinds, because there are multiple ways to talk to other computers:

Rust sockets are a little bit different. There are different types, such as UDP and TCP listeners and streams. we want a TCP stream, so we instantiate and connect a socket like so:

```rust
let address = format!("{}:{}", host, port);
match TcpStream::connect(address) {
  Ok(mut socket) => {
    ...
  Err(_e) => {
    println!("Failed to receive data");
    exit(1);
  }
```

(Fun sidenote, this code is simpler than the equivalent python code for once)

Once you've made the connection, you can send it some data using the `write` method. 

```rust
    ...
    let request_string = format!("GET {} HTTP/1.1\r\n\
                                  Host: {}\r\n\
                                  Connection: close\r\n\r\n",
                                  path, host);

    socket.write(request_string.as_bytes()).unwrap();
```

We've used two error catching constructs here. The call to `connect` results in an `Option<T>`, which can resolve to either `None` or `Some(T)`. `match` lets us explicitly check and handle errors on our own by using `Ok` and `Err`. The call to `unwrap` later will either return the value contained inside of the `Option`, or it will trigger a `panic!`. Since this is a toy browser, we're ok with letting it crash and burn if it connects but the socket can't write for some reason.

When writing the data to the socket, we also have to remember to send the bytes, not the internal representation that Rust might use, so we use `as_bytes()`.

If you dig around with the `write` call, you'll notice that it returns a number. That tells you how many bytes of data you sent to the other computer; if, say, your network connection failed midway through sending the data, you might want to know how much you sent before the connection failed.

Go further: You can find out more about the "sockets" API on Wikipedia. Python mostly implements that API directly.

Go further: Secure HTTP (the https protocol) uses something called TLS to encrypt all traffic on a socket. TLS is pretty complicated, but your language might have a simple library for using it. 

## The host's reply

If you look at your `telnet`session, you should see that the other computer's response starts with this line:

```
HTTP/1.0 200 OK
```

That tells you that the host confirms that it, too, speaks `HTTP/1.0`, and that it found your request to be "OK" (which has a corresponding numeric code of 200). You may be familiar with `404 Not Found`. That's something the server could say instead of `200 OK`, or it could even say `403 Forbidden`or `500 Server Error`. There are lots of these codes, and they have a pretty neat organization scheme:

- The 100s are informational messages
- The 200s mean you were successful
- The 300s mean you need to do a follow-up action (usually to follow a redirect)
- The 400s mean you sent a bad request
- The 500s mean the server handled the request badly

Note the genius of having two sets of error codes (400s and 500s): which one you get tells you who the server thinks is at fault (the server or the browser). You can find a full list of the different codes [on Wikipedia](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes).

After the `200 OK` line, the server sends its own headers. When I did this, I got these headers (but yours may differ):

```
Cache-Control: max-age=604800
Content-Type: text/html; charset=UTF-8
Date: Mon, 25 Feb 2019 16:49:28 GMT
Etag: "1541025663+ident"
Expires: Mon, 04 Mar 2019 16:49:28 GMT
Last-Modified: Fri, 09 Aug 2013 23:54:35 GMT
Server: ECS (sec/96EC)
Vary: Accept-Encoding
X-Cache: HIT
Content-Length: 1270
Connection: close
```

There is **a lot** here, including information about the information you are requesting (`Content-Type`, `Content-Length`, and `Last-Modified`), information about the server (`Server`, `X-Cache`), information about how long the browser should cache this information (`Cache-Control`, `Expires`, `Etag`), and a bunch of random other information. Let's move on for now.

After the headers there is a blank line, and then there is a bunch of HTML code. Your browser knows that it is HTML because of the `Content-Type` header, which says that it is `text/html`.[19] That HTML code is the *body* of the server's reply.

Let's read the HTTP response programmatically. Generally, you'd use the `read` function on sockets, which gives whatever bits of the response have already arrived. Then you write a loop that collects bits of the response as they arrive. However, we will assume that socket will close, or that we will recieve an EOF from the request when all the bits have arrived. This makes our lives dramatically simpler, but again, not the most robust thing we could do.

For this, we'll use `read_to_end`, and assume that everything we need is contained in the buffer.

```rust
match socket.read_to_end(&mut buf) {
  Ok(_) => {
    let response = String::from_utf8_lossy(&buf);
    ...
  },
  Err(_e) => {
    println!("Failed to receive data");
    exit(1);
  }
}
```

Here, we simply convert the response into utf8, with characters that it can't represent as an ugly �. 

Let's split the response into pieces. The first line is the status line, then the headers, and then the body: 

```rust
    ...
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
      headers.insert(header_line[0].to_string().to_lowercase().trim(),
        header_line[1].to_string().to_lowercase().trim());
    };

    return (headers, body);
```

For the headers, I split each line at the first colon and make a dictionary (a key-value map) of header name to header value. Headers are case-insensitive, so I normalize them to lower case. Also, white-space is insignificant in HTTP header values, so I strip off extra whitespace at the beginning and end

**Go further**: Many common (and uncommon) HTTP headers are described on Wikipedia.

**Go further**: Instead of calling decode on the whole response, parse the headers first and then use the Content-Type header to determine which codec to decode the body with. 

##Displaying the HTML

The HTML code that the server sent us defines the content you see in your browser window when you go to http://example.org/index.html. I'll be talking much, much more about HTML in the future posts, but for now let me keep it very simple.

In HTML, there are tags and text. Each tag starts with a `<` and ends with a `>`; generally speaking, tags tell you what kind of thing some content is, while text is the actual content.[22] Most tags come in pairs of a start and an end tag; for example, the title of the page is enclosed a pair of tags: `<title>` and `</title>`. Each tag, inside the angle brackets, has a tag name (like title here), and then optionally a space followed by attributes, and its pair has a `/` followed by the tag name (and no attributes). Some tags do not have pairs, because they don't surround text, they just carry information. For example, on `http://example.org/index.html`, there is the tag:

`<meta charset="utf-8" />`

This tag once again repeats that the character set with which to interpret the page body is `utf-8`. Sometimes, tags that don't contain information end in a slash, but not always, because web developers aren't always so diligent.

The most important HTML tag is called `<body>` (with its pair, `</body>`). Between these tags is the content of the page; outside of these tags is various information about the page, like the aforementioned title, information about how the page should look (`<style>` and `</style>`), and metadata using the aforementioned `<meta/>` tag.

So, to create our very very simple web browser, let's take the page HTML and print all the text in it (but not the tags):[23] 

```rust
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
```

This code is pretty complex. It goes through the request body character by character, and it has two states: `in_angle`, when it is currently between a pair of angle brackets, and `!in_angle`. When the current character is an angle bracket, changes between those states; when it is not, and it is not inside a tag, it prints the current character.

**Go further**: The `Accept-Encoding` header allows a web browser to advertise that it supports [receiving compressed documents](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding). Try implementing support for one of the common compression formats (like `deflate` or `gzip`)!

## Summary

This post went from an empty file to a rudimentary web browser that can:

- Parse a URL into a host, a port, a path, and a fragment.
- Connect to that host at that port using `sockets`
- Send an HTTP request to that host, including a `Host ` header
- Split the HTTP response into a status line, headers, and a body
- Print the text (and not the tags) in the body

Yes, this is still more of a command-line tool than a web browser, but what we have already has some of the core capabilities of a browser.

Collect the code samples given in this post into a file. You should have three functions:

- `parse(url)`

  Takes in a string URL and returns a host string, a numeric port, a path string, and a fragment string. The path should include the initial slash, and the fragment should *not*include the initial `#`.

- `request(host, port, path)`

  Takes in a host, a port, and a path; connects to the host/port using sockets; sends it an HTTP request (including the `Host` header); splits the response into a status line, headers, and a body; checks that the status line starts with `HTTP/1.0` and has the status code `200`[25]; and then returns the headers as a dictionary and the body as a string.

- `show(body)`

  Prints the text, but not the tags, in an HTML document

  It should be possible to string these functions together like so:

  ```
  import sys
  host, port, path, fragment = parse(sys.argv[1])
  headers, body = request(host, port, path)
  show(body)
  ```

  This code uses the `sys`library to read the first argument (`sys.argv[1]`) from the command line to use as a URL.

## Exercises

- Along with `Host`, send the `User-Agent`header in the `request`function. Its value can be whatever you want—it identifies your browser to the host.
- Add support for the `file://`scheme to `parse`. Unlike `http://`, the file protocol has an empty host and port, because it always refers to a path on your local computer. You will need to modify `parse`to return the scheme as an extra output, which will be either `http`or `file`. Then, you'll need to modify `request`to take in the scheme and to "request" `file`URLs by calling `open`on the path and reading it. Naturally, in that case, there will be no headers.
- Error codes in the 300 range refer to redirects. Change the browser so that, for 300-range statuses, the browser repeats the request with the URL in the `Location`header. Note that the `Location`header might not include the host and scheme. If it starts with `/`, prepend the scheme and host. You can test this with with the URL http://tinyurl.com/yyutdgeu, which should redirect back to this page.
- Only show the text of an HTML document between `<body>`and `</body>`. This will avoid printing the title and various style information. You will need to add additional variables `in_body`and `tag`to that loop, to track whether or not you are between `body`tags and to keep around the tag name when inside a tag.
- Support multiple file formats in `show`: use the `Content-Type`header to determine the content type, and if it isn't `text/html`, just show the whole document instead of stripping out tags and only showing text in the `<body>`.

 