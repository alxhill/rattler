use axum::routing::get_service;
use std::net::SocketAddr;
use std::path::Path;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
use url::Url;

pub struct SimpleChannelServer {
    local_addr: SocketAddr,
    shutdown_sender: Option<oneshot::Sender<()>>,
}

impl SimpleChannelServer {
    /// Returns the root `Url` to the server.
    pub fn url(&self) -> Url {
        Url::parse(&format!("http://localhost:{}", self.local_addr.port())).unwrap()
    }
}

impl SimpleChannelServer {
    pub fn new(path: impl AsRef<Path>) -> Self {
        // Define a service to serve the contents of the folder. The `precompressed_gzip` method
        // adds the behavior that a file gzip compressed file called `<path>.gz` is preferred over
        // the original file. This is very useful because we can store gzipped compressed files in
        // the repository instead of the full-blown jsons.
        let service = get_service(ServeDir::new(path).precompressed_gzip());

        // Create a router that will serve the static files from the channel.
        let app = axum::Router::new().fallback_service(service);

        // Construct the server that will listen on localhost but with a *random port*. The random
        // port is very important because it enables creating multiple instances at the same time.
        // We need this to be able to run tests in parallel.
        let addr = SocketAddr::new([127, 0, 0, 1].into(), 0);
        let server = axum::Server::bind(&addr).serve(app.into_make_service());

        // Get the address of the server so we can bind to it at a later stage.
        let addr = server.local_addr();

        // Setup a graceful shutdown trigger which is fired when this instance is dropped.
        let (tx, rx) = oneshot::channel();
        let server = server.with_graceful_shutdown(async {
            rx.await.ok();
        });

        // Spawn the server. Let go of the JoinHandle, we can use the graceful shutdown trigger to
        // stop the server.
        let future = tokio::spawn(server);
        std::mem::drop(future);

        Self {
            local_addr: addr,
            shutdown_sender: Some(tx),
        }
    }
}

impl Drop for SimpleChannelServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_sender.take() {
            let _ = tx.send(());
        }
    }
}
