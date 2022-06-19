use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Config {
    /// Set a socket address to listen to
    #[clap(short = 's', long, value_parser, default_value = "127.0.0.1:7782")]
    pub address: SocketAddr,
    /// Path to the SQLite database
    #[clap(short, long, default_value = "spvc.db")]
    pub db_path: String,
    /// Checks the start of each URL against this list
    #[clap(short = 'u', long)] // TODO: Make this required.
    pub allowed_urls: Vec<String>,
    /// Enable saving the visitor's IP address
    #[clap(short = 'i', long)]
    pub save_ip: bool,
    /// Enable saving the visitor's user agent
    #[clap(short = 'a', long)]
    pub save_user_agent: bool,
}
