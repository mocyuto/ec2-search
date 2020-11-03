extern crate roxmltree;
extern crate rusoto_core;
extern crate rusoto_ec2;

use roxmltree::Document;
use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use std::str;
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

#[derive(StructOpt)]
struct SearchQueryOpt {
    #[structopt(
        short = "q",
        long,
        conflicts_with("exact_query"),
        about = "ambiguous search with asterisk. tag name"
    )]
    query: Option<String>,
    #[structopt(
        short,
        long = "exq",
        conflicts_with("query"),
        about = "search by name exactly"
    )]
    exact_query: Option<String>,
    #[structopt(long, about = "query with instance ids. `i-` can be omitted")]
    ids: Option<String>,
}

#[tokio::main]
async fn main() {
    match Opt::from_args() {
        Opt::InstanceIds(opt) => instance_ids(opt).await,
        Opt::InstancePrivateIps(opt) => instance_private_ips(opt).await,
    }
}

async fn instance_ids(opt: SearchQueryOpt) {
    let instances = get_instances(name_query(&opt), id_query(&opt)).await;
    for id in &instances {
        println!("{} : {}", id.id, id.name);
    }
    println!("counts: {}", &instances.len());
}

async fn instance_private_ips(opt: SearchQueryOpt) {
    let instances = get_instances(name_query(&opt), id_query(&opt)).await;
    for i in &instances {
        println!("{:?} : {}", i.private_ip, i.name);
    }
    println!("counts: {}", &instances.len());
}

fn split(q: &str, is_exact: bool) -> Vec<String> {
    let format = |s: &str| {
        if is_exact {
            s.to_string()
        } else {
            format!("*{}*", s)
        }
    };
    q.split(',').map(|s| format(s)).collect()
}
fn name_query(opt: &SearchQueryOpt) -> Option<Vec<String>> {
    let input = opt.query.as_ref().map(|q| split(q, false));
    let exact_input = opt.exact_query.as_ref().map(|q| split(q, true));
    input
        .map(|i| exact_input.map(|e| [i, e].concat()))
        .flatten()
}
fn id_query(opt: &SearchQueryOpt) -> Option<Vec<String>> {
    fn add_i(s: &str) -> String {
        if !s.contains("i-") {
            "i-".to_string() + s
        } else {
            s.to_string()
        }
    }
    opt.ids
        .as_ref()
        .map(|q| q.split(',').map(|s| add_i(s)).collect())
}

struct Instance {
    id: String,
    name: String,
    private_ip: Vec<String>,
}
async fn get_instances(input: Option<Vec<String>>, ids: Option<Vec<String>>) -> Vec<Instance> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut req = DescribeInstancesRequest::default();
    req.filters = input.map(|i| {
        vec![rusoto_ec2::Filter {
            name: Some("tag:Name".to_string()),
            values: Some(i),
        }]
    });
    req.instance_ids = ids;
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
                let doc = Document::parse(&e.body_as_str()).unwrap();
                let finder = |s: &str| {
                    doc.descendants()
                        .find(|n| n.has_tag_name(s))
                        .map(|n| n.text())
                        .flatten()
                        .unwrap_or("unknown")
                };
                panic!(
                    "[ERROR] code:{}, message: {}",
                    finder("Code"),
                    finder("Message")
                );
            }
            _ => {
                panic!("[ERROR] Should have a typed error from EC2");
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
