use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::body::HttpBody;
use hyper::service::{make_service_fn, service_fn};
use shared::config::read_config;
use shared::system_state::Status::Idle;
use shared::system_state::{Status, SystemState};

async fn process_request(req: Request<Body>, state: Arc<Mutex<SystemState>>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/status") => {
            let status = state.lock().unwrap().status;
            *response.body_mut() = Body::from(format!("{status:?}"));
        },
        (&Method::POST, "/shutdown") => {
            let mut state = state.lock().unwrap();
            state.status = Status::Exiting;
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config(Path::new("./config.toml"))?;
    let state = Arc::new(Mutex::new(SystemState::new()));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));

    let make_service = make_service_fn(move |_| {
        let state = state.clone();
        let service = service_fn(move |req| process_request(req, state.clone()));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service).await;

    if let Err(e) = server {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
