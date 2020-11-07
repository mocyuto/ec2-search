extern crate rusoto_core;
extern crate rusoto_ec2;

use crate::utils::{err_handler, name_query};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum InstanceOpt {
    #[structopt(
        visible_alias = "ids",
        about = "search instance ids with query. if set comma, search OR"
    )]
    InstanceIds(SearchQueryOpt),
    #[structopt(
        visible_alias = "prips",
        about = "search private ips with query. if set comma, search OR"
    )]
    PrivateIps(SearchQueryOpt),
    #[structopt(
        visible_alias = "prdns",
        about = "search private ips with query. if set comma, search OR"
    )]
    PrivateDNS(SearchQueryOpt),
}

#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
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

pub async fn matcher(opt: InstanceOpt) {
    match opt {
        InstanceOpt::InstanceIds(opt) => instance_ids(opt).await,
        InstanceOpt::PrivateIps(opt) => instance_private_ips(opt).await,
        InstanceOpt::PrivateDNS(opt) => instance_private_dns(opt).await,
    }
}

async fn instance_ids(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    for id in &instances {
        println!("{} : {}", id.id, id.name);
    }
    println!("counts: {}", &instances.len());
}

async fn instance_private_ips(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    for i in &instances {
        println!(
            "{:?} : {}",
            i.private_ip.as_ref().unwrap_or(&"".to_string()),
            i.name
        );
    }
    println!("counts: {}", &instances.len());
}
async fn instance_private_dns(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    for i in &instances {
        println!(
            "{:?} : {}",
            i.private_dns.as_ref().unwrap_or(&"".to_string()),
            i.name
        );
    }
    println!("counts: {}", &instances.len());
}

struct Instance {
    id: String,
    name: String,
    private_ip: Option<String>,
    private_dns: Option<String>,
}
async fn get_instances(opt: &SearchQueryOpt) -> Vec<Instance> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut req = DescribeInstancesRequest::default();
    req.filters = name_query(&opt.query, &opt.exact_query).map(|i| {
        vec![rusoto_ec2::Filter {
            name: Some("tag:Name".to_string()),
            values: Some(i),
        }]
    });
    req.instance_ids = id_query(opt);
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
                    private_ip: i.private_ip_address.as_ref().map(|s| s.to_string()),
                    private_dns: i.private_dns_name.as_ref().map(|s| s.to_string()),
                })
                .collect::<Vec<_>>()
        }
        Err(err) => panic!(err_handler(err)),
    }
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

#[test]
fn test_id_query() {
    let opt = SearchQueryOpt {
        query: None,
        exact_query: None,
        ids: Some("1234".to_string()),
    };
    assert_eq!(id_query(&opt), Some(vec!["i-1234".to_string()]));
    let opt = SearchQueryOpt {
        query: None,
        exact_query: None,
        ids: Some("i-1234".to_string()),
    };
    assert_eq!(id_query(&opt), Some(vec!["i-1234".to_string()]));
    let opt = SearchQueryOpt {
        query: None,
        exact_query: None,
        ids: Some("i-1234,3333".to_string()),
    };
    assert_eq!(
        id_query(&opt),
        Some(vec!["i-1234".to_string(), "i-3333".to_string()])
    );
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
