use crate::utils::{get_values,print_table,split, Tag};
use regex::Regex;
use rusoto_core::Region;
use rusoto_elbv2::{DescribeTargetGroupsInput, DescribeTargetHealthInput, Elb, ElbClient, DescribeTagsInput};
use structopt::StructOpt;
use itertools::Itertools;


#[derive(Debug, StructOpt)]
pub enum TargetGroupOpt {
    #[structopt(about = "display basic info")]
    Info(SearchQueryOpt),
    #[structopt(visible_alias = "lb", about = "display load balancer")]
    LoadBalancerArn(SearchQueryOpt),
    #[structopt(about = "display port")]
    Port(SearchQueryOpt),

    #[structopt(about = "get target healths")]
    Health(SearchQueryOpt),
}

#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
    #[structopt(
        short = "q",
        long,
        help = "ambiguous search with asterisk on target group name or ALB arn.  if set comma, search OR"
    )]
    query: Option<String>,
    #[structopt(
    short = "T",
    long,
    help = "Accepts a comma separated list of tags that are going to be presented as columns.
        Tags are case-sensitive."
    )]
    tag_columns: Option<String>,
    #[structopt(long = "show-all-tags", help = "Show all tags.")]
    show_all_tags: bool,
}

pub async fn matcher(opt: TargetGroupOpt) {
    match opt {
        TargetGroupOpt::Info(opt) => info(opt).await,
        TargetGroupOpt::LoadBalancerArn(opt) => load_balancer_arn(opt).await,
        TargetGroupOpt::Port(opt) => port(opt).await,
        TargetGroupOpt::Health(opt) => target_health(opt).await,
    }
}

async fn info(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    let len = tgs.len();

    let tag_column: Vec<String> = if opt.show_all_tags {
        tgs.iter()
            .map(|t| t.tags.iter().map(|ot| ot.key.to_string()))
            .flatten()
            .unique()
            .collect()
    } else {
        opt.tag_columns
            .map(|t| split(&*t, true))
            .unwrap_or_default()
    };

    let rows: Vec<Vec<String>> = tgs
        .into_iter()
        .map(|t| {
            let r = get_values(&t.tags, &tag_column);
            vec![
                t.name,
                t.target_type,
                t.lb.map(|l| format!("{:?}", l)).unwrap_or_default(),
            ]
                .into_iter()
                .chain(r)
                .collect()
        })
        .collect();
    let header: Vec<String> = vec![
        "Name".to_string(),
        "TargetType".to_string(),
        "LB".to_string(),
    ]
        .into_iter()
        .chain(tag_column)
        .collect();
    print_table(header,rows);
    println!("counts: {}", len);
}

async fn load_balancer_arn(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    let len = tgs.len();
    let rows: Vec<Vec<String>> = tgs
        .into_iter()
        .map(|t| {
            vec![
                t.name,
                t.lb_arn.map(|l| format!("{:?}", l)).unwrap_or_default(),
            ]
        })
        .collect();
    print_table(vec!["Name".to_string(), "LB arn".to_string()], rows);
    println!("counts: {}", len);
}

async fn port(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    let len = tgs.len();
    let rows: Vec<Vec<String>> = tgs
        .into_iter()
        .map(|t| vec![t.name, format!("{}", t.port)])
        .collect();
    print_table(vec!["Name".to_string(), "Port".to_string()], rows);
    println!("counts: {}", len);
}

async fn target_health(opt: SearchQueryOpt) {
    let tgs = get_target_groups(&opt).await;
    if tgs.len() != 1 {
        println!("need to be narrowed to 1");
        return;
    }
    let h = get_target_health(tgs.first().unwrap().arn.clone()).await;
    let len = h.len();
    let rows: Vec<Vec<String>> = h
        .into_iter()
        .map(|t| vec![t.id, t.port, t.status])
        .collect();
    print_table(
        vec!["ID".to_string(), "Port".to_string(), "Status".to_string()],
        rows,
    );
    println!("counts: {}", len);
}

struct TargetGroup {
    name: String,
    port: i64,
    arn: String,
    target_type: String,
    lb: Option<Vec<String>>,
    lb_arn: Option<Vec<String>>,
    tags: Vec<Tag>,
}
async fn get_target_groups(opt: &SearchQueryOpt) -> Vec<TargetGroup> {
    let elb = ElbClient::new(Region::ApNortheast1);
    let mut m: Option<String> = None;
    let mut vector: Vec<TargetGroup> = vec![];
    loop {
        let (mut v, mark) = target_group(&elb, &m).await;
        m = mark;
        vector.append(&mut v);
        if m.is_none() {
            break;
        }
    }
    let tgs = vector
        .into_iter()
        .filter(|t| search_name(&opt.query, &t.name, &t.lb_arn))
        .collect();
    set_tags(&elb, tgs).await
}

async fn target_group(
    elb: &ElbClient,
    marker: &Option<String>,
) -> (Vec<TargetGroup>, Option<String>) {
    match elb
        .describe_target_groups(DescribeTargetGroupsInput {
            load_balancer_arn: None,
            marker: marker.clone(),
            names: None,
            page_size: None,
            target_group_arns: None,
        })
        .await
    {
        Ok(res) => {
            let tgs = res.target_groups.unwrap_or_default();
            (
                tgs.into_iter()
                    .map(|t| TargetGroup {
                        name: t.target_group_name.unwrap_or_default(),
                        port: t.port.unwrap_or_default(),
                        arn: t.target_group_arn.unwrap_or_default(),
                        target_type: t.target_type.unwrap_or_default(),
                        lb: t
                            .load_balancer_arns
                            .as_ref()
                            .map(|v| v.iter().map(|arn| extract_lb_name(arn)).collect()),
                        lb_arn: t.load_balancer_arns,
                        tags: vec![],
                    })
                    .collect(),
                res.next_marker,
            )
        }
        Err(err) => panic!("{}", err.to_string()),
    }
}

fn search_name(query: &Option<String>, tg_name: &str, lb_arn: &Option<Vec<String>>) -> bool {
    if query.is_none() || tg_name.is_empty() {
        return true;
    }
    let tg: String = tg_name.to_string();
    let qu: String = query.as_ref().unwrap().clone();
    let lb: String = lb_arn.as_ref().map(|lv| lv.join(",")).unwrap_or_default();
    for q in qu.split(',') {
        if tg.contains(q) || lb.contains(q) {
            return true;
        }
    }

    false
}

async fn set_tags(elb: &ElbClient, tgs: Vec<TargetGroup>) -> Vec<TargetGroup>{
    let mut arns = tgs.iter()
        .map(|t| t.arn.clone());
    let mut offset = 0;
    let mut vector: Vec<TargetGroup> = vec![];
    loop {
        if offset > arns.len() {
            break
        }
        offset += 20;
        let input = DescribeTagsInput{
            resource_arns: arns.by_ref().take(offset).collect(),
        };
        match elb.describe_tags(input).await{
            Ok(res) => {
                res.tag_descriptions.into_iter()
                    .for_each(|vt|{
                        vt.into_iter()
                            .for_each(|td| {
                                let target_op = tgs.iter().find(|tg| td.resource_arn == Some(tg.arn.clone()));
                                if target_op.is_some() {
                                    let tags = td.tags.map(|ot| {
                                        ot.into_iter()
                                            .map(|t| Tag {
                                                key: t.key,
                                                value: t.value,
                                            })
                                            .collect()
                                    }).unwrap_or_default();
                                    let t = target_op.map(|t|{
                                    TargetGroup {
                                        name: t.name.clone(),
                                        port: t.port.clone(),
                                        arn: t.arn.clone(),
                                        target_type: t.target_type.clone(),
                                        lb: t.lb.clone(),
                                        lb_arn: t.lb_arn.clone(),
                                        tags,
                                    }}).unwrap();
                                    vector.push(t)
                                }
                            })
                    })
            },
            Err(err) => panic!("{}",err.to_string()),
        }
    }
    vector
}
#[test]
fn test_search_name() {
    assert_eq!(
        search_name(&Some("api".to_string()), &"aa".to_string(), &None),
        false
    );
    assert_eq!(
        search_name(
            &Some("aa".to_string()),
            &"".to_string(),
            &Some(vec!["aa".to_string()])
        ),
        true
    );

    assert_eq!(
        search_name(
            &Some("api,test".to_string()),
            &"test-api".to_string(),
            &None
        ),
        true
    );
    assert_eq!(
        search_name(&Some("api".to_string()), &"ap".to_string(), &None),
        false
    );
    assert_eq!(
        search_name(
            &Some("api".to_string()),
            &"ap".to_string(),
            &Some(vec!["api-lb".to_string()])
        ),
        true
    );
}

fn extract_lb_name(lb_arn: &str) -> String {
    lazy_static! {
        static ref ALB: Regex = Regex::new(r"^.+loadbalancer/app/(.+)/.+$").unwrap();
        static ref NLB: Regex = Regex::new(r"^.+loadbalancer/net/(.+)/.+$").unwrap();
    }

    if lb_arn.contains(&"loadbalancer/app/") {
        let c = ALB.captures(lb_arn).unwrap();
        return c[1].to_string();
    } else if lb_arn.contains(&"loadbalancer/net/") {
        let c = NLB.captures(lb_arn).unwrap();
        return c[1].to_string();
    }
    "".to_string()
}
#[test]
fn test_extract_lb_name() {
    assert_eq!(
        extract_lb_name(&"arn:aws:elasticloadbalancing:ap-northeast-1:11111111:loadbalancer/app/api-alb/abcdefg123".to_string()),
        "api-alb".to_string());
    assert_eq!(
        extract_lb_name(&"arn:aws:elasticloadbalancing:ap-northeast-1:11111111:loadbalancer/net/api-alb/abcdefg123".to_string()),
        "api-alb".to_string());
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
        Err(err) => panic!("{}", err.to_string()),
    }
}
