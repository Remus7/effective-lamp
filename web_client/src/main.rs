/// Documentation for REST calls in Rust can be found at: 
/// https://rust-lang-nursery.github.io/rust-cookbook/web/clients/apis.html

use serde_json::{Value, Map};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use reqwest::Error;
use reqwest;
use std::result::Result;

#[allow(dead_code)]
enum RequestType{
    Get,
    List,
    Put,
    Post,
    Delete
}

#[derive(Serialize, Deserialize, Debug)]
struct Json {
    name: String,
    age: u8,
    is_male: bool,
}

#[allow(dead_code)]
fn demo_json_parse(){
    let json_str = r#"
        {
            "name" : "Remus",
            "age" : 16,
            "is_male" : true
        }
    "#;

    let res_untyped: serde_json::Result<Value> = serde_json::from_str(json_str);
    let res: serde_json::Result<Json> = serde_json::from_str(json_str);

    match res_untyped{
        Ok(val) => {
            println!("{:?}", val);
        },
        Err(e) => panic!("!!Error: {}", e),
    }
    match res{
        Ok(val) => {
            println!("{:?}", val);
        },
        Err(e) => panic!("!!Error: {}", e),
    }
}

async fn request_json<T>(request_url: String) -> Result<T, Error>
where 
    T: DeserializeOwned,    
{
    let response = reqwest::get(&request_url).await?;
    let json = response.json().await?;
    Ok(json)
}

async fn post_json<T>(request_url: String, body: &'static str) -> Result<T, Error>
where 
    T: DeserializeOwned,    
{
    let client = reqwest::Client::new();
    let response = client.post(&request_url).body(body).send().await?;
    let json = response.json().await?;
    Ok(json)
}

#[tokio::main]
async fn main() -> Result<(), Error>{
    let my_request = RequestType::Post;
    // demo_json_parse();
    
    match my_request{
        RequestType::Get => {
            let url = format!("https://jsonplaceholder.typicode.com/todos/1");
            let json = request_json::<Map<String, Value>>(url).await?;
            print!("{:?}", json);
        },
        RequestType::Post => {
            let json_str = r#"
                {
                    "name" : "Remus",
                    "age" : 16,
                    "is_male" : true
                }
            "#;

            let url = format!("https://jsonplaceholder.typicode.com/todos");
            let json = post_json::<Map<String, Value>>(url, json_str).await?;
            print!("{:?}", json);
        },
        _ => unimplemented!(),
    };
    Ok(())
}
