mod instance;

use instance::InstanceOpt;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(visible_alias = "i", about = "search instance")]
    Instance(InstanceOpt),
}

#[tokio::main]
async fn main() {
    match Cli::from_args().cmd {
        Command::Instance(opt) => instance::matcher(opt).await,
    }
}
