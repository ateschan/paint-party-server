use crate::ws::{self, Dot};
use crate::ws::{read_dots_from_file, write_file};
use crate::ws::{PEDK, PEK};
use crate::Clients;
use std::env;
use std::path::Path;
use warp::ws::Message;

pub async fn remove_dots(client_id: &str, clients: &Clients, message: Vec<&str>) {
    if message[2] != PEK && message[2] != PEDK {
        #[cfg(test)]
        println!("INVALID APIKEY ATTEMPTED");
        return;
    }
    
    //BUG NEEDS TO BE CHECKED
    let dot_ids: Vec<String> = ws::deserialize_dot_ids_from_string(message[3]).unwrap();

    let locked = clients.lock().await;
    for client in locked.iter() {
        if client.1.client_id != client_id && client.1.current_room.to_string() == message[1] {
            let sender = client.1.sender.clone().unwrap();

            let ms_builder = Message::text(format!(
                "RMV_RES {}",
                serde_json::to_string(&dot_ids).unwrap()
            ));
            #[cfg(test)]
            println!("REMOVE MESSAGE SENT: {:?}", ms_builder);
            let _ = sender.send(Ok(ms_builder));
        }
    }

    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let filename = Path::new(upload_dir).join(message[1]);
    let mut existing_dots: Vec<Dot> = read_dots_from_file(&filename).unwrap();
    existing_dots.retain(|dot| !dot_ids.contains(&dot.id));
    let _ = write_file(filename, existing_dots).await;
}
