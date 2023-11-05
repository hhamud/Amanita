use amanita_client::run_sender;
use amanita_server::run_reciever;
use clap::Parser;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cmd {
    //reciever
    #[arg(short, long, help = "Directory to output to")]
    output_dir: Option<PathBuf>,

    //reciever
    #[arg(short, long, help = "opened port")]
    port: Option<String>,

    //sender
    #[arg(short, long, help = "Directory to Copy from")]
    from: Option<PathBuf>,

    //sender
    #[arg(
        short,
        long,
        help = "Websocket url that must be in the format of 'ws://localhost:PORT/ws' "
    )]
    to: Option<Url>,
}

#[tokio::main]
async fn main() {
    let cmd = Cmd::parse();

    match (cmd.output_dir, cmd.port, cmd.from, cmd.to) {
        (Some(output_dir), Some(port), _, _) => run_reciever(port, output_dir).await,
        (_, _, Some(from), Some(to)) => run_sender(from, to).await,
        _  => println!("Invalid command or missing required arguments. Please use --help for usage information.")
    };
}
