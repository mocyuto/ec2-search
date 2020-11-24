use ec2_search::instance;
use ec2_search::targetgroup;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(visible_alias = "i", about = "Search instance")]
    Instance(instance::InstanceOpt),
    #[structopt(visible_alias = "tg", about = "Search target group")]
    TargetGroup(targetgroup::TargetGroupOpt),
    #[structopt(about = "Prints version information")]
    Version,
}

#[tokio::main]
async fn main() {
    match Cli::from_args().cmd {
        Command::Instance(opt) => instance::matcher(opt).await,
        Command::TargetGroup(opt) => targetgroup::matcher(opt).await,
        Command::Version => version().await,
    }
}

async fn version() {
    println!("ec2-search {}", env!("CARGO_PKG_VERSION"))
}
