use std::collections::{HashMap};
use std::hash::{Hash, Hasher, BuildHasher};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::{AppData, intruder::Intruder};
use mongodb::Client;

#[derive(Debug, Clone,Deserialize, Serialize, Eq, PartialEq, Hash)]
pub struct IpTok {
    saddr: IpAddr,
    sport: u16,
}

#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
pub struct IpId {
    saddr: IpAddr,
    sport: u16,
    id: ObjectId,
}

#[derive(Debug, Clone,Deserialize, Serialize, Hash)]
struct IpInfo {

}

#[derive(Debug, Clone)]
pub struct IpIdCache{
    mongo: Client,
    cache: HashMap<IpTok,IpId>
}
impl IpIdCache {
    fn init(ap: AppData) -> Self {
        IpIdCache { mongo: ap.mongo.clone(), cache: HashMap::new() }
    }
    async fn retrieve(&self, intruder: &Intruder) -> Option<IpInfo> {
        None   
    }
    fn retrieve_from_memory(&self, intruder: &Intruder) -> Option<IpInfo> {
        None
    }
    async fn retrieve_from_database(&self, intruder: &Intruder) -> Option<IpInfo> {
        None
    }
    fn add(&mut self, intruder: &Intruder) {

    }
}
