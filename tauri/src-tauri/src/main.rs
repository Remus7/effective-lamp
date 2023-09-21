// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused_imports)]
use reqwest::get;
#[allow(unused_imports)]
use serde_json::{Value, Map};
#[allow(unused_imports)]
use std::{thread, time};
#[allow(unused_imports)]
use tauri::Window;

use serde::Serialize;

const KEY: &str = "845bd16319b1f602272aaa589497d94c";

#[derive(Debug, Serialize)]
pub enum CommandError{
    Error(String)
}

async fn request_value(url: String) -> Result<Value, CommandError>{
    let response = reqwest::get(url).await.map_err(|err| CommandError::Error(format!("{:?}", err)))?.text().await.map_err(|err| CommandError::Error(format!("{:?}", err)))?;

    Ok( serde_json::from_str(&response).map_err(|_err| CommandError::Error("What are you doing with your life".to_owned()))? )
}

#[tauri::command]
async fn get_temperature(latitude: f64, longitude: f64) -> Result<String, CommandError> {
    let url = format!("https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}", latitude, longitude, KEY);
    println!("Getting temperature: {}", url);

    let data = request_value(url).await?;
    // println!("{:?}", data);

    let temp = data["main"]["temp"].as_f64().unwrap() - 273.15;
    Ok(format!("{:.2}", temp))
}

#[tauri::command]
async fn get_location(city: String, state: String, country: String) -> Result<(f64, f64), CommandError>{
    let url = format!("http://api.openweathermap.org/geo/1.0/direct?q={city},{state},{country}&appid={KEY}");
    println!("Getting location: {}", url);

    let data = request_value(url).await?;
    // println!("{:?}", data);

    let lat = data[0]["lat"].as_f64().unwrap();
    let lon = data[0]["lon"].as_f64().unwrap();
    Ok((lat, lon))
}

#[tauri::command]
async fn get_image(latitude: f64, longitude: f64) -> Result<String, CommandError>{
    let url = format!("https://api.openweathermap.org/data/2.5/weather?lat={latitude}&lon={longitude}&appid={KEY}");
    println!("Getting image: {}", url);

    let data = request_value(url).await?;
    let id = data["weather"][0]["icon"].as_str().unwrap().to_owned();
    // id = id[..id.len()- 1].to_owned();

    Ok(format!("https://openweathermap.org/img/wn/{id}@2x.png"))
}

#[tauri::command]
async fn get_city(latitude: f64, longitude: f64) -> Result<(String, String), CommandError>{
    let url = format!("https://api.openweathermap.org/data/2.5/weather?lat={latitude}&lon={longitude}&appid={KEY}");
    println!("Getting image: {}", url);

    let data = request_value(url).await?;
    let city = match data["name"].as_str(){
        Some(msg) => msg.to_owned(),
        None => "???".to_owned(),
    };
    let country = match data["sys"]["country"].as_str(){
        Some(msg) => msg.to_owned(),
        None => "???".to_owned(),
    };

    Ok((city, country))
}

#[tauri::command]
fn init_process(window: Window, time: u64){
    println!("Starting thread");
    thread::spawn(move ||{
        loop{
            thread::sleep(time::Duration::from_secs(time));
            window.emit("getlocation", ()).unwrap();
        }
    });
}
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_temperature,
            get_location,
            get_image,
            get_city,
            init_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
