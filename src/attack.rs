use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::{AppData, intruder::Intruder};
use mongodb::Client;
use geolocation::{find, Locator};

use std::fmt::{self, Debug, Formatter};

#[derive(Clone,Deserialize, Serialize, Eq, PartialEq, Hash)]
pub struct GeoInfo {
    pub ip: String,
    pub latitude: String,
    pub longitude: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub timezone: String,
    pub location: String,
}
impl Debug for GeoInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Locator")
            .field("ip", &self.ip)
            .field("latitude", &self.latitude)
            .field("longitude", &self.longitude)
            .field("city", &self.city)
            .field("region", &self.region)
            .field("country", &self.country)
            .field("timezone", &self.timezone)
            .field("location", &self.location)
            .finish();

        Ok(())
    }
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
            location: loc.location.clone(),
        }
    }
}

async fn get_ggip_web(intruder: &Intruder) {
    let ip = intruder.ip.clone();
    let loc = GeoInfo::from(&find(&ip).unwrap());
    let info  = serde_json::to_value(loc).unwrap();
    println!("{:?}", info);
}

pub async fn get_geoip(ap: &AppData, intruder: &Intruder) {
    get_ggip_web(intruder).await;

}