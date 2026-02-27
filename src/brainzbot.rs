use poise::Context;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use std::error::Error;

pub struct Brainz {
    http: Client,
    conn: MultiplexedConnection,
}

pub type BrainzError = Box<dyn Error + Send + Sync>;
pub type BrainzContext<'a> = Context<'a, Brainz, BrainzError>;

impl Brainz {
    pub fn new(http: Client, conn: MultiplexedConnection) -> Self {
        Self { http, conn }
    }

    pub fn conn(&self) -> &MultiplexedConnection {
        &self.conn
    }
}
