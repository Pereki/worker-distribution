use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Argument {
    /// Name of the person to greet
    #[arg(short, long, value_enum)]
    pub server_type: ServerType,

    /// Network port to use
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..))]
    pub port: u16,

    /// Distributor ip address
    #[arg(short, long)]
    pub distributor_ip: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ServerType {
    /// Register the Server as a Distributor, distributing tasks
    DISTRIBUTOR,
    /// Register the Server as a Worker, executing tasks
    WORKER,
}
