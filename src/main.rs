use mongodb::{Client, options::ClientOptions, Collection,
    bson::Document};
use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncWriteExt, AsyncBufReadExt}
};
use std::error::Error;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
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
struct AppData {
    conf: Config,
    mongo: Option<Client>,
    stream: TcpStream,

}


//async fn netapp(client: &Client) {
async fn netapp(stream: &TcpStream, client: Client)  {
    let coll = open_intruders_collection(&client).await;
    //for db_name in client.list_database_names(None, None).await.expect("Bad") {
    //    println!("{}", db_name);
    //};

    
}


pub async fn connect_to_database(conf: &mut Config) -> Client {
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

        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            netapp(&stream, client).await;
        });
    }

}
