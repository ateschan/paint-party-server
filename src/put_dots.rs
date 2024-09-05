use crate::ws::Dot;
use crate::ws::{read_dots_from_file, write_file};
use crate::ws::{PEDK, PEK, PK};
use crate::Clients;
use std::env;
use std::path::Path;
use warp::ws::Message;

pub async fn put_dots(client_id: &str, clients: &Clients, message: Vec<&str>) {
    if message[2] != PK && message[2] != PEK && message[2] != PEDK {
        #[cfg(test)]
        println!("INVALID APIKEY ATTEMPTED");
        return;
    }

    let dots: Vec<Dot> = serde_json::from_str(message[3]).unwrap();
    let locked = clients.lock().await;
    for client in locked.iter() {
        if client.1.client_id != client_id && client.1.current_room.to_string() == message[1] {
            let sender = client.1.sender.clone().unwrap();
            let ms_builder = Message::text(format!(
                "UPD_RES {} {}",
                message[1],
                serde_json::to_string(&dots).unwrap()
            ));
            #[cfg(test)]
            println!("UPDATE MESSAGE SENT: {:?}", ms_builder);
            let _ = sender.send(Ok(ms_builder));
        }
    }

    //FOR DEBUGGING
    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let filename = Path::new(upload_dir).join(message[1]);
    let existing_dots: Vec<Dot> = read_dots_from_file(&filename).unwrap();

    //let raw : String = format!("{}", raw[2]);
    let mut layered_proper: Vec<Dot> = Vec::new();
    layered_proper.extend(existing_dots);
    layered_proper.extend(dots.iter().cloned());
    #[cfg(test)]
    println!("OPENING: {:?}", filename);
    #[cfg(test)]
    println!("DEPOSITING: {:?}", layered_proper);

    let _ = write_file(filename, layered_proper).await;

    if let Some(value) = locked.get(client_id) {
        if let Some(sender) = &value.sender {
            println!("PUT received!");
            let _ = sender.send(Ok(Message::text("PUT_RES FILE_WRITTEN_SUCCESSFULLY")));
        }
    }
}
