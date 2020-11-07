mod instance;
mod targetgroup;

use instance::InstanceOpt;
use structopt::StructOpt;
use targetgroup::TargetGroupOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(visible_alias = "i", about = "search instance")]
    Instance(InstanceOpt),
    #[structopt(visible_alias = "tg", about = "search target group")]
    TargetGroup(TargetGroupOpt),
}

#[tokio::main]
async fn main() {
    match Cli::from_args().cmd {
        Command::Instance(opt) => instance::matcher(opt).await,
        Command::TargetGroup(opt) => targetgroup::matcher(opt).await,
    }
}
