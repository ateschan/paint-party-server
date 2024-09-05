use crate::ws::PEDK;
use crate::Clients;
use std::env;
use std::path::Path;
use warp::ws::Message;

pub async fn delete_dots(client_id: &str, clients: &Clients, message: Vec<&str>) {
    if message[2] != PEDK {
        #[cfg(test)]
        println!("INVALID APIKEY ATTEMPTED");
        return;
    }
    //FOR DEBUGGING
    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let filename = Path::new(upload_dir).join(message[1]);
    #[cfg(test)]
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
    for client in locked.iter() {
        if client.1.client_id != client_id && client.1.current_room.to_string() == message[1] {
            let sender = client.1.sender.clone().unwrap();
            let ms_builder = Message::text(format!("CLR_RES {}", message[1]));
            #[cfg(test)]
            println!("CLEAR MESSAGE SENT: {:?}", ms_builder);
            let _ = sender.send(Ok(ms_builder));
        }
    }
}
