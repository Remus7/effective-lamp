use reqwest;
use serde_json::{Map, Value};
use anyhow;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::io::Write;
use curl::easy::{Easy, List};

use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey, read_service_account_key};
use std::path::Path;
use reqwest::header::AUTHORIZATION;

static FOLDER_ID: &str = "0BzIGYtKj20XuZjFOekxHUWE2WlE";
static EXCEL_MIME_TYPE: &str = "application/vnd.google-apps.spreadsheet";
static FOLDER_MIME_TYPE: &str = "application/vnd.google-apps.folder";

static PHOTOS_NAME: &str = "people";

#[derive(PartialEq)]
#[allow(dead_code)]
enum Task{
    FetchFiles,
    FetchPhotos,
}

/// Get the authenticator token for the service account
/// Necessary in order to establish the scope of the request
async fn get_auth_token() -> Result<String, anyhow::Error>{
    let service_account_key : ServiceAccountKey = read_service_account_key(Path::new("../../systems-cs-pub-ro-497f6e6f3774.json")).await?;
    let authenticator = ServiceAccountAuthenticator::builder(service_account_key).build().await.expect("Failed to create authentication");
    let scopes = &["https://www.googleapis.com/auth/drive.readonly"];

    let result = authenticator.token(scopes).await?;
    match result.token(){
        Some(t) => Ok(t.to_owned()),
        None => Err(anyhow::anyhow!("No token found")),
    }
}

/// List all files inside a folder, identified by a folder_id
async fn list_files(auth_token: &str, folder_id: &str) -> Result<Value, anyhow::Error>{
    let endpoint = "https://www.googleapis.com";
    let query = format!("'{folder_id}'+in+parents");
    let url = format!("{endpoint}/drive/v3/files?q={query}");
    // println!("{}", url);

    let client = reqwest::Client::new();
    let response = client.get(&url).header(AUTHORIZATION, format!("Bearer {auth_token}")).send().await?.text().await?;
    let json = serde_json::from_str(&response)?;
    Ok(json)
}

/// Export spreadsheet file, indentified by file_id, and convert it to json format
async fn convert_content(auth_token: &str, file_id: &str) -> Result<Value, anyhow::Error>{
    let endpoint = "https://www.googleapis.com";
    let url = format!("{endpoint}/drive/v3/files/{file_id}/export?mimeType=text/csv");
    // println!("{}", url);

    let client = reqwest::Client::new();
    let csv_file = client.get(&url).header(AUTHORIZATION, format!("Bearer {auth_token}")).send().await?.text().await?; // Fetch raw csv content

    let filename = "csv_text.csv";
    let mut file = File::create(filename)?;
    file.write_all(csv_file.as_bytes())?;

    let csv2json = Command::new("csv2json")     // Declare 'csv2json' command
        .arg("--in")                            // Pass input file
        .arg(filename)
        .output()                               // Pipe the output to current variable
        .expect("csv2json command failed to start");

    fs::remove_file(&format!("./{filename}"))?;

    let json_text = String::from_utf8(csv2json.stdout)?;
    let json : Value = serde_json::from_str(json_text.as_str())?;
    Ok(json)
}

/// Download image, indentified by file_id
async fn download_image(file_name: &str, auth_token: &str, file_id: &str, mime_type: &str) -> Result<(), anyhow::Error>{
    let endpoint = "https://www.googleapis.com";
    let url = format!("{endpoint}/drive/v3/files/{file_id}?mimeType={mime_type}&alt=media");
    // println!("{}", url);

    let mut file = File::create(&format!("{PHOTOS_NAME}/{file_name}"))?;

    let mut handle = Easy::new();
    handle.url(&url)?;
    handle.write_function(move |data| {
        file.write_all(data).unwrap();
        Ok(data.len())
    })?;

    let mut list = List::new();
    list.append(&format!("Authorization: Bearer {auth_token}"))?;
    handle.http_headers(list)?;

    handle.perform()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
    //let current_task = Task::FetchPhotos;

    let token = get_auth_token().await?;
    let folder_list = list_files(&token, FOLDER_ID).await?;
    let all_files_id = Value::as_array(&folder_list["files"]).unwrap();

    let mut all_files_content = Map::<String, Value>::new();

    //match current_task {
    //    Task::FetchFiles => {
            for i in all_files_id.iter(){
                let file_type = i["mimeType"].as_str().unwrap();

                if file_type == EXCEL_MIME_TYPE {
                    let file_id = i["id"].as_str().unwrap();
                    let file_content = convert_content(&token, file_id).await?;
                    let file_name = i["name"].as_str().unwrap();

                    all_files_content.insert(file_name.to_owned(), file_content);
                }
            }
            let json_result = Value::Object(all_files_content);
            let mut file = File::create("json_result.json")?;
            file.write_all(json_result.to_string().as_bytes())?;
    //     },

    //     Task::FetchPhotos => {
            let dir_path = &format!("./{PHOTOS_NAME}");
            if Path::new(dir_path).is_dir() {
                fs::remove_dir_all(PHOTOS_NAME)?;
            }
            fs::create_dir(dir_path)?;

            for i in all_files_id.iter(){
                let file_type = i["mimeType"].as_str().unwrap();
                let file_name = i["name"].as_str().unwrap();

                if file_type == FOLDER_MIME_TYPE && file_name == "people" {
                    let folder_id = i["id"].as_str().unwrap();
                    let all_photos = list_files(&token, folder_id).await?;
                    let all_photos_id = Value::as_array(&all_photos["files"]).unwrap();
                    // println!("{}", all_photos);
                    
                    for j in all_photos_id.iter() {
                        let image_id = j["id"].as_str().unwrap();
                        let mime_type = j["mimeType"].as_str().unwrap();
                        let file_name = j["name"].as_str().unwrap();

                        download_image(file_name, &token, image_id, mime_type).await?;
                        // println!("{:?}", image_content_buffer);

                    }
                }
            }
    //     },
    // }
    
    Ok(())
}
