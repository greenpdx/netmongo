use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::{AppData, intruder::Intruder, Config, find_ipinfo_collection, ipcache::{IpTok, ConfIpInfo}, Result};
use mongodb::Client;
use ipgeolocate::{Locator, Service, GeoError};



#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
pub struct GeoInfo {
    pub ip: String,
    pub latitude: String,
    pub longitude: String,
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
            latitude: loc.latitude.clone(),
            longitude: loc.longitude.clone(),
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
            println!("{:?}",loc);
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

    let _coll = find_ipinfo_collection(&client, iptok).await;
  
}

pub async fn new_geoip( iptok: &IpTok, conf: &ConfIpInfo) -> Result<GeoInfo> {
    //let ip = intruder.ip_v4_address.clone();
    //println!("{:?}", intruder);
    let serv = conf.geosrv;

    let geoinfo = get_ggip_web( iptok, serv).await;
    Ok(geoinfo.unwrap())

}


pub async fn hackback(ap: &AppData) {

}
