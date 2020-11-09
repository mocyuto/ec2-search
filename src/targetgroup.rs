use crate::utils::print_table;
use rusoto_core::Region;
use rusoto_elbv2::{DescribeTargetGroupsInput, Elb, ElbClient};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum TargetGroupOpt {
    #[structopt(about = "display port with query. if set comma, search OR")]
    Port(SearchQueryOpt),
}

#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
    #[structopt(short = "q", long, about = "ambiguous search with asterisk. tag name")]
    query: Option<String>,
}

pub async fn matcher(opt: TargetGroupOpt) {
    match opt {
        TargetGroupOpt::Port(opt) => ip_host(opt).await,
    }
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
struct TargetGroup {
    name: String,
    port: i64,
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
