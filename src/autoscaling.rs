use crate::awsutils::{config, datetime_str};
use crate::utils::{get_values, print_table, split, Tag};
use aws_sdk_autoscaling::model::Instance;
use aws_sdk_autoscaling::Client;
use itertools::Itertools;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum AutoScalingGroupOpt {
    #[structopt(about = "display basic info")]
    Info(SearchInfoQueryOpt),
    #[structopt(visible_alias = "act", about = "display activities")]
    Activities(SearchQueryOpt),
    #[structopt(visible_alias = "inst", about = "display instances")]
    Instances(SearchQueryOpt),
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

#[derive(Debug, StructOpt)]
pub struct SearchInfoQueryOpt {
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

pub async fn matcher(opt: AutoScalingGroupOpt) {
    match opt {
        AutoScalingGroupOpt::Info(opt) => info(opt).await,
        AutoScalingGroupOpt::Activities(opt) => activities(opt).await,
        AutoScalingGroupOpt::Instances(opt) => instances(opt).await,
    }
}
async fn info(opt: SearchInfoQueryOpt) {
    let asg = get_autoscaling_groups(&SearchQueryOpt { query: opt.query }).await;
    let len = asg.len();

    let tag_column: Vec<String> = if opt.show_all_tags {
        asg.iter()
            .flat_map(|t| t.tags.iter().map(|ot| ot.key.to_string()))
            .unique()
            .collect()
    } else {
        opt.tag_columns
            .map(|t| split(&*t, true))
            .unwrap_or_default()
    };

    let rows: Vec<Vec<String>> = asg
        .into_iter()
        .map(|t| {
            let r = get_values(&t.tags, &tag_column);
            vec![
                t.name,
                t.instances.len().to_string(),
                t.desired_capacity
                    .map(|i| i.to_string())
                    .unwrap_or_default(),
                t.min_capacity.map(|i| i.to_string()).unwrap_or_default(),
                t.max_capacity.map(|i| i.to_string()).unwrap_or_default(),
            ]
            .into_iter()
            .chain(r)
            .collect()
        })
        .collect();
    let header: Vec<String> = vec![
        "Name".to_string(),
        "Instances".to_string(),
        "Desired".to_string(),
        "Min".to_string(),
        "Max".to_string(),
    ]
    .into_iter()
    .chain(tag_column)
    .collect();
    print_table(header, rows);
    println!("counts: {}", len);
}
async fn activities(opt: SearchQueryOpt) {
    let asg = get_autoscaling_groups(&opt).await;
    if asg.len() != 1 {
        println!("need to be narrowed to 1");
        return;
    }
    let a = get_activities(asg.first().unwrap().name.clone()).await;
    let len = a.len();
    let rows: Vec<Vec<String>> = a
        .into_iter()
        .map(|t| vec![t.status, t.description, t.start_at, t.end_at])
        .collect();
    print_table(
        vec![
            "Status".to_string(),
            "Desc".to_string(),
            "StartTime".to_string(),
            "EndTime".to_string(),
        ],
        rows,
    );
    println!("counts: {}", len);
}

async fn instances(opt: SearchQueryOpt) {
    let asg = get_autoscaling_groups(&opt).await;
    let rows: Vec<Vec<String>> = asg
        .into_iter()
        .flat_map(|a| {
            let name = a.name;
            a.instances
                .into_iter()
                .map(|i| {
                    vec![
                        name.clone(),
                        i.instance_id.unwrap_or_default(),
                        i.lifecycle_state
                            .map(|s| s.as_str().to_string())
                            .unwrap_or_default(),
                        i.instance_type.unwrap_or_default(),
                        i.availability_zone.unwrap_or_default(),
                        i.health_status.unwrap_or_default(),
                    ]
                })
                .collect::<Vec<_>>()
        })
        .collect();
    print_table(
        vec![
            "ASG Name".to_string(),
            "ID".to_string(),
            "LifeCycle".to_string(),
            "InstanceType".to_string(),
            "AZ".to_string(),
            "Status".to_string(),
        ],
        rows,
    )
}

struct AutoScalingGroup {
    name: String,
    instances: Vec<Instance>,
    min_capacity: Option<i32>,
    max_capacity: Option<i32>,
    desired_capacity: Option<i32>,
    tags: Vec<Tag>,
}

async fn get_autoscaling_groups(opt: &SearchQueryOpt) -> Vec<AutoScalingGroup> {
    let client = Client::new(&config().await);
    let mut m: Option<String> = None;
    let mut vector: Vec<AutoScalingGroup> = vec![];
    loop {
        let (mut v, mark) = autoscaling_groups(&client, &m).await;
        m = mark;
        vector.append(&mut v);
        if m.is_none() {
            break;
        }
    }
    vector
        .into_iter()
        .filter(|t| search_name(&opt.query, &t.name, &t.tags))
        .collect()
}

async fn autoscaling_groups(
    client: &Client,
    marker: &Option<String>,
) -> (Vec<AutoScalingGroup>, Option<String>) {
    let resp = client
        .describe_auto_scaling_groups()
        .set_next_token(marker.clone())
        .send();
    match resp.await {
        Ok(res) => {
            let groups = res.auto_scaling_groups.unwrap_or_default();
            (
                groups
                    .into_iter()
                    .map(|t| AutoScalingGroup {
                        name: t.auto_scaling_group_name.unwrap_or_default(),
                        instances: t.instances.unwrap_or_default(),
                        min_capacity: t.min_size,
                        max_capacity: t.max_size,
                        desired_capacity: t.desired_capacity,
                        tags: t
                            .tags
                            .map(|ot| {
                                ot.into_iter()
                                    .map(|t| Tag {
                                        key: t.key.unwrap_or_default(),
                                        value: t.value,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect(),
                res.next_token,
            )
        }
        Err(err) => panic!("{}", err.to_string()),
    }
}

fn search_name(query: &Option<String>, name: &str, tags: &[Tag]) -> bool {
    if query.is_none() || name.is_empty() {
        return true;
    }
    let tg: String = name.to_string();
    let qu: String = query.as_ref().unwrap().clone();
    for q in qu.split(',') {
        if tg.contains(q) {
            return true;
        }
        if tags
            .iter()
            .any(|t| t.key.contains(q) || t.value.as_ref().filter(|v| v.contains(q)).is_some())
        {
            return true;
        }
    }

    false
}
#[test]
fn test_search_name() {
    assert_eq!(
        search_name(&Some("api".to_string()), &"aa".to_string(), &vec![]),
        false
    );

    assert_eq!(
        search_name(
            &Some("api,test".to_string()),
            &"test-api".to_string(),
            &vec![]
        ),
        true
    );
    assert_eq!(
        search_name(&Some("api".to_string()), &"ap".to_string(), &vec![]),
        false
    );
    assert_eq!(
        search_name(
            &Some("test".to_string()),
            &"ap".to_string(),
            &vec![Tag {
                key: "test".to_string(),
                value: None
            }]
        ),
        true
    );
    assert_eq!(
        search_name(
            &Some("test".to_string()),
            &"ap".to_string(),
            &vec![Tag {
                key: "tag".to_string(),
                value: Some("test".to_string())
            }]
        ),
        true
    );
}

struct Activity {
    status: String,
    description: String,
    start_at: String,
    end_at: String,
}
async fn get_activities(asg_name: String) -> Vec<Activity> {
    let cli = Client::new(&config().await);
    match cli
        .describe_scaling_activities()
        .auto_scaling_group_name(asg_name)
        .send()
        .await
    {
        Ok(res) => res
            .activities
            .unwrap_or_default()
            .into_iter()
            .map(|a| Activity {
                status: a
                    .status_code
                    .map(|c| c.as_str().to_string())
                    .unwrap_or_default(),
                description: a.description.unwrap_or_default(),
                start_at: a.start_time.map(datetime_str).unwrap_or_default(),
                end_at: a.end_time.map(datetime_str).unwrap_or_default(),
            })
            .collect(),
        Err(err) => panic!("{}", err.to_string()),
    }
}
