extern crate rusoto_core;
extern crate rusoto_ec2;
extern crate rusoto_ssm;

use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use rusoto_ssm::{Ssm, SsmClient, StartSessionRequest};
use structopt::StructOpt;

use std::str;

#[derive(StructOpt)]
enum Opt {
    #[structopt(visible_alias = "ids")]
    InstanceIds(InstanceIdsOpt),
    #[structopt(visible_alias = "start")]
    StartSession(SsmOpt),
}

#[derive(StructOpt)]
struct InstanceIdsOpt {
    #[structopt(short = "q", long)]
    query: String,
}

#[derive(StructOpt)]
struct SsmOpt {
    #[structopt(short = "i", long)]
    instance_id: String,
}

#[tokio::main]
async fn main() {
    match Opt::from_args() {
        Opt::InstanceIds(opt) => instance_ids(opt).await,
        Opt::StartSession(opt) => start_session(opt),
    }
}

async fn instance_ids(opt: InstanceIdsOpt) {
    let input = opt
        .query
        .split(",")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    get_instance_ids(input)
        .await
        .iter()
        .for_each(|i| println!("{} : {}", i.id, i.name));
}

struct Instance {
    id: String,
    name: String,
}
async fn get_instance_ids(input: Vec<String>) -> Vec<Instance> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut req = DescribeInstancesRequest::default();
    req.filters = Some(vec![rusoto_ec2::Filter {
        name: Some("tag:Name".to_string()),
        values: Some(input),
    }]);
    match ec2.describe_instances(req).await {
        Ok(res) => {
            let instances = res
                .reservations
                .iter()
                .flat_map(|res| res.iter().next())
                .flat_map(|r| r.instances.as_ref().unwrap());

            println!("{:?}", instances);
            instances
                .map(|i| Instance {
                    id: i.instance_id.as_ref().unwrap().to_string(),
                    name: i
                        .tags
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|t| t.key == Some("Name".to_string()))
                        .map(|t| t.value.as_ref().unwrap())
                        .unwrap()
                        .to_string(),
                })
                .collect::<Vec<_>>()
        }
        Err(error) => match error {
            RusotoError::Unknown(ref e) => {
                panic!("{}", str::from_utf8(&e.body).unwrap());
            }
            _ => {
                panic!("Should have a typed error from EC2");
            }
        },
    }
}

fn start_session(opt: SsmOpt) {
    let client = SsmClient::new(Region::ApNortheast1);
    let mut req = StartSessionRequest::default();
    req.target = opt.instance_id;
    let _res = client.start_session(req);
    println!("unimplemented")
}
