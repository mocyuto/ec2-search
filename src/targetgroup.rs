use crate::utils::print_table;
use rusoto_core::Region;
use rusoto_elbv2::{DescribeTargetGroupsInput, DescribeTargetHealthInput, Elb, ElbClient};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TargetGroupOpt {
    #[structopt(
        visible_alias = "lb",
        about = "display load balancer arn with query. if set comma, search OR"
    )]
    LoadBalancerArn(SearchQueryOpt),
    #[structopt(about = "display port with query. if set comma, search OR")]
    Port(SearchQueryOpt),

    #[structopt(about = "get target healths")]
    Health(SearchQueryOpt),
}

#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
    #[structopt(short = "q", long, about = "ambiguous search with asterisk. tag name")]
    query: Option<String>,
}

pub async fn matcher(opt: TargetGroupOpt) {
    match opt {
        TargetGroupOpt::LoadBalancerArn(opt) => load_balancer_arn(opt).await,
        TargetGroupOpt::Port(opt) => ip_host(opt).await,
        TargetGroupOpt::Health(opt) => target_health(opt).await,
    }
}

async fn load_balancer_arn(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    let rows: Vec<Vec<String>> = tgs
        .iter()
        .map(|t| {
            vec![
                t.name.clone(),
                t.lb_arn
                    .as_ref()
                    .map(|l| format!("{:?}", l))
                    .unwrap_or_default(),
            ]
        })
        .collect();
    print_table(vec!["Name", "LB"], rows);

    println!("counts: {}", &tgs.len());
}

async fn ip_host(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    let rows: Vec<Vec<String>> = tgs
        .iter()
        .map(|t| vec![t.name.clone(), format!("{}", t.port)])
        .collect();
    print_table(vec!["Name", "Port"], rows);

    println!("counts: {}", &tgs.len());
}

async fn target_health(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    if tgs.len() != 1 {
        println!("need to be narrowed to 1");
        return;
    }
    let h = get_target_health(tgs.first().unwrap().arn.clone()).await;

    let rows: Vec<Vec<String>> = h
        .iter()
        .map(|t| vec![t.id.clone(), t.port.clone(), t.status.clone()])
        .collect();
    print_table(vec!["ID", "Port", "Status"], rows);

    println!("counts: {}", &h.len());
}

struct TargetGroup {
    name: String,
    port: i64,
    arn: String,
    lb_arn: Option<Vec<String>>,
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
            let tgs = res.target_groups.unwrap_or_default();
            tgs.into_iter()
                .filter(|t| search_name(&opt.query, &t.target_group_name))
                .map(|t| TargetGroup {
                    name: t.target_group_name.unwrap_or_default(),
                    port: t.port.unwrap_or_default(),
                    arn: t.target_group_arn.unwrap_or_default(),
                    lb_arn: t.load_balancer_arns,
                })
                .collect()
        }
        Err(err) => panic!(err.to_string()),
    }
}

fn search_name(query: &Option<String>, tg_name: &Option<String>) -> bool {
    if query.is_none() || tg_name.is_none() {
        return true;
    }
    let tg: String = tg_name.as_ref().unwrap().clone();
    let qu: String = query.as_ref().unwrap().clone();
    for q in qu.split(',') {
        if tg.contains(q) {
            return true;
        }
    }
    false
}
#[test]
fn test_search_name() {
    assert_eq!(
        search_name(&Some("api".to_string()), &Some("aa".to_string())),
        false
    );
    assert_eq!(
        search_name(&Some("api,test".to_string()), &Some("test-api".to_string())),
        true
    );
    assert_eq!(
        search_name(&Some("api".to_string()), &Some("ap".to_string())),
        false
    )
}
struct TargetHealth {
    id: String,
    port: String,
    status: String,
}
async fn get_target_health(arn: String) -> Vec<TargetHealth> {
    let elb = ElbClient::new(Region::ApNortheast1);

    match elb
        .describe_target_health(DescribeTargetHealthInput {
            target_group_arn: arn,
            targets: None,
        })
        .await
    {
        Ok(res) => res
            .target_health_descriptions
            .unwrap_or_default()
            .iter()
            .map(|h| TargetHealth {
                id: h
                    .target
                    .as_ref()
                    .map(|t| t.id.to_string())
                    .unwrap_or_default(),
                port: h
                    .health_check_port
                    .as_ref()
                    .map(|p| p.to_string())
                    .unwrap_or_default(),
                status: h
                    .target_health
                    .as_ref()
                    .map(|t| t.state.as_ref().map(|s| s.to_string()))
                    .flatten()
                    .unwrap_or_default(),
            })
            .collect(),
        Err(err) => panic!(err.to_string()),
    }
}
