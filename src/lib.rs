extern crate aws_sdk_autoscaling;
extern crate aws_sdk_ec2;
extern crate regex;
extern crate roxmltree;
extern crate rusoto_core;
extern crate rusoto_elbv2;

#[macro_use]
extern crate lazy_static;

pub mod autoscaling;
pub mod awsutils;
pub mod instance;
pub mod targetgroup;
pub mod utils;
