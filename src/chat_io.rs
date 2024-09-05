use crate::ws::Chat;
use crate::ws::PEDK;
use crate::Clients;
use warp::ws::Message;

pub async fn chat_io(client_id: &str, clients: &Clients, message: Vec<&str>) {
    if message[2] != PEDK {
        #[cfg(test)]
        println!("INVALID APIKEY ATTEMPTED");
        return;
    }

    let chat = Chat {
        user: client_id.to_string(),
        message: message[3].to_string(),
    };

    let locked = clients.lock().await;
    match locked.get(client_id) {
        Some(v) => {
            if let Some(sender) = &v.sender {
                let _ = sender.send(Ok(Message::text("CHT_SELF_RES CHAT_SENT_SUCCESSFULLY")));
            }
        }
        None => return,
    }
    for client in locked.iter() {
        if client.1.client_id != client_id {
            let sender = client.1.sender.clone().unwrap();
            let ms_builder =
                Message::text(format!("CHT_RES {}", serde_json::to_string(&chat).unwrap()));
            #[cfg(test)]
            println!("CHAT MESSAGE SENT: {:?}", ms_builder);
            let _ = sender.send(Ok(ms_builder));
        }
    }
}
