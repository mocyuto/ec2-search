extern crate roxmltree;
extern crate rusoto_core;
extern crate rusoto_ec2;

use roxmltree::Document;
use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};
use structopt::StructOpt;

#[derive(StructOpt)]
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

pub struct Instance {
    pub id: String,
    pub name: String,
    pub private_ip: Vec<String>,
}
pub async fn get_instances(opt: &SearchQueryOpt) -> Vec<Instance> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut req = DescribeInstancesRequest::default();
    req.filters = name_query(opt).map(|i| {
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
    if input.is_none() && exact_input.is_none() {
        None
    } else {
        Some([input.unwrap_or_default(), exact_input.unwrap_or_default()].concat())
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
fn test_name_query() {
    let opt = SearchQueryOpt {
        query: Some("test".to_string()),
        exact_query: None,
        ids: None,
    };
    assert_eq!(name_query(&opt), Some(vec!["*test*".to_string()]));
    let opt = SearchQueryOpt {
        query: Some("api,test".to_string()),
        exact_query: None,
        ids: None,
    };
    assert_eq!(
        name_query(&opt),
        Some(vec!["*api*".to_string(), "*test*".to_string()])
    );
    let opt = SearchQueryOpt {
        query: None,
        exact_query: Some("ipa".to_string()),
        ids: None,
    };
    assert_eq!(name_query(&opt), Some(vec!["ipa".to_string()]));
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

// extract private IPs from instance
fn private_ip(i: &rusoto_ec2::Instance) -> Vec<String> {
    i.network_interfaces
        .as_ref()
        .unwrap()
        .iter()
        .map(|ni| ni.private_ip_address.as_ref().unwrap().to_string())
        .collect()
}
