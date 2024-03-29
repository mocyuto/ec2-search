use crate::awsutils::{config, GlobalOpt};
use crate::utils::{get_values, print_table, split, Tag};
use aws_sdk_ec2::model::Tag as ec2_tag;
use aws_sdk_ec2::Client;
use itertools::Itertools;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum InstanceOpt {
    #[structopt(visible_alias = "ids", about = "search instance ids with query.")]
    InstanceIds(SearchQueryOpt),
    #[structopt(about = "search instance ips with query.")]
    Ips(SearchQueryOpt),
    #[structopt(visible_alias = "dns", about = "search instance DNS name with query.")]
    DnsName(SearchQueryOpt),
    #[structopt(about = "search instance basic info with query.")]
    Info(SearchInfoQueryOpt),
}

#[derive(Debug, StructOpt)]
pub struct SearchQueryOpt {
    #[structopt(
        short = "q",
        long,
        help = "ambiguous search with asterisk on tag name. if set comma, search OR"
    )]
    query: String,
}
#[derive(Debug, StructOpt)]
pub struct SearchInfoQueryOpt {
    #[structopt(
        short = "q",
        long,
        help = "ambiguous search with asterisk on tag name. if set comma, search OR"
    )]
    query: String,
    #[structopt(
        short = "o",
        help = "Output format. One of:
    name|wide"
    )]
    output: Option<String>,
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

pub async fn matcher(global_opt: GlobalOpt, opt: InstanceOpt) {
    let cli = Client::new(&config(global_opt).await);
    match opt {
        InstanceOpt::Info(opt) => info(&cli, opt).await,
        InstanceOpt::InstanceIds(opt) => instance_ids(&cli, opt).await,
        InstanceOpt::Ips(opt) => instance_ips(&cli, opt).await,
        InstanceOpt::DnsName(opt) => instance_private_dns(&cli, opt).await,
    }
}
async fn info(cli: &Client, opt: SearchInfoQueryOpt) {
    let instances = get_instances(cli, &SearchQueryOpt { query: opt.query }).await;
    let tag_column: Vec<String> = if opt.show_all_tags {
        instances
            .iter()
            .flat_map(|t| t.tags.iter().map(|ot| ot.key.to_string()))
            .unique()
            .collect()
    } else {
        opt.tag_columns
            .map(|t| split(&*t, true))
            .unwrap_or_default()
    };
    match opt.output.as_deref() {
        Some("name") => {
            let rows: Vec<Vec<String>> = instances.into_iter().map(|i| vec![i.name]).collect();
            print_table(vec![], rows);
        }
        Some("wide") => {
            let rows: Vec<Vec<String>> = instances
                .into_iter()
                .map(|i| {
                    let r = get_values(&i.tags, &tag_column);
                    vec![
                        i.id,
                        i.name,
                        i.status,
                        i.instance_type,
                        i.private_dns,
                        i.private_ip,
                        i.az,
                        i.lifecycle,
                    ]
                    .into_iter()
                    .chain(r)
                    .collect()
                })
                .collect();
            print_table(
                vec![
                    "ID".to_string(),
                    "Name".to_string(),
                    "Status".to_string(),
                    "Type".to_string(),
                    "PrivateDNS".to_string(),
                    "PrivateIP".to_string(),
                    "AZ".to_string(),
                    "LifeCycle".to_string(),
                ]
                .into_iter()
                .chain(tag_column)
                .collect(),
                rows,
            );
        }
        None => {
            let len = instances.len();
            let rows: Vec<Vec<String>> = instances
                .into_iter()
                .map(|i| {
                    let r = get_values(&i.tags, &tag_column);
                    vec![i.id, i.name, i.status, i.instance_type]
                        .into_iter()
                        .chain(r)
                        .collect()
                })
                .collect();
            print_table(
                vec![
                    "ID".to_string(),
                    "Name".to_string(),
                    "Status".to_string(),
                    "Type".to_string(),
                ]
                .into_iter()
                .chain(tag_column)
                .collect(),
                rows,
            );
            println!("counts: {}", len);
        }
        Some(a) => {
            eprintln!(
                "Error: unable to match a printer suitable for the output format '{}'. \
             allow formats are: name,wide",
                a
            );
            process::exit(1);
        }
    }
}

async fn instance_ids(cli: &Client, opt: SearchQueryOpt) {
    let instances = get_instances(cli, &opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances.into_iter().map(|i| vec![i.id, i.name]).collect();
    print_table(vec!["ID".to_string(), "Name".to_string()], rows);
    println!("counts: {}", len);
}

async fn instance_ips(cli: &Client, opt: SearchQueryOpt) {
    let instances = get_instances(cli, &opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances
        .into_iter()
        .map(|i| vec![i.private_ip, i.public_ip.unwrap_or_default(), i.id, i.name])
        .collect();
    print_table(
        vec![
            "Private IP".to_string(),
            "Public IP".to_string(),
            "ID".to_string(),
            "Name".to_string(),
        ],
        rows,
    );

    println!("counts: {}", len);
}
async fn instance_private_dns(cli: &Client, opt: SearchQueryOpt) {
    let instances = get_instances(cli, &opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances
        .into_iter()
        .map(|i| {
            vec![
                i.private_dns,
                i.public_dns.unwrap_or_default(),
                i.id,
                i.name,
            ]
        })
        .collect();
    print_table(
        vec![
            "Private DNS".to_string(),
            "Public DNS".to_string(),
            "ID".to_string(),
            "Name".to_string(),
        ],
        rows,
    );
    println!("counts: {}", len);
}

struct Instance {
    id: String,
    name: String,
    instance_type: String,
    status: String,
    az: String,
    lifecycle: String,
    private_ip: String,
    public_ip: Option<String>,
    private_dns: String,
    public_dns: Option<String>,
    tags: Vec<Tag>,
}

async fn get_instances(client: &Client, opt: &SearchQueryOpt) -> Vec<Instance> {
    let mut m: Option<String> = None;
    let mut vector: Vec<Instance> = vec![];
    loop {
        let (mut v, mark) = instances(client, &m).await;
        m = mark;
        vector.append(&mut v);
        if m.is_none() {
            break;
        }
    }
    vector.into_iter().filter(|i| search(i, opt)).collect()
}
async fn instances(cli: &Client, marker: &Option<String>) -> (Vec<Instance>, Option<String>) {
    match cli
        .describe_instances()
        .set_next_token(marker.clone())
        .send()
        .await
    {
        Ok(res) => {
            let instances = res
                .reservations
                .into_iter()
                .flat_map(|v| v.into_iter().flat_map(|r| r.instances.unwrap_or_default()));
            (
                instances
                    .map(|i| Instance {
                        name: name(&i.tags),
                        id: i.instance_id.unwrap_or_default(),
                        status: i
                            .state
                            .and_then(|i| i.name.map(|n| n.as_str().to_string()))
                            .unwrap_or_default(),
                        lifecycle: i
                            .instance_lifecycle
                            .map(|i| i.as_str().to_string())
                            .unwrap_or_else(|| "normal".to_string()),
                        az: i
                            .placement
                            .and_then(|p| p.availability_zone)
                            .unwrap_or_default(),
                        instance_type: i
                            .instance_type
                            .map(|i| i.as_str().to_string())
                            .unwrap_or_default(),
                        private_ip: i.private_ip_address.unwrap_or_default(),
                        public_ip: i.public_ip_address,
                        private_dns: i.private_dns_name.unwrap_or_default(),
                        public_dns: i.public_dns_name,
                        tags: i
                            .tags
                            .map(|vt| {
                                vt.into_iter()
                                    .map(|t| Tag {
                                        key: t.key.unwrap_or_default(),
                                        value: t.value,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                    })
                    .collect::<Vec<_>>(),
                res.next_token,
            )
        }
        Err(err) => panic!("{}", err.to_string()),
    }
}

fn search(i: &Instance, opt: &SearchQueryOpt) -> bool {
    for q in opt.query.split(',') {
        if i.name.contains(q)
            || i.id.contains(q)
            || i.private_dns.contains(q)
            || i.tags
                .iter()
                .any(|t| t.key.contains(q) || t.value.as_ref().filter(|v| v.contains(q)).is_some())
        {
            return true;
        }
    }
    false
}
#[test]
fn test_search() {
    let i = Instance {
        id: "i-2342545".to_string(),
        name: "api".to_string(),
        instance_type: "t3.micro".to_string(),
        status: "running".to_string(),
        az: "ap-northeast-1".to_string(),
        lifecycle: "normal".to_string(),
        private_ip: "192.168.0.1".to_string(),
        private_dns: "192.168.0.1.ap-northeast-1".to_string(),
        public_ip: None,
        public_dns: None,
        tags: vec![Tag {
            key: "env".to_string(),
            value: Some("production".to_string()),
        }],
    };
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "234254".to_string()
            }
        ),
        true
    );
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "api,test".to_string()
            }
        ),
        true
    );
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "test".to_string()
            }
        ),
        false
    );
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "192.168".to_string()
            }
        ),
        true
    );
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "server,test".to_string()
            }
        ),
        false
    );
    assert_eq!(
        search(
            &i,
            &SearchQueryOpt {
                query: "production".to_string()
            }
        ),
        true
    );
}

// extract Tag Name from instance
fn name(i: &Option<Vec<ec2_tag>>) -> String {
    i.as_ref()
        .and_then(|v| {
            v.iter()
                .find(|t| t.key == Some("Name".to_string()))
                .map(|t| t.value.clone().unwrap_or_default())
        })
        .unwrap_or_default()
}
#[test]
fn test_get_name_from_tag() {
    let tag = aws_sdk_ec2::model::tag::Builder::default()
        .set_key(Some("Name".to_string()))
        .set_value(Some("api".to_string()))
        .build();
    let t: Option<Vec<ec2_tag>> = Some(vec![tag]);
    assert_eq!(name(&t), "api".to_string());
}
