use std::collections::{HashMap};
use std::hash::{Hash, Hasher, BuildHasher};
use chrono::{DateTime, Utc, serde::ts_seconds};
use mongodb::bson::{Document, oid::ObjectId, self};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
//se crate::attack::new_geoip;
use crate::{AppData, intruder::Intruder, Result, ipinfo_coll};
use mongodb::Client;
use std::default::Default;
use ipgeolocate::{Locator, Service, GeoError};
use serde_json::Value;

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
#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
pub struct IpInfo {
    iptok: IpTok,
    ipid: IpId,
    geoinfo: Option<GeoInfo>,
    #[serde(with = "ts_seconds")]
    first: DateTime<Utc>,
}
impl IpInfo {
    async fn init(iptok: &IpTok, ap: &AppData) -> Self {
        let ginfo = match new_geoip(iptok, &ap.conf.ipinfo).await {
            Ok(ginfo) => { Some(ginfo)},
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
    cache: HashMap<IpTok,IpInfo>
}
impl IpInfoCache {
    pub fn init(client: &Client, conf: &ConfIpInfo) -> Self {
        IpInfoCache { mongo: client.clone(), conf: conf.clone(), cache: HashMap::new() }
    }
    pub async fn retrieve(&self, iptok: &IpTok) -> Option<IpInfo> {
        match self.retrieve_from_memory(iptok).await {
            Some(ipi) => { return Some(ipi) },
            None => {
                match self.retrieve_from_database(iptok).await {
                    Some(ipi) => { return Some(ipi)},
                    None => {
                        let ginfo = new_geoip(iptok, &self.conf).await;

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
        let mut cursor = coll.find(ipdoc, None).await;
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
        let ipid = IpId::default();
        let now = Utc::now();
        let ipinfo = IpInfo {
            iptok: iptok.clone(),
            ipid: ipid,
            geoinfo: Some(ginfo),
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


#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
pub struct GeoInfo {
    pub ip: String,
    //pub latitude: Option<String>,
    //pub longitude: Option<String>,
    pub city: String,
    pub region: String,
    pub country: String,
    pub timezone: String,
    //pub location: String,
}

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

async fn get_ggip_web(iptok: &IpTok, serv: &Service) -> anyhow::Result<GeoInfo> {
    let serv = Service::IpApi;
    let loc = match  iptok.saddr {
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
    };
    let loc = GeoInfo::from(&loc );
    //let info  = serde_json::to_value(loc).unwrap();
    println!("{:?}", loc);
    Ok(loc)
}

async fn get_ggip_db(iptok: &IpTok, ap: &AppData) {
    let client = &ap.mongo;
    let rmtaddr = ap.stream.peer_addr().unwrap().ip();
    let rmtport = ap.stream.peer_addr().unwrap().port();

    //let _ = coll.insert_one()
}

pub async fn new_geoip( iptok: &IpTok, conf: &ConfIpInfo) -> Result<GeoInfo> {
    //let ip = intruder.ip_v4_address.clone();
    //println!("{:?}", intruder);
    let serv = conf.geosrv;

    match get_ggip_web( iptok, &serv).await {
        Ok(gi) => { return Ok(gi)},
        Err(err) => {
            return Err(err);
        }
    }
    //println!("{:?}", geoinfo);

    //Ok(geoinfo.unwrap())

}


pub async fn hackback(ap: &AppData) {

}
