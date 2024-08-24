#[macro_use]
extern crate rocket;
extern crate fern;
extern crate log;
use rocket::tokio::io::AsyncWriteExt;
use rocket::{serde::json::Json, tokio::fs::File};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::Path;
use std::vec::Vec;
pub mod util;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dot {
    pub x: f32,
    pub y: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub size: f32,
}

static API_KEY : &str = "supersecret";

#[get("/<id>/<pass>")]
async fn retrieve(id: &str, pass: &str) -> Option<File> {
    if pass != API_KEY {
        return None;
    }
    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let filename = Path::new(upload_dir).join(id);
    if Path::new(upload_dir).join(id).exists() {
        File::open(&filename).await.ok()
    } else {
        let empty_vec: Vec<Dot> = vec![Dot {
            x: 0.0,
            y: 0.0,
            r: 0,
            g: 0,
            b: 0,
            size: 0.0,
        }];

        let file = rocket::tokio::fs::File::create(&filename).await;
        let data = serde_json::to_string(&empty_vec).unwrap();
        file.expect("").write_all(data.as_bytes()).await;

        File::open(&filename).await.ok()
    }
}

#[get("/<id>/<pass>")]
async fn delete(id: &str, pass: &str) -> std::io::Result<String> {
    
    if pass != API_KEY {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Incorrect password"));
    }
    

    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let file_path = format!("{}/{}", upload_dir, id);

    // Attempt to delete the file directly
    match std::fs::remove_file(&file_path) {
        Ok(_) => Ok(String::from("File deleted successfully")),
        Err(e) => Err(e),
    }


}

#[rocket::post("/<id>/<pass>", data = "<dots>")]
async fn upload<'a>(id: &str, dots: Json<Vec<Dot>>, pass : &str) -> std::io::Result<String> {

    if pass != API_KEY {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Incorrect password"));
    }
    
    let upload_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/", "rooms");
    let filename = Path::new(upload_dir).join(id);

    // Load existing data if the file exists
    let existing_dots: Vec<Dot> = if filename.exists() {
        let file = OpenOptions::new().read(true).open(&filename)?;
        serde_json::from_reader(file)?
    } else {
        Vec::new()
    };

    debug!("EXISTING DOTS: {:?}", existing_dots);
    debug!("NEW DOTS: {:?}", dots.0.iter().cloned());

    // Append new dots to existing data
    let mut layered_proper: Vec<Dot> = Vec::new();
    layered_proper.extend(existing_dots);
    layered_proper.extend(dots.0.iter().cloned());

    // Overwrite the file with the updated data
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&filename)?;

    serde_json::to_writer_pretty(&mut file, &layered_proper)?;

    Ok(String::from("OK"))
}

#[launch]
fn rocket() -> _ {
    //logger for debugging
    //let _ = setup_logger();
    rocket::build()
        .mount("/", routes![retrieve, upload])
        .mount("/delete/", routes![delete])
}

pub fn get_unique_dots(dots: &mut Vec<Dot>) -> Vec<Dot> {
    let mut result: Vec<Dot> = Vec::new();
    for dot in dots {
        if !result.contains(dot) {
            result.push(dot.clone());
        }
    }
    result
}
