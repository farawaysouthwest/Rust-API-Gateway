use std::net::SocketAddr;
use std::sync::Arc;
use hyper::Server;
use hyper::service::{make_service_fn, service_fn};
use log::{debug, info};
use simple_logger::SimpleLogger;
use crate::controller::ControllerInterface;

mod config_parser;
mod controller;


#[tokio::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();

    let config_path = match args.len() {
        1 => "config.yaml",
        2 => &args[1],
        _ => panic!("Too many arguments")
    };

    // Initialize the logger.
    SimpleLogger::new().init().expect("Unable to initialize logger");

    // Load the config from the config file, and parse the port.
    let config = config_parser::load_config(config_path);
    let port = config.get_port();

    // Create a new controller with the config, wrapped in an Arc so it can be shared between threads.
    let controller = Arc::new(controller::Controller::new(config));

    // Create a SocketAddr from the port.
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // Log that the server is starting
    info!("Starting server on {}", addr);

    // Create a new service to handle requests.
    let make_service = make_service_fn(move |_conn| {
        let controller = controller.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let controller = controller.clone();
                async move {
                    debug!("Received request: {:?}", req);
                    return controller.handle_request(req).await;
                }
                }))
            }
        });

    // Start the server.
    let server = Server::bind(&addr).serve(make_service);

    // Wait for the server to exit.
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}


