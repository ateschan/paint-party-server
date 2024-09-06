use crate::chat_io;
use crate::delete_dots::delete_dots;
use crate::get_dots::get_dots;
use crate::put_dots::put_dots;
use crate::{remove_dots, Client, Clients};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use tokio::fs::write;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::reply::html;
use warp::ws::{Message, WebSocket};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Chat {
    pub message: String,
    pub user: String,
    pub color: (u8, u8, u8),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub size: f32,
    pub id: String,
}

pub static PK: &str = env!("paint_key");
pub static PEK: &str = env!("paint_erase_key");
pub static PEDK: &str = env!("paint_erase_delete_key");

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
        current_room: 0,
        color: (0, 0, 0),
    };

    clients.lock().await.insert(uuid.clone(), new_client);
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("Error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, msg, &clients).await;
    }

    clients.lock().await.remove(&uuid);
    #[cfg(test)]
    println!("{} disconnected", uuid);
}

async fn client_msg(client_id: &str, msg: Message, clients: &Clients) {
    #[cfg(test)]
    println!("received message from {}: {:?}", client_id, msg);

    let raw = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    //So here I would have an if message = GET
    let message: Vec<&str> = raw.split(' ').collect();

    match message[0] {
        //Reads file
        "GET" => {
            get_dots(client_id, clients, message).await;
        }

        //Puts new dots to file, sends update to all
        "PUT" => {
            put_dots(client_id, clients, message).await;
        }

        //Removes all dots from file, sends update to all
        "DEL" => {
            delete_dots(client_id, clients, message).await;
        }

        //Removes specific dots from file
        "RMV" => {
            remove_dots::remove_dots(client_id, clients, message).await;
        }

        //Removes all dots from file, sends update to all
        "CHT" => {
            chat_io::chat_io(client_id, clients, message).await;
        }
        &_ => todo!(),
    }
}

pub fn read_dots_from_file(file: &Path) -> Result<Vec<Dot>, Box<dyn std::error::Error>> {
    let existing_dots: Vec<Dot> = if file.exists() {
        let file = OpenOptions::new().read(true).open(file)?;
        serde_json::from_reader(file)?
    } else {
        Vec::new()
    };
    Ok(existing_dots)
}

pub async fn write_file(
    filename: PathBuf,
    file_contents: Vec<Dot>,
) -> Result<impl warp::Reply, warp::Rejection> {
    write(filename, serialize_dots_to_string(file_contents).unwrap())
        .await
        .unwrap(); // propagate errors with ?
    Ok(html("File written successfully!"))
}

pub fn serialize_dots_to_string(dots: Vec<Dot>) -> Result<String, serde_json::Error> {
    let json_string = serde_json::to_string(&dots)?;
    Ok(json_string)
}
