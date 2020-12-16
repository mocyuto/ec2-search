use crate::utils::{err_handler, print_table};
use rusoto_core::Region;
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum InstanceOpt {
    #[structopt(visible_alias = "ids", about = "search instance ids with query.")]
    InstanceIds(SearchQueryOpt),
    #[structopt(about = "search instance ips with query.")]
    Ips(SearchQueryOpt),
    #[structopt(visible_alias = "dns", about = "search instance DNS name with query.")]
    DNSName(SearchQueryOpt),
    #[structopt(about = "search instance basic info with query.")]
    Info(SearchQueryOpt),
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

pub async fn matcher(opt: InstanceOpt) {
    match opt {
        InstanceOpt::Info(opt) => info(opt).await,
        InstanceOpt::InstanceIds(opt) => instance_ids(opt).await,
        InstanceOpt::Ips(opt) => instance_ips(opt).await,
        InstanceOpt::DNSName(opt) => instance_private_dns(opt).await,
    }
}
async fn info(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances
        .into_iter()
        .map(|i| vec![i.id, i.name, i.status, i.instance_type])
        .collect();
    print_table(vec!["ID", "Name", "Status", "Type"], rows);
    println!("counts: {}", len);
}

async fn instance_ids(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances.into_iter().map(|i| vec![i.id, i.name]).collect();
    print_table(vec!["ID", "Name"], rows);
    println!("counts: {}", len);
}

async fn instance_ips(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances
        .into_iter()
        .map(|i| {
            vec![
                i.private_ip.unwrap_or_default(),
                i.public_ip.unwrap_or_default(),
                i.name,
            ]
        })
        .collect();
    print_table(vec!["Private IP", "Public IP", "Name"], rows);

    println!("counts: {}", len);
}
async fn instance_private_dns(opt: SearchQueryOpt) {
    let instances = get_instances(&opt).await;
    let len = instances.len();
    let rows: Vec<Vec<String>> = instances
        .into_iter()
        .map(|i| {
            vec![
                i.private_dns.unwrap_or_default(),
                i.public_dns.unwrap_or_default(),
                i.name,
            ]
        })
        .collect();
    print_table(vec!["Private DNS", "Public DNS", "Name"], rows);
    println!("counts: {}", len);
}

struct Instance {
    id: String,
    name: String,
    instance_type: String,
    status: String,
    private_ip: Option<String>,
    public_ip: Option<String>,
    private_dns: Option<String>,
    public_dns: Option<String>,
    tags: Vec<Tag>,
}
struct Tag {
    key: String,
    value: Option<String>,
}
async fn get_instances(opt: &SearchQueryOpt) -> Vec<Instance> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut m: Option<String> = None;
    let mut vector: Vec<Instance> = vec![];
    loop {
        let (mut v, mark) = instances(&ec2, &m).await;
        m = mark;
        vector.append(&mut v);
        if m.is_none() {
            break;
        }
    }
    vector.into_iter().filter(|i| search(i, &opt)).collect()
}
async fn instances(ec2: &Ec2Client, marker: &Option<String>) -> (Vec<Instance>, Option<String>) {
    let req = DescribeInstancesRequest {
        filters: None,
        instance_ids: None,
        dry_run: None,
        max_results: None,
        next_token: marker.clone(),
    };
    match ec2.describe_instances(req).await {
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
                        status: i.state.map(|i| i.name).flatten().unwrap_or_default(),
                        instance_type: i.instance_type.unwrap_or_default(),
                        private_ip: i.private_ip_address,
                        public_ip: i.public_ip_address,
                        private_dns: i.private_dns_name,
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
        Err(err) => panic!(err_handler(err)),
    }
}

fn search(i: &Instance, opt: &SearchQueryOpt) -> bool {
    for q in opt.query.split(',') {
        if i.name.contains(q)
            || i.id.contains(q)
            || i.private_dns.as_ref().filter(|d| d.contains(q)).is_some()
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
        private_ip: Some("192.168.0.1".to_string()),
        private_dns: Some("192.168.0.1.ap-northeast-1".to_string()),
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
fn name(i: &Option<Vec<rusoto_ec2::Tag>>) -> String {
    i.as_ref()
        .map(|v| {
            v.iter()
                .find(|t| t.key == Some("Name".to_string()))
                .map(|t| t.value.clone().unwrap_or_default())
        })
        .flatten()
        .unwrap_or_default()
}
#[test]
fn test_get_name_from_tag() {
    let t: Option<Vec<rusoto_ec2::Tag>> = Some(vec![rusoto_ec2::Tag {
        key: Some("Name".to_string()),
        value: Some("api".to_string()),
    }]);
    assert_eq!(name(&t), "api".to_string());
}
