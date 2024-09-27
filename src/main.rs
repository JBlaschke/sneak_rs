#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use env_logger;

use clap::{Parser, Subcommand};


#[derive(Subcommand, Clone, Debug)]
enum Mode {
    Server {
        #[arg(long)]
        host: String,
        #[arg(short, long)]
        port: u16
    },
    Client {
        #[arg(short, long)]
        bind: String,
        #[arg(short, long)]
        port: u16
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Sneak past firewalls by making all traffic look like HTTP(s)",
    arg_required_else_help = true
)]
struct Args {
    #[command(subcommand)]
    pub mode: Mode,

    #[clap(long)]
    pub tunnel_address: String,

    #[clap(long)]
    pub tunnel_port: u16
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}



use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use reqwest::blocking::Client;
use std::thread;

fn client() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3333")?;
    let http_client = Client::new();

    info!("Client-side proxy listening on port 3333");

    trace!("Making initial connection request");

    let client_id = http_client
        .get("http://0.0.0:8080/new_client")
        .send()
        .unwrap();

    trace!("Using client ID: {:?}", client_id);

    for incoming in listener.incoming() {
        match incoming {
            Ok(mut mysql_client_stream) => {
                let http_client = http_client.clone();
                thread::spawn(move || {
                    handle_mysql_client(mysql_client_stream, http_client);
                });
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }

    Ok(())
}

fn handle_mysql_client(mut mysql_client_stream: TcpStream, http_client: Client) {
    let mut buffer = [0u8; 1024];
    trace!("Started handler for new SQL connection");
    loop {
        trace!("Reading data");
        let bytes_read = match mysql_client_stream.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from MySQL client: {}", e);
                return;
            }
        };

        trace!("Processing data: {:?}", buffer);

        if bytes_read == 0 {
            trace!("Nothing to do!");
            return;
        }

        // Send the data to the server via HTTP
        let response = http_client
            .post("http://0.0.0.0:8080")
            .body(buffer[..bytes_read].to_vec())
            .send();

        match response {
            Ok(mut resp) => {
                let mut response_buffer = vec![];
                resp.read_to_end(&mut response_buffer).unwrap();
                mysql_client_stream.write_all(&response_buffer).unwrap();
            }
            Err(e) => eprintln!("Failed to send request to server: {}", e),
        }
    }
}



// use std::io::{Read, Write};
// use std::net::TcpStream;
// use std::sync::{Arc, Mutex};
// use std::thread;
use tiny_http::{Server, Response, Request, Method};

fn server() {
    let server = Server::http("0.0.0.0:8080").unwrap(); // HTTP server listening on port 8080
    info!("Server-side proxy listening on port 8080");

    for request in server.incoming_requests() {
        trace!("Received incoming request {:?}", request);

        let method = request.method().clone();
        let path = request.url();

        // Match the path and respond accordingly
        match (method, path) {
            (Method::Get, "/new_client") => {
                trace!("Received request for new client, returning:");
            },
            _ => {}
        }

        thread::spawn(move || {
            handle_http_request(request);
        });
    }
}

fn handle_http_request(mut request: Request) {
    let mut body = vec![];
    request.as_reader().read_to_end(&mut body).unwrap();

    // Connect to MySQL server
    let mut mysql_server_stream = TcpStream::connect("127.0.0.1:3306").unwrap();

    // Send the received data to the MySQL server
    mysql_server_stream.write_all(&body).unwrap();

    // Receive the response from the MySQL server
    let mut response_buffer = [0u8; 1024];
    let bytes_read = mysql_server_stream.read(&mut response_buffer).unwrap();

    // Send the MySQL response back to the client-side component
    let response = Response::from_data(response_buffer[..bytes_read].to_vec());
    request.respond(response).unwrap();
}






fn main() {
    let args = Args::parse_args();
    env_logger::init();

    trace!("Started sneak with inputs: {:?}", args);

    match args.mode {
        Mode::Client { bind, port } => {
            info!("Starting Client listening to: {:?}:{:?}", bind, port);
            client();
        }

        Mode::Server { host, port } => {
            info!("Starting Server forwarding: {:?}:{:?}", host, port);
            server();
        }
    }
}
