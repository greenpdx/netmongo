use mongodb::{Client, options::ClientOptions, Collection};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex
    //io::{AsyncWriteExt, AsyncBufReadExt},
};
//use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
//use std::collections::HashMap;
//use serde::{Serialize, Deserialize};
//use chrono::{DateTime, Utc};
//use std::str::FromStr;
//use ipgeolocate::{Locator, Service, GeoError};
use std::sync::Arc;


use clap::Parser;
mod intruder;
mod telnet;
mod ipcache;
//mod attack;

use telnet::handle_telnet_client;
use intruder::Intruder;
//use attack::new_geoip;
use ipcache::{IpInfo, IpTok, ConfIpInfo, IpInfoCache};
use anyhow::Result;

//type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type CacheMap = Arc<Mutex<IpInfoCache>>;
//type CacheMap = Arc<Mutex<HashMap<IpTok, &'static IpInfo>>>;

#[derive(Debug, Clone)]
pub struct Config {
    dburl: String,
    hostname: String,
    _database: String,
    ipinfo: ConfIpInfo,
}

impl Config {
    async fn init(_args: &Args) -> Self {
        let confipinfo = ConfIpInfo::default();
        Config {
            dburl: "mongodb://10.1.42.239".to_string(),
            hostname: "netapp".to_string(),
            _database: "netapp".to_string(),
            ipinfo: confipinfo,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: Option<String>,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[derive(Debug)]
pub struct AppData {
    _conf: Config,
    mongo: Client,
    stream: TcpStream,
}

impl AppData {
    pub async fn init(conf: &Config, mongo: &Client, stream: TcpStream) -> Self {
        AppData {
            _conf: conf.clone(),
            mongo: mongo.clone(),
            stream: stream,
        }
    }
}

//async fn netapp(client: &Client) {
//async fn netapp(ap: AppData, cache: &Arc<std::sync::Mutex<IpInfoCache>>)  -> Result<()> {
async fn netapp(ap: AppData, cache: &CacheMap )  -> Result<()> {
    //let mut stream = ap.stream;
    let client = &ap.mongo;
    let srcaddr = ap.stream.peer_addr().unwrap().ip();
    let dstport = ap.stream.local_addr().unwrap().port();

    let _coll = open_intruders_collection(&client).await;
    //for db_name in client.list_database_names(None, None).await.expect("Bad") {
    //    println!("{}", db_name);
    //};
    //let srcaddr = IpAddr::from_str(&"109.205.213.221").unwrap();
    let mut intruder = Intruder::init(srcaddr, dstport);

    let _ = handle_telnet_client(&ap, &mut intruder, cache).await;

    Ok(())
    
}

async fn connect_to_database(conf: &mut Config) -> Client {
    let mut cliopts = ClientOptions::parse(conf.dburl.clone()).await.expect("BAD OPTS");
    cliopts.app_name = Some(conf.hostname.clone());
    let client = Client::with_options(cliopts).expect("NO CLIENT");
    client
}

async fn open_intruders_collection(client: &Client) -> Collection<Intruder> {
    let db = client.database("netapp");
    let collection = db.collection::<Intruder>("intruders");
    collection
}

pub async fn ipinfo_coll(client: &Client, _iptok: &IpTok) -> Collection<IpInfo> {
    let db = client.database("netapp");
    let collection = db.collection::<IpInfo>("ipinfo");
    collection
}
pub async fn save_ipinfo_collection(client: &Client, _ipinfo: &IpInfo) {
    let db = client.database("netapp");
    let _collection = db.collection::<IpInfo>("ipinfo");

  
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let args = Args::parse();

    let mut conf = Config::init(&args).await;
    let client = connect_to_database(&mut conf).await;
    
    let listener = TcpListener::bind("0.0.0.0:2223").await.unwrap();

    let cache = Arc::new(Mutex::new(IpInfoCache::init(&client, &conf.ipinfo)));
    //let cache = Arc::new(Mutex::new(HashMap::new()));
    //let cache = Arc::new(HashMap::new());
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let client = client.clone();
        let ap = AppData::init(&conf, &client, stream).await;
        let cache = cache.clone();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        //tokio::spawn(async move {
        tokio::spawn(async move {
            let _ = netapp(ap, &cache).await;
        });
    }

}
