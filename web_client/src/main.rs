/// Documentation for REST calls in Rust can be found at: 
///     https://rust-lang-nursery.github.io/rust-cookbook/web/clients/apis.html
///     https://docs.rs/reqwest/latest/reqwest/
///
/// Guide for API url depending on request type:
///     https://jsonplaceholder.typicode.com/guide/

use serde_json::{Map, Value};
use reqwest::Error;
use reqwest;
use std::result::Result;
use std::io;

#[allow(dead_code)]
enum RequestType{
    Get,
    List,
    Post,
    Put,
    Delete
}

fn try_u32_to_request(x: u32) -> Result<RequestType, &'static str>{
    match x{
        0 => Ok(RequestType::Get),
        1 => Ok(RequestType::List),
        2 => Ok(RequestType::Post),
        3 => Ok(RequestType::Put),
        4 => Ok(RequestType::Delete),
        _ => Err("Request doesn't exist"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    println!("Choose request type:\n   [0] Get\n   [1] List\n   [2] Post");

    let mut input_text = String::new();
    io::stdin()
        .read_line(&mut input_text)
        .expect("failed to read from stdin");

    let trimmed = input_text.trim();
    let my_request = match trimmed.parse::<u32>() {
        Ok(i) => try_u32_to_request(i),
        Err(..) => panic!("Not an integer"),
    };

    match my_request {
        Ok(request) => {
            println!("Setting up request");
            match request{
                RequestType::Get => {
                    let url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
                    let response = reqwest::get(&url).await?;
                    let json : Map<String, Value> = response.json().await?;

                    println!("{:?}", json);
                },
                RequestType::Post => {
                    let url = "https://jsonplaceholder.typicode.com/todos".to_owned();
                    let client = reqwest::Client::new();

                    // // Uncomment to set up json from HashMap
                    // let mut json = std::collections::HashMap::new();
                    // json.insert("lang", "rust");
                    // json.insert("body", "json");

                    // Uncomment to set up json from json! macro
                    let json = serde_json::json!({
                        "lang" : "rust",
                        "body" : "json",
                    });

                    let response = client.post(&url).json(&json).send().await?;
                    let json : Map<String, Value> = response.json().await?;

                    println!("{:?}", json);
                },
                RequestType::List => {
                    let url = "https://jsonplaceholder.typicode.com/posts".to_owned();
                    let response = reqwest::get(&url).await?;
                    let json : Vec<Map<String, Value>> = response.json().await?;

                    for i in &json{
                        println!("{:?}\n", i);
                    }
                },
                RequestType::Delete => {
                    let client = reqwest::Client::new();
                    let response = client.delete("https://jsonplaceholder.typicode.com/posts/1").send().await?;
                    let json: Map<String, Value> = response.json().await?;
                    println!("{:?}", json);
                }
                _ => unimplemented!(),
            } 
        },
        Err(e) => println!("{}", e),
    }
    Ok(())
}
