use aws_config::meta::region::RegionProviderChain;
use aws_smithy_types::date_time::Format;
use aws_smithy_types::DateTime;
use aws_types::config::Config;
use aws_types::region::Region;

pub fn datetime_str(dt: DateTime) -> String {
    match dt.fmt(Format::HttpDate) {
        Ok(r) => r,
        Err(err) => panic!("{}", err.to_string()),
    }
}

pub struct GlobalOpt {
    pub region: Option<String>,
}

pub async fn config(opt: GlobalOpt) -> Config {
    let region_provider = RegionProviderChain::first_try(opt.region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    aws_config::from_env().region(region_provider).load().await
}
