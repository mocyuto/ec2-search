use ec2_search::autoscaling;
use ec2_search::instance;
use ec2_search::targetgroup;
use std::io;
use structopt::clap::Shell;
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
    #[structopt(visible_alias = "asg", about = "Search auto scaling group")]
    AutoScalingGroup(autoscaling::AutoScalingGroupOpt),
    #[structopt(about = "Prints version information")]
    Version,
    #[structopt(about = "Prints Completion")]
    Completion(CompletionOpt),
}
#[derive(Debug, StructOpt)]
enum CompletionOpt {
    Zsh,
    Bash,
    Fish,
}

#[tokio::main]
async fn main() {
    match Cli::from_args().cmd {
        Command::Instance(opt) => instance::matcher(opt).await,
        Command::TargetGroup(opt) => targetgroup::matcher(opt).await,
        Command::AutoScalingGroup(opt) => autoscaling::matcher(opt).await,
        Command::Version => version(),
        Command::Completion(opt) => match opt {
            CompletionOpt::Bash => completion(Shell::Bash),
            CompletionOpt::Zsh => completion(Shell::Zsh),
            CompletionOpt::Fish => completion(Shell::Fish),
        },
    }
}

fn version() {
    println!("ec2-search {}", env!("CARGO_PKG_VERSION"))
}

fn completion(s: Shell) {
    Cli::clap().gen_completions_to(env!("CARGO_BIN_NAME"), s, &mut io::stdout())
}
#[test]
fn test_completion() {
    completion(Shell::Bash);
    completion(Shell::Zsh);
    completion(Shell::Fish);
}
