use crate::{Client, Clients};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use tokio::fs::{write, File};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::reply::{html, with_status};
use warp::ws::{Message, WebSocket};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub size: f32,
    pub id : String
}

static API_KEY: &str = "supersecretapikey";

pub async fn client_connection(ws: WebSocket, clients: Clients) {
    println!("establishing client connection... {:?}", ws);

    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let uuid = Uuid::new_v4().simple().to_string();

    let new_client = Client {
        client_id: uuid.clone(),
        sender: Some(client_sender),
    };

    clients.lock().await.insert(uuid.clone(), new_client);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, msg, &clients).await;
    }

    clients.lock().await.remove(&uuid);
    println!("{} disconnected", uuid);
}

async fn client_msg(client_id: &str, msg: Message, clients: &Clients) {
    println!("received message from {}: {:?}", client_id, msg);

    let raw = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    //So here I would have an if message = GET
    let message: Vec<&str> = raw.split(' ').collect();

    if message[0] == "GET" || message[0] == "GET\n" {
        if API_KEY != message[2] || !is_i32(message[1]) {
            println!("INVALID APIKEY ATTEMPTED");
            return;
        }
        let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
        let file_path = format!("{}/{}", upload_dir, message[1]);
        println!("{:?}", file_path);
        let file = match Path::new(&file_path).exists() {
            true => File::open(&file_path).await,
            false => {
                let empty_vec: Vec<Dot> = vec![Dot {
                    x: 0.0,
                    y: 0.0,
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                    size: 0.0,
                    id : "".to_string()
                }];

                let _ = tokio::fs::File::create(&file_path).await;
                let data = serde_json::to_string(&empty_vec).unwrap();
                match write(&file_path, data.as_bytes()).await {
                    Ok(a) => println!("File written successfully {:?}", a),
                    Err(e) => eprintln!("Could not write to file {:?}", e),
                };

                File::open(&file_path).await
            }
        };

        let mut contents = vec![];
        let _ = file.unwrap().read_to_end(&mut contents).await;

        let locked = clients.lock().await;
        match locked.get(client_id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    println!("GET Recieved! File at {:?}", file_path);
                    let _ = sender.send(Ok(Message::text(format!(
                        "GET_RES {}",
                        from_utf8(&contents).unwrap()
                    ))));
                }
            }
            None => return,
        }
        return;
    };

    if message[0] == "PUT" || message[0] == "PUT\n" {
        if API_KEY != message[2] || !is_i32(message[1]) {
            println!("INVALID APIKEY ATTEMPTED");
            return;
        }

        //FOR DEBUGGING
        let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
        let filename = Path::new(upload_dir).join(message[1]);
        let existing_dots: Vec<Dot> = read_dots_from_file(&filename).unwrap();

        //let raw : String = format!("{}", raw[2]);
        let dots: Vec<Dot> = serde_json::from_str(message[3]).unwrap();

        let mut layered_proper: Vec<Dot> = Vec::new();
        layered_proper.extend(existing_dots);
        layered_proper.extend(dots.iter().cloned());

        println!("OPENING: {:?}", filename);
        println!("DEPOSITING: {:?}", layered_proper);

        let _ = write_file(filename, layered_proper).await;

        let locked = clients.lock().await;
        match locked.get(client_id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    println!("GET Recieved!");
                    let _ = sender.send(Ok(Message::text("PUT_RES FILE_WRITTEN_SUCCESSFULLY")));
                }
            }
            None => return,
        }

        for client in locked.iter() {
            if client.1.client_id != client_id {
                let sender = client.1.sender.clone().unwrap();

                let ms_builder = Message::text(format!(
                    "UPD_RES {} {}", 
                    message[1],
                    serde_json::to_string(&dots).unwrap()
                ));

                println!("UPDATE MESSAGE SENT: {:?}", ms_builder);

                let _ = sender.send(Ok(ms_builder));

            }
        }

        return;
    };

    if message[0] == "DEL" || message[0] == "DE:\n" {
        if API_KEY != message[2] || !is_i32(message[1]) {
            println!("INVALID APIKEY ATTEMPTED");
            return;
        }
        //FOR DEBUGGING
        let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
        let filename = Path::new(upload_dir).join(message[1]);
        println!("DELETING: {:?}", filename);

        let _ = tokio::fs::remove_file(filename).await;

        let locked = clients.lock().await;
        match locked.get(client_id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    let _ = sender.send(Ok(Message::text("DEL_RES FILE_DELETED_SUCCESSFULLY")));
                }
            }
            None => return,
        }
    };
}

fn read_dots_from_file(file: &Path) -> Result<Vec<Dot>, Box<dyn std::error::Error>> {
    let existing_dots: Vec<Dot> = if file.exists() {
        let file = OpenOptions::new().read(true).open(file)?;
        serde_json::from_reader(file)?
    } else {
        Vec::new()
    };

    Ok(existing_dots)
}

async fn write_file(
    filename: PathBuf,
    file_contents: Vec<Dot>,
) -> Result<impl warp::Reply, warp::Rejection> {
    write(filename, serialize_dots_to_string(file_contents).unwrap()).await; // propagate errors with ?
    Ok(html("File written successfully!"))
}

fn serialize_dots_to_string(dots: Vec<Dot>) -> Result<String, serde_json::Error> {
    let json_string = serde_json::to_string(&dots)?;
    Ok(json_string)
}

fn is_i32(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}
