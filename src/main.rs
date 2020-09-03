extern crate rusoto_core;
extern crate rusoto_ec2;

use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use structopt::StructOpt;

use std::str;

#[derive(StructOpt)]
enum Opt {
    #[structopt(visible_alias = "ids")]
    InstanceIds(InstanceIdsOpt),
}

#[derive(StructOpt)]
struct InstanceIdsOpt {
    #[structopt(short = "q", long, about = "search with asterisk")]
    query: Option<String>,
    #[structopt(short, long = "exq", about = "search exactly")]
    exact_query: Option<String>,
}

#[tokio::main]
async fn main() {
    match Opt::from_args() {
        Opt::InstanceIds(opt) => instance_ids(opt).await,
    }
}

fn split(q: String, is_exact: bool) -> Vec<String> {
    let format = |s| {
        if is_exact {
            format!("{}", s)
        } else {
            format!("*{}*", s)
        }
    };
    q.split(",").map(|s| format(s.to_string())).collect()
}

async fn instance_ids(opt: InstanceIdsOpt) {
    if opt.query.is_none() && opt.exact_query.is_none() {
        panic!("need to set `query` or `exact_query`")
    }
    let mut input = opt.query.map(|q| split(q, false)).unwrap_or(vec![]);
    let mut exact_input = opt.exact_query.map(|q| split(q, true)).unwrap_or(vec![]);
    input.append(&mut exact_input);
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
