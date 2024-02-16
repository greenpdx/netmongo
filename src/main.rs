use mongodb::{Client, options::ClientOptions, Collection,
    bson::Document};
use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncWriteExt, AsyncBufReadExt},
};
use std::net::{Ipv4Addr, Ipv6Addr};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

mod intruder;
mod telnet;
mod ipcache;
mod attack;

use telnet::{handle_telnet_client};
use intruder::Intruder;
use attack::get_geoip;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    dburl: String,
    hostname: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dburl: "mongodb://10.1.42.239".to_string(),
            hostname: "netapp".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct AppData {
    conf: Config,
    mongo: Client,
    stream: TcpStream,
}

impl AppData {
    pub async fn init(conf: &Config, mongo: &Client, stream: TcpStream) -> Self {
        AppData {
            conf: conf.clone(),
            mongo: mongo.clone(),
            stream: stream,
        }
    }
}

//async fn netapp(client: &Client) {
async fn netapp(ap: AppData)  -> Result<()> {
    //let mut stream = ap.stream;
    let client = &ap.mongo;
    let rmtaddr = ap.stream.peer_addr().unwrap().ip();
    let rmtport = ap.stream.peer_addr().unwrap().port();

    let _coll = open_intruders_collection(&client).await;
    //for db_name in client.list_database_names(None, None).await.expect("Bad") {
    //    println!("{}", db_name);
    //};
    let mut intruder = Intruder::init(rmtaddr, rmtport);
    let _ = handle_telnet_client(&ap, &mut intruder).await;

    get_geoip(&ap, &intruder).await;

    Ok(())
    
}


async fn connect_to_database(conf: &mut Config) -> Client {
    let mut cliopts = ClientOptions::parse(conf.dburl.clone()).await.expect("BAD OPTS");
    cliopts.app_name = Some(conf.hostname.clone());
    let client = Client::with_options(cliopts).expect("NO CLIENT");
    client
}

async fn open_intruders_collection(client: &Client) -> Collection<Document> {
    let db = client.database("netapp");
    let collection = db.collection::<Document>("test");
    collection
}


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let mut conf = Config::default();
    let client = connect_to_database(&mut conf).await;
    
    let listener = TcpListener::bind("0.0.0.0:2223").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let client = client.clone();
        let ap = AppData::init(&conf, &client, stream).await;
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            let _ =netapp(ap).await;
        });
    }

}
