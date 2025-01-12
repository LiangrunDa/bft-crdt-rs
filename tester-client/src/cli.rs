use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// ./tester-client --exp-name orset --num 1 --server1 "localhost:50051" --server2 "localhost:50052"
    #[clap(short, long)]
    pub exp_name: String,
    #[clap(short, long)]
    pub num: u8,
    #[clap(short, long)]
    pub depth: u8,
    #[clap(short, long, default_value = "0")]
    pub seed: u64,
    
    #[clap(long, default_value = "localhost:50051")]
    pub server1: String,
    
    #[clap(long, default_value = "localhost:50052")]
    pub server2: String,
    
}

pub fn parse_args() -> Args {
    Args::parse()
}