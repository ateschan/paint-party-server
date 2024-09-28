use crate::ws::Chat;
use crate::ws::PEDK;
use crate::Clients;
use warp::ws::Message;
extern crate rand;
use rand::Rng;

pub async fn chat_io(client_id: &str, clients: &Clients, message: Vec<&str>) {
    if message[2] != PEDK {
        #[cfg(test)]
        println!("INVALID APIKEY ATTEMPTED");
        return;
    }

    //init chat
    let mut chat = Chat::default();

    let mut locked = clients.lock().await;

    match locked.get_mut(client_id) {
        Some(v) => {
            if let Some(sender) = &v.sender {
                if v.color == (0, 0, 0) {
                    v.color = gen_color();
                }
                //fill chat with message colot and content
                chat = Chat {
                    user: client_id.to_string(),
                    message: {
                        let mut messagebuilder : String = String::from("");
                        for i in 3..message.len(){
                            if i != 3 {
                                messagebuilder += "_"
                            }
                            messagebuilder += message[i];
                        }
                        messagebuilder
                    },
                    color: v.color,
                };
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

fn gen_color() -> (u8, u8, u8) {
    let r = rand::thread_rng().gen();
    let g = rand::thread_rng().gen();
    let b = rand::thread_rng().gen();
    (r, g, b)
}
