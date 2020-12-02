use crate::utils::print_table;
use rusoto_autoscaling::{AutoScalingGroupNamesType, Autoscaling, AutoscalingClient, Instance};
use rusoto_core::Region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum AutoScalingGroupOpt {
    #[structopt(about = "display basic info")]
    Info(SearchQueryOpt),
}
#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
    #[structopt(
        short = "q",
        long,
        help = "ambiguous search with asterisk on target group name or ALB arn.  if set comma, search OR"
    )]
    query: Option<String>,
}

pub async fn matcher(opt: AutoScalingGroupOpt) {
    match opt {
        AutoScalingGroupOpt::Info(opt) => info(opt).await,
    }
}
async fn info(opt: SearchQueryOpt) {
    let asg = get_auto_scaling_groups(&opt).await;
    let len = asg.len();
    let rows: Vec<Vec<String>> = asg
        .into_iter()
        .map(|t| {
            vec![
                t.name,
                t.instances.len().to_string(),
                t.desired_capacity.to_string(),
                t.min_capacity.to_string(),
                t.max_capacity.to_string(),
            ]
        })
        .collect();
    print_table(vec!["Name", "Instances", "Desired", "Min", "Max"], rows);
    println!("counts: {}", len);
}

struct AutoScalingGroup {
    name: String,
    instances: Vec<Instance>,
    min_capacity: i64,
    max_capacity: i64,
    desired_capacity: i64,
}
async fn get_auto_scaling_groups(opt: &SearchQueryOpt) -> Vec<AutoScalingGroup> {
    let elb = AutoscalingClient::new(Region::ApNortheast1);
    let mut m: Option<String> = None;
    let mut vector: Vec<AutoScalingGroup> = vec![];
    loop {
        let (mut v, mark) = auto_scaling_group(&elb, &m).await;
        m = mark;
        vector.append(&mut v);
        if m.is_none() {
            break;
        }
    }
    vector
        .into_iter()
        .filter(|t| search_name(&opt.query, &t.name))
        .collect()
}

async fn auto_scaling_group(
    cli: &AutoscalingClient,
    marker: &Option<String>,
) -> (Vec<AutoScalingGroup>, Option<String>) {
    match cli
        .describe_auto_scaling_groups(AutoScalingGroupNamesType {
            auto_scaling_group_names: None,
            max_records: None,
            next_token: marker.clone(),
        })
        .await
    {
        Ok(res) => {
            let asg = res.auto_scaling_groups;
            (
                asg.into_iter()
                    .map(|t| AutoScalingGroup {
                        name: t.auto_scaling_group_name,
                        instances: t.instances.unwrap_or_default(),
                        min_capacity: t.min_size,
                        max_capacity: t.max_size,
                        desired_capacity: t.desired_capacity,
                    })
                    .collect(),
                res.next_token,
            )
        }
        Err(err) => panic!(err.to_string()),
    }
}

fn search_name(query: &Option<String>, name: &str) -> bool {
    if query.is_none() || name.is_empty() {
        return true;
    }
    let tg: String = name.to_string();
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
        search_name(&Some("api".to_string()), &"aa".to_string()),
        false
    );

    assert_eq!(
        search_name(&Some("api,test".to_string()), &"test-api".to_string(),),
        true
    );
    assert_eq!(
        search_name(&Some("api".to_string()), &"ap".to_string()),
        false
    );
}
