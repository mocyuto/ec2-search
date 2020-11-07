mod instance;

use instance::SearchQueryOpt;
use structopt::StructOpt;

#[derive(StructOpt)]
enum Opt {
    #[structopt(
        visible_alias = "ids",
        about = "search ids with query. if set comma, search OR"
    )]
    InstanceIds(SearchQueryOpt),
    #[structopt(
        visible_alias = "ips",
        about = "search private ips with query. if set comma, search OR"
    )]
    InstancePrivateIps(SearchQueryOpt),
}

#[tokio::main]
async fn main() {
    match Opt::from_args() {
        Opt::InstanceIds(opt) => instance_ids(opt).await,
        Opt::InstancePrivateIps(opt) => instance_private_ips(opt).await,
    }
}

async fn instance_ids(opt: SearchQueryOpt) {
    let instances = instance::get_instances(&opt).await;
    for id in &instances {
        println!("{} : {}", id.id, id.name);
    }
    println!("counts: {}", &instances.len());
}

async fn instance_private_ips(opt: SearchQueryOpt) {
    let instances = instance::get_instances(&opt).await;
    for i in &instances {
        println!("{:?} : {}", i.private_ip, i.name);
    }
    println!("counts: {}", &instances.len());
}
