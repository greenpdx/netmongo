use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use chrono::{DateTime, Utc, serde::ts_seconds};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct IPInfo {
    ip: String,
    country_name: String,
    #[serde(rename = "country_code2")]
    country_code: String,
    isp: String,
}

impl IPInfo {
    fn new() -> Self {
        Self {
            ip: "".to_string(),
            country_name: "".to_string(),
            country_code: "".to_string(),
            isp: "".to_string(),
        }
    }
}

#[derive(Debug, Clone,Deserialize, Serialize)]
pub struct Intruder {
    pub username: String,
    pub password: String,
    pub ip_info: IPInfo,
    pub ip_v4_address: Option<Ipv4Addr>,
    pub ip_v6_address: Option<Ipv6Addr>,
    pub ip: String,
    pub source_port: u16,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

impl Intruder {
    pub fn init(ip_address: IpAddr, source_port: u16) -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
            ip_info: IPInfo::new(),
            ip_v4_address: None,
            ip_v6_address: None,
            ip: "".to_string(),
            source_port: 0,
            time: Utc::now(),
        }
    }

    pub fn set_ip(&mut self) {
        if let Some(ip) = self.ip_v4_address {
            self.ip = ip.to_string();
        } else if let Some(ip) = self.ip_v6_address {
            self.ip = ip.to_string();
        } else {
            self.ip = "".to_string();
        }
    }

    fn time_to_text(&self) -> String {
        self.time.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}