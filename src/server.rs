use crate::MAP;
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::{resolve, ResponseBuilder};
use std::convert::Infallible;
use std::error::Error;
use std::io::Error as IoError;
use std::net::SocketAddr;

pub async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Starting http server...");
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler)) });
    let server = Server::bind(&addr).serve(service);
    log::debug!("Listening on http://{}", addr);
    return server.await.map_err(Into::into);
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, IoError> {
    let file_id = req.uri().path().trim_start_matches("/").to_string();
    let filename = MAP
        .lock()
        .unwrap()
        .get(&file_id)
        .unwrap_or(&file_id)
        .to_string();
    resolve("./tmp", &req).await.map(|result| {
        let mut res = ResponseBuilder::new()
            .request(&req)
            .build(result)
            .expect("Failed to build response");
        res.headers_mut().append(
            hyper::header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment;filename={filename}")).unwrap(),
        );
        res
    })
}
