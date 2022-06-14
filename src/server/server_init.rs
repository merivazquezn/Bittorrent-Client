use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use std::net::TcpListener;
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct Server;

impl Server {
    pub fn start() -> JoinHandle<()> {
        std::thread::spawn(move || {
            let _ = Self::listen(LOCALHOST);
        })
    }

    fn listen(address: &str) -> Result<(), ServerError> {
        let listener: TcpListener = TcpListener::bind(address)?;
        let pool = ThreadPool::new(POOL_WORKERS)?;
        for stream in listener.incoming() {
            let stream = stream?;
            pool.execute(|| {
                ServerConnection::run(stream);
            });
        }

        Ok(())
    }
}
