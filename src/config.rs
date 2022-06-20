use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Config {
    /// Checks the start of each URL against this list
    #[clap(required(true))]
    pub allowed_urls: Vec<String>,
    /// Set a socket address to listen to
    #[clap(short = 'a', long, value_parser, default_value = "127.0.0.1:7782")]
    pub address: SocketAddr,
    /// Path to the SQLite database
    #[clap(short, long, default_value = "spvc.db")]
    pub db_path: String,
    /// Enable saving the visitor's IP address
    #[clap(short = 'i', long)]
    pub save_ip: bool,
    /// Enable saving the visitor's user agent
    #[clap(short = 'u', long)]
    pub save_user_agent: bool,
    /// Save visits with missing referer header instead of treating them as unauthorized calls
    #[clap(short = 'm', long)]
    pub save_missing_referer: bool,
}
