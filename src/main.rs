extern crate rusoto_core;
extern crate rusoto_ec2;

use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use structopt::StructOpt;

use std::str;

#[derive(StructOpt)]
enum Opt {
    #[structopt(
        visible_alias = "ids",
        about = "seach ids with query. if set comma, search OR"
    )]
    InstanceIds(SearchByNameOpt),
    #[structopt(
        visible_alias = "ips",
        about = "seach private ips with query. if set comma, search OR"
    )]
    InstancePrivateIps(SearchByNameOpt),
}

#[derive(StructOpt)]
struct SearchByNameOpt {
    #[structopt(
        short = "q",
        long,
        conflicts_with("exact_query"),
        about = "search with asterisk"
    )]
    query: Option<String>,
    #[structopt(short, long = "exq", conflicts_with("query"), about = "search exactly")]
    exact_query: Option<String>,
}

#[tokio::main]
async fn main() {
    match Opt::from_args() {
        Opt::InstanceIds(opt) => instance_ids(opt).await,
        Opt::InstancePrivateIps(opt) => instance_private_ips(opt).await,
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

async fn instance_ids(opt: SearchByNameOpt) {
    let mut input = opt.query.map(|q| split(q, false)).unwrap_or(vec![]);
    let mut exact_input = opt.exact_query.map(|q| split(q, true)).unwrap_or(vec![]);
    input.append(&mut exact_input);
    let instances = get_instances(input).await;
    for id in &instances {
        println!("{} : {}", id.id, id.name);
    }
    println!("counts: {}", &instances.len());
}

async fn instance_private_ips(opt: SearchByNameOpt) {
    let mut input = opt.query.map(|q| split(q, false)).unwrap_or(vec![]);
    let mut exact_input = opt.exact_query.map(|q| split(q, true)).unwrap_or(vec![]);
    input.append(&mut exact_input);
    let instances = get_instances(input).await;
    for i in &instances {
        println!("{:?} : {}", i.private_ip, i.name);
    }
    println!("counts: {}", &instances.len());
}

struct Instance {
    id: String,
    name: String,
    private_ip: Vec<String>,
}
async fn get_instances(input: Vec<String>) -> Vec<Instance> {
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
                .flat_map(|res| res.iter())
                .flat_map(|r| r.instances.as_ref().unwrap());
            instances
                .map(|i| Instance {
                    id: i.instance_id.as_ref().unwrap().to_string(),
                    name: name(i),
                    private_ip: private_ip(i),
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

// extract Tag Name from instance
fn name(i: &rusoto_ec2::Instance) -> String {
    i.tags
        .as_ref()
        .unwrap()
        .iter()
        .find(|t| t.key == Some("Name".to_string()))
        .map(|t| t.value.as_ref().unwrap())
        .unwrap()
        .to_string()
}

// extract private IPs from instance
fn private_ip(i: &rusoto_ec2::Instance) -> Vec<String> {
    i.network_interfaces
        .as_ref()
        .unwrap()
        .iter()
        .map(|ni| ni.private_ip_address.as_ref().unwrap().to_string())
        .collect()
}
