use hyper::{Body, Response, Error, Request, Client};
use hyper::client::HttpConnector;
use hyper::http::request::Parts;
use log::{debug};
use crate::config_parser;

#[derive(Clone)]
pub struct Controller {
    client: Client<HttpConnector>,
    config: config_parser ::GatewayConfig
}

pub trait ControllerInterface {
    fn new(config: config_parser::GatewayConfig) -> Controller;
}

impl ControllerInterface for Controller {
    fn new(config: config_parser::GatewayConfig) -> Controller {
        Controller {
            client: Client::new(),
            config
        }
    }
}

impl Controller {
    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        let path = req.uri().path();

        // Check if the requested path is the health-check endpoint
        if path == "/health-check" && req.method() == hyper::Method::GET {
            return self.health_check();
        }

        let service_config = match self.get_service_config(path) {
            Some(service_config) => service_config,
            None => { return self.not_found() }
        };

        let (parts, body) = req.into_parts();

        let downstream_request = self.build_downstream_request(parts, body, service_config).await?;

        match self.forward_request(downstream_request).await {
            Ok(response) => Ok(response),
            Err(_) => self.not_found()
        }
    }

    fn not_found(&self) -> Result<Response<Body>, Error> {
        debug!("Responding with 404 Not Found");
        let mut response = Response::new(Body::from("404 Not Found"));
        *response.status_mut() = hyper::StatusCode::NOT_FOUND;
        Ok(response)
    }

    fn service_unavailable<T>(&self, reason: T) -> Result<Response<Body>, Error>
        where
            T: Into<Body>,
    {
        debug!("Responding with 503 Service Unavailable");
        let mut response = Response::new(reason.into());
        *response.status_mut() = hyper::StatusCode::SERVICE_UNAVAILABLE;
        Ok(response)
    }
    fn get_service_config(&self, path: &str) -> Option<&config_parser::ServiceConfig> {
        let option = self.config.services.iter().find(|service| service.path == path);
        match option {
            Some(service_config) => Some(service_config),
            None => None
        }
    }


    // Return a 200 response for the health check
    fn health_check(&self) -> Result<Response<Body>, Error> {
        let response = Response::new(Body::from("OK"));
        debug!("Responding with 200 OK for health check");
        Ok(response)
    }

    async fn build_downstream_request(&self, parts: Parts, body: Body, service_config: &config_parser::ServiceConfig) -> Result<Request<Body>, Error> {

        // build uri
        let req = Request::from_parts(parts, body);
        let uri = format!("{}:{}{}", service_config.target_service, service_config.target_port, req.uri().path());


        // build request
        let mut builder = Request::builder().uri(uri).method(req.method()).version(req.version());
        *builder.headers_mut().unwrap() = req.headers().clone();

        // build body
        let downstream_request = builder.body(req.into_body()).expect("Unable to build downstream request");

        Ok(downstream_request)
    }

    async fn forward_request(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        match self.client.request(req).await {
            Ok(res) => {
                // If the request is successful, return the response
                debug!("Request forwarded successfully");
                Ok(res)
            }
            Err(_) => {
                // If there is an error connecting to the requested service, return an error
                debug!("Failed to forward request");
                self.service_unavailable("Failed to forward request")
            }
        }
    }
}