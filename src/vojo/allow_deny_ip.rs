use ipnet::Ipv4Net;
use iprange::IpRange;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

use super::app_error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AllowDenyObject {
    pub limit_type: AllowType,
    pub value: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AllowType {
    #[default]
    AllowAll,
    DenyAll,
    Allow,
    Deny,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AllowResult {
    #[default]
    Allow,
    Deny,
    Notmapping,
}
impl AllowDenyObject {
    pub fn is_allow(&self, client_ip: String) -> Result<AllowResult, AppError> {
        if self.limit_type == AllowType::AllowAll {
            return Ok(AllowResult::Allow);
        }
        if self.limit_type == AllowType::DenyAll {
            return Ok(AllowResult::Deny);
        }
        if self.value.is_none() {
            return Err(AppError(String::from(
                "the value counld not be none when the limit_type is not AllowAll or DenyAll!",
            )));
        }
        let config_ip = self.value.clone().unwrap();
        let value_mapped_ip;
        if config_ip.contains('/') {
            let ip_range: IpRange<Ipv4Net> =
                [config_ip].iter().map(|s| s.parse().unwrap()).collect();
            let source_ip = client_ip.parse::<Ipv4Addr>().unwrap();
            value_mapped_ip = ip_range.contains(&source_ip);
        } else if self.value.clone().unwrap() == client_ip {
            value_mapped_ip = true;
        } else {
            value_mapped_ip = false;
        }
        if value_mapped_ip && self.limit_type == AllowType::Allow {
            return Ok(AllowResult::Allow);
        }
        if value_mapped_ip && self.limit_type == AllowType::Deny {
            return Ok(AllowResult::Deny);
        }

        Ok(AllowResult::Notmapping)
    }
}
