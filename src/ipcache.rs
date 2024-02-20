use std::collections::HashMap;
use std::hash::Hash;
use chrono::{DateTime, Utc, serde::ts_seconds};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
//se crate::attack::new_geoip;
use crate::{AppData,  Result, ipinfo_coll};
use mongodb::Client;
use std::default::Default;
use ipgeolocate::Service;
use serde_json::Value;
use reqwest as rq;
//use std::sync::Arc;
use anyhow::anyhow;

#[derive(Debug, Clone,Deserialize, Serialize, Eq, PartialEq, Hash)]
pub struct IpTok {
    pub saddr: IpAddr,
    pub dport: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfIpInfo {
    geosrv: Service,

}
impl Default for ConfIpInfo {
    fn default() -> Self {
        ConfIpInfo {
            geosrv: Service::IpApiCo,
        }
    }
}
#[derive(Debug, Clone,Deserialize, Serialize)]
pub struct IpInfo {
    iptok: IpTok,
    ipid: IpId,
    geoinfo: Option<GeoInfo>,
    #[serde(with = "ts_seconds")]
    first: DateTime<Utc>,
}
impl IpInfo {
    async fn _init(iptok: &IpTok, ap: &AppData) -> Self {
        let ginfo = match new_geoip(iptok, &ap._conf.ipinfo).await {
            Ok(ginfo) => { 
                let val: GeoInfoIpApiCo = serde_json::from_value(ginfo.clone()).expect("No1");
                Some(GeoInfo::IpApiCo(val))
            },
            Err(err) => {
                println!("{:?}",err);
                None
            }
        };
        let now = Utc::now();
        IpInfo {
            iptok: iptok.clone(),
            ipid: IpId::default(),
            geoinfo: ginfo,
            first: now,
        }
    }
}

#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
struct IpId {

}
impl Default for IpId {
    fn default () -> Self {
        IpId {}
    }
}

#[derive(Debug, Clone)]
pub struct IpInfoCache{
    mongo: Client,
    conf: ConfIpInfo,
    cache: HashMap<IpTok,IpInfo>,
}
impl IpInfoCache {
    pub fn init(client: &Client, conf: &ConfIpInfo) -> Self {
        IpInfoCache { mongo: client.clone(), conf: conf.clone(), cache: HashMap::new()}
    }
    pub async fn retrieve(&self, iptok: &IpTok) -> Option<IpInfo> {
        let iptok = IpTok { saddr: IpAddr::from ([109,205,213,221]), dport: 23};
;

        match self.retrieve_from_memory(&iptok).await {
            Some(ipi) => { return Some(ipi) },
            None => {
                match self.retrieve_from_database(&iptok).await {
                    Some(ipi) => { return Some(ipi)},
                    None => {
                        let _ginfo = new_geoip(&iptok, &self.conf).await;

                        return None
                    }
                }
            }
        }
    }
    async fn retrieve_from_memory(&self, iptok: &IpTok) -> Option<IpInfo> {
        let rslt = self.cache.contains_key(iptok);
        let ipinfo: Option<IpInfo> = if rslt {
            Some(self.cache.get(iptok).unwrap().clone())
        } else {
            None
        };
        ipinfo

    }
    async fn retrieve_from_database(&self, iptok: &IpTok) -> Option<IpInfo> {
        let client = &self.mongo;
        let coll = ipinfo_coll(&client, &iptok).await;
    
        let ipdoc = bson::to_document(iptok).unwrap();
        let cursor = coll.find(ipdoc, None).await;
        let mut cursor = cursor.unwrap();

        if cursor.advance().await.unwrap() {
            let ipinfo = cursor.deserialize_current();
            let ipinfo = ipinfo.unwrap();
            println!("{:?}", ipinfo);
            return Some(ipinfo)
        } else {
            return None
        }
    }
    async fn add(&mut self, iptok: &IpTok) {
        let ginfo = new_geoip(iptok, &self.conf).await;
        let ginfo = ginfo.unwrap();
        let val: GeoInfoIpApiCo = serde_json::from_value(ginfo.clone()).expect("No");
                
        let ipid = IpId::default();
        let now = Utc::now();
        let ipinfo = IpInfo {
            iptok: iptok.clone(),
            ipid: ipid,
            geoinfo: Some(GeoInfo::IpApiCo(val)),
            first: now,
        };
        
        let coll = ipinfo_coll(&self.mongo, iptok).await;
        let rslt = match coll.insert_one(&ipinfo, None).await {
            Ok(id) => { 
                println!("{:?}",id);
                id
            },
            Err(err) => {
                println!("{:?}", err);
                return;
            }
        };
        self.cache.insert(iptok.clone(), ipinfo);

    }
}

#[derive(Debug, Clone,Deserialize, Serialize)]
enum GeoInfo {
    IpApiCo(GeoInfoIpApiCo),
}


#[derive(Debug, Clone,Deserialize, Serialize)]
pub struct GeoInfoIpApiCo {
    status: String,
    pub country: String,
    #[serde(alias = "countryCode")]
    pub country_code: String,
    pub region: String,
    #[serde(alias = "regionName")]
    pub region_name: String,
    pub city: String,
    pub zip: String,
    pub lat: f32,
    pub lon: f32,
    pub timezone: String,
    isp: String,
    org: String,
    #[serde(alias = "as")]
    gas: String,
    query: String,
}
/*
impl From<&Locator> for GeoInfo {
    fn from(loc: &Locator) -> Self {
        GeoInfo {
            ip: loc.ip.clone(),
            //latitude: Some(loc.latitude.clone()),
            //longitude: Some(loc.longitude.clone()),
            city: loc.city.clone(),
            region: loc.region.clone(),
            country: loc.country.clone(),
            timezone: loc.timezone.clone(),
            //location: loc.location.clone(),
        }
    }
}
*/
async fn get_ggip_web(iptok: &IpTok, _serv: &Service) -> anyhow::Result<Value> {
    let ip = iptok.saddr.to_string();

    let url = &("http://ip-api.com/json/".to_string() + &ip + "?66846719");

    //let url = "http://ip-api.com/json/109.205.213.221?66846719";
    let res = rq::get(url)
        .await?
        .text()
        .await?;
/* let loc = match  iptok.saddr {
        IpAddr::V4(ip4) => {
            let loc = Locator::get_ipv4(ip4, serv).await;
            println!("IP4 {:?}",&loc);
            match loc {
                Ok(loc) => { loc },
                Err(err) => {
                    println!("{:?}",err);
                    return Err(err.into());
                }
            }
        },
        IpAddr::V6(ip6) => {
            let loc = Locator::get_ipv6(ip6, serv).await;
            println!("{:?}", &loc);
            match loc {
                Ok(loc) => { loc },
                Err(err) => {
                    println!("{:?}",err);
                    return Err(err.into());
                }
            }
        },
    };*/
    //let loc = GeoInfo::from(&loc );
    let info: Value  = serde_json::from_str(&res)?;
    let val: GeoInfoIpApiCo = serde_json::from_value(info.clone())?;
    println!("{:?} {:?}", &info, &val);
    Ok(info)
}

/*
async fn get_ggip_db(iptok: &IpTok, ap: &AppData) {
    let client = &ap.mongo;
    let rmtaddr = ap.stream.peer_addr().unwrap().ip();
    let rmtport = ap.stream.peer_addr().unwrap().port();

    //let _ = coll.insert_one()
}*/

pub async fn new_geoip( iptok: &IpTok, conf: &ConfIpInfo) -> Result<Value> {
    //let ip = intruder.ip_v4_address.clone();
    println!("new_geoip {:?}", iptok);
    let serv = conf.geosrv;

    match get_ggip_web( iptok, &serv).await {
        Ok(gi) => { 
            println!("rslt_new_geoip {:?}", gi);    
            return Err(anyhow!("Missing attribute: "));
        },
        Err(err) => {
            println!("{:?}", err);
            return Err(err);
        }
    }
    //println!("{:?}", geoinfo);

    //Ok(geoinfo.unwrap())

}