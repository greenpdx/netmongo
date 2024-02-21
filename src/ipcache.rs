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
    async fn init(iptok: &IpTok, ap: &AppData) -> Self {
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
    pub async fn retrieve(&mut self, iptok: &IpTok) -> Option<IpInfo> {
        //let iptok = IpTok { saddr: IpAddr::from ([109,205,213,221]), dport: 23};
;

        match self.retrieve_from_memory(&iptok).await {
            Some(ipi) => { return Some(ipi) },
            None => {
                match self.retrieve_from_database(&iptok).await {
                    Some(ipi) => { return Some(ipi)},
                    None => {
                        let ginfo = match new_geoip(&iptok, &self.conf).await {
                            Ok(gi) => {
                                let ipid = IpId::default();
                                let ginfo = match serde_json::from_value(gi.clone()) {
                                    Ok(val) => {
                                        GeoInfo::IpApiCo(val)
                                    },
                                    Err(err) => {
                                        println!("{:?}",err);
                                        return None
                                    }
                                };

                                let ipinfo = self.add(&iptok, &ginfo, &ipid).await;
                                match ipinfo {
                                    Ok(ii) => { return Some(ii) },
                                    Err(err) => {
                                        println!("{:?}",err);
                                        return None
                                    }
                                } 
                            },
                            Err(err) => {

                            }
                        };

                        return None
                    }
                }
            }
        }
    }
    async fn retrieve_from_memory(&self, iptok: &IpTok) -> Option<IpInfo> {
        let rslt = self.cache.contains_key(iptok);
        let ipinfo: Option<IpInfo> = if rslt {
            let ipinfo = self.cache.get(iptok).unwrap().clone();
            println!("MEM {:?}", ipinfo);
            Some(ipinfo)
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
            println!("DB {:?}", ipinfo);
            return Some(ipinfo)
        } else {
            return None
        }
    }
    async fn add(&mut self, iptok: &IpTok, ginfo: &GeoInfo, ipid: &IpId) -> Result<IpInfo>{
    
        let now = Utc::now();
        let ipinfo = IpInfo {
            iptok: iptok.clone(),
            ipid: ipid.clone(),
            geoinfo: Some(ginfo.clone()),
            first: now,
        };
        
        let coll = ipinfo_coll(&self.mongo, iptok).await;
        let rslt = match coll.insert_one(&ipinfo, None).await {
            Ok(id) => { 
                println!("{:?}",id);
                id
            },
            Err(err) => {
                println!("{:?}", &err);
                return Err(err.into());
            }
        };
        self.cache.insert(iptok.clone(), ipinfo.clone());
        println!("ADD {:?}", ipinfo);
        Ok(ipinfo)

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

    let res = rq::get(url)
        .await?
        .text()
        .await?;

    //let loc = GeoInfo::from(&loc );
    let info: Value  = serde_json::from_str(&res)?;
    let val: GeoInfoIpApiCo = serde_json::from_value(info.clone())?;
    let ginfo = GeoInfo::IpApiCo(val);
    //println!("{:?} {:?}", &info, &ginfo);
    Ok(info)
}

pub async fn new_geoip( iptok: &IpTok, conf: &ConfIpInfo) -> Result<Value> {
    //let ip = intruder.ip_v4_address.clone();
    println!("new_geoip {:?}", iptok);
    let serv = conf.geosrv;

    match get_ggip_web( iptok, &serv).await {
        Ok(gi) => { 
            println!("rslt_new_geoip {:?}", gi);    
            return Ok(gi);
        },
        Err(err) => {
            println!("{:?}", err);
            return Err(err);
        }
    }
    //println!("{:?}", geoinfo);

    //Ok(geoinfo.unwrap())

}

#[cfg(test)]
mod tests {
    use std::sync::{Arc};
    use tokio::sync::Mutex;

    use crate::{Config, connect_to_database, Args, CacheMap, Intruder}; 
    use mongodb::Database;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;


    const ips: [[u8;4];11] = [
        [5,188,86,212],
        [79,110,62,122],
        [141,98,11,91],
        [167,94,138,2],
        [109,205,213,104],
        [35,216,133,23],
        [162,142,125,140],
        [162,216,150,166],
        [80,66,83,114],
        [183,136,225,48],
        [77,90,185,72],
    ];
    async fn ic_db_clear(db: &Database) {
        let coll = db.collection::<Intruder>("intruders");
        let rslt = coll.drop(None).await;
        let coll = db.collection::<IpInfo>("ipinfo");
        let rslt = coll.drop(None).await;

    }

    async fn ic_conn_db() -> Client {
        let args = Args { name: None, count: 0};
        let mut conf = Config::init(&args).await;
        let client = connect_to_database(&mut conf).await;
        client
    }

    async fn ic_cache_new(clear: bool) -> CacheMap {
        let client = ic_conn_db().await;
        let db = client.database("ts_netapp");
        if clear {ic_db_clear(&db).await;};  
        let ipconf = ConfIpInfo::default();
        let cache = Arc::new(Mutex::new(IpInfoCache::init(&client, &ipconf)));

        cache
    }

    #[tokio::test]
    async fn ic_cache_ip_new( ) {
        let iptok = IpTok { saddr: IpAddr::from([109,205,213,221]), dport: 23 };
        let conf = ConfIpInfo::default();
        let cache = ic_cache_new(true).await;
        let ipinfo = cache.lock().await.retrieve(&iptok).await;
        println!("0 {:?}", ipinfo);
        let ipinfo = cache.lock().await.retrieve(&iptok).await;
        println!("1 {:?}", ipinfo);

    }

    #[tokio::test]
    async fn ic_cache_ip_find() {
        //let iptok = IpTok { saddr: IpAddr::from([109,205,213,221]), dport: 23 };
        //let conf = ConfIpInfo::default();
        let mcache = ic_cache_new(false).await;
        let mut tasks = Vec::with_capacity(ips.len());

        for (idx, ip) in ips.iter().enumerate() {
            let ipaddr = IpAddr::from(*ip);
            let iptok = IpTok { saddr: ipaddr, dport: 23};
            println!("IP {:?}", &iptok);
            let cache = mcache.clone();
            tasks.push(tokio::spawn(async move {
                let ipinfo = &cache.lock().await.retrieve(&iptok).await;
                println!("LOOP {:?}", idx);
            }));
        };
        for task in tasks {
            println!("{:?}",task.await.unwrap());
        }
        //println!("2 {:?}", ipinfo);


    }
}

