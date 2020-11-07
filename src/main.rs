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
    #[structopt(visible_alias = "i", about = "search instance")]
    Instance(instance::InstanceOpt),
    #[structopt(visible_alias = "tg", about = "search target group")]
    TargetGroup(targetgroup::TargetGroupOpt),
}

#[tokio::main]
async fn main() {
    match Cli::from_args().cmd {
        Command::Instance(opt) => instance::matcher(opt).await,
        Command::TargetGroup(opt) => targetgroup::matcher(opt).await,
    }
}
