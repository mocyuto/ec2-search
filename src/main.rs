extern crate rusoto_core;
extern crate rusoto_ec2;

use rusoto_core::{Region, RusotoError};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client};

use std::str;

#[tokio::main]
async fn main() {
    let input: Vec<String> = std::env::args().skip(1).collect();
    let a = get_instance_ids(input);
    println!("{:?}", a.await);
}

async fn get_instance_ids(input: Vec<String>) -> Vec<String> {
    let ec2 = Ec2Client::new(Region::ApNortheast1);
    let mut req = DescribeInstancesRequest::default();
    req.filters = Some(vec![rusoto_ec2::Filter {
        name: Some("tag:Name".to_string()),
        values: Some(input),
    }]);
    match ec2.describe_instances(req).await {
        Ok(res) => {
            let instances = res
                .reservations
                .iter()
                .flat_map(|res| res.iter().next())
                .flat_map(|r| r.instances.as_ref().unwrap());

            instances
                .map(|i| i.instance_id.as_ref().unwrap().to_string())
                .collect::<Vec<_>>()
        }
        Err(error) => match error {
            RusotoError::Unknown(ref e) => {
                panic!("{}", str::from_utf8(&e.body).unwrap());
            }
            _ => {
                panic!("Should have a typed error from EC2");
            }
        },
    }
}
