use crate::ws::Dot;
use crate::Clients;
use std::env;
use std::path::Path;
use std::str::from_utf8;
use tokio::fs::{write, File};
use tokio::io::AsyncReadExt;
use warp::ws::Message;

pub async fn get_dots(client_id: &str, clients: &Clients, message: Vec<&str>) {
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
                id: "".to_string(),
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
    let mut locked = clients.lock().await;
    match locked.get(client_id) {
        Some(v) => {
            if let Some(sender) = &v.sender {
                #[cfg(test)]
                println!("GET Recieved! File at {:?}", file_path);
                let _ = sender.send(Ok(Message::text(format!(
                    "GET_RES {}",
                    from_utf8(&contents).unwrap()
                ))));
            }
        }
        None => return,
    }

    let current = locked.get_mut(client_id).unwrap();
    #[cfg(test)]
    println!(
        "CLIENT IS IN ROOM {} CHANGING TO {}",
        current.current_room, message[1]
    );
    current.current_room = message[1].parse::<i32>().unwrap();
}
