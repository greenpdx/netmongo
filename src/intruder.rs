use serde::{Deserialize, Serialize};
use log::{info, warn, error};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use chrono::{DateTime, Utc, serde::ts_seconds};
use crate::{ipcache::IpTok,AppData,Config, open_intruders_collection};

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
    pub iptok: IpTok,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

impl Intruder {
    pub fn init(ip_address: IpAddr, dst_port: u16) -> Self {
        Self {
            iptok: IpTok { saddr: ip_address, dport: dst_port},
            username: None,
            password: None,
            time: Utc::now(),
        }
    }

    fn time_to_text(&self) -> String {
        self.time.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub async fn wrdb_intruder(&self, ap:&AppData) {
        let client = &ap.mongo;
    
        let coll = open_intruders_collection(&client).await;
        let rslt = match coll.insert_one(self, None).await {
            Ok(d) => {
                d
            },
            Err(err) => {
                    println!("{:?}",err);
                    return
            },
         };

         println!("Intruder {:?}", rslt);
    }
}