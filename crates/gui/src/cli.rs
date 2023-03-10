use clap::Parser;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Result guide host name
    #[arg(long)]
    pub host: Option<String>,
    /// Result guide host port
    #[arg(short, long)]
    pub port: Option<u16>,
}
