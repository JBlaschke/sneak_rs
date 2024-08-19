#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use env_logger;

use clap::{Parser, Subcommand};


#[derive(Subcommand, Clone, Debug)]
enum Mode {
    Server {
        #[arg(long)]
        host: String,
        #[arg(short, long)]
        port: u16
    },
    Client {
        #[arg(short, long)]
        bind: String,
        #[arg(short, long)]
        port: u16
    }
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = "Sneak past firewalls by making all traffic look like HTTP(s)",
    arg_required_else_help = true
)]
struct Args {
    #[command(subcommand)]
    pub mode: Mode,

    #[clap(long)]
    pub tunnel_address: String,

    #[clap(long)]
    pub tunnel_port: u16
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}


fn main() {
    let args = Args::parse_args();
    env_logger::init();

    info!("Started sneak with inputs: {:?}", args);
}
