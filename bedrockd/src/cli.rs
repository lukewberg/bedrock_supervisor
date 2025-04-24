use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub daemon: bool,
    #[arg(short, long)]
    pub config: bool    
}