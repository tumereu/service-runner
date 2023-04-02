use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::{env, thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use shared::config::read_config;
use shared::system_state::{Status, SystemState};

async fn process_request(req: Request<Body>, state: Arc<Mutex<SystemState>>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/status") => {
            let state: SystemState = state.lock().unwrap().clone();
            *response.body_mut() = Body::from(serde_json::to_string(&state).unwrap());
        },
        (&Method::POST, "/shutdown") => {
            {
                let mut state = state.lock().unwrap();
                state.status = Status::Exiting;
            }
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config_dir: String = env::args().collect::<Vec<String>>()
        .get(1)
        .ok_or("Specify the configuration directory in order to run the app")?
        .clone();

    let config = read_config(&config_dir)?;
    let state = Arc::new(Mutex::new(SystemState::new()));
    // Initialize a shutdown signal
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let runner_state = state.clone();
    thread::spawn(move || {
        loop {
            {
                let mut state = runner_state.lock().unwrap();
                let status = state.status;
                if status == Status::Exiting {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10))
        }

        println!("Exiting server");
        shutdown_tx.send(())
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));

    let make_service = make_service_fn(move |_| {
        let state = state.clone();
        let service = service_fn(move |req| process_request(req, state.clone()));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await;

    if let Err(e) = server {
        eprintln!("Unexpected error in server: {}", e);
    }

    Ok(())
}