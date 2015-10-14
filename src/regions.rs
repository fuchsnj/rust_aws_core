#[derive(Debug, Clone, Copy)]
pub enum Region {
    UsEast1,
    UsWest1,
    UsWest2,
    EuWest1,
    EuCentral1,
    ApSoutheast1,
    ApNortheast1,
    ApSoutheast2,
    SaEast1,
}

pub fn region_in_aws_format(region: Region) -> String {
    match region {
        Region::UsEast1 => "us-east-1".to_string(),
        Region::UsWest1 => "us-west-1".to_string(),
        Region::UsWest2 => "us-west-2".to_string(),
        Region::EuWest1 => "eu-west-1".to_string(),
        Region::EuCentral1 => "eu-central-1".to_string(),
        Region::ApSoutheast1 => "ap-southeast-1".to_string(),
        Region::ApNortheast1 => "ap-northeast-1".to_string(),
        Region::ApSoutheast2 => "ap-southeast-2".to_string(),
        Region::SaEast1 => "sa-east-1".to_string(),
    }
}