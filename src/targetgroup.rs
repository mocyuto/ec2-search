extern crate roxmltree;
extern crate rusoto_core;
extern crate rusoto_elbv2;

use roxmltree::Document;
use rusoto_core::{Region, RusotoError};
use rusoto_elbv2::{DescribeTargetGroupsInput, Elb, ElbClient};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TargetGroupOpt {
    #[structopt(
        visible_alias = "ip",
        about = "display ips and port with query. if set comma, search OR"
    )]
    TargetIpPort(SearchQueryOpt),
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
}

pub async fn matcher(opt: TargetGroupOpt) {
    match opt {
        TargetGroupOpt::TargetIpPort(opt) => ip_host(opt).await,
    }
}

async fn ip_host(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    for id in &tgs {
        println!("{} : {}", id.ip, id.port);
    }
    println!("counts: {}", &tgs.len());
}
struct TargetGroup {
    name: String,
    ip: String,
    port: i32,
}
async fn get_target_groups(opt: &SearchQueryOpt) -> Vec<TargetGroup> {
    let elb = ElbClient::new(Region::ApNortheast1);
    match elb
        .describe_target_groups(DescribeTargetGroupsInput {
            load_balancer_arn: None,
            marker: None,
            names: None,
            page_size: None,
            target_group_arns: None,
        })
        .await
    {
        Ok(res) => {
            println!("{:?}", res);
        }
        Err(err) => match err {
            RusotoError::Unknown(ref e) => {
                panic!("{}", err);
            }
            _ => {
                panic!("error");
            }
        },
    }
    vec![]
}
