use reqwest;
use serde_json::{Map, Value};
use anyhow;
use std::process::{Command, Stdio};
use std::fs::File;

use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey, read_service_account_key};
use std::path::Path;
use reqwest::header::AUTHORIZATION;

static FOLDER_ID: &str = "0BzIGYtKj20XuZjFOekxHUWE2WlE";
static API_KEY: &str = "AIzaSyAWMfQ8ENz8f_88_vX-MlcdCdMM5h4X80E";
static EXCEL_MIME_TYPE: &str = "application/vnd.google-apps.spreadsheet";

/// Get the authenticator token for the service account
/// Necessary in order to establish the scope of the request
async fn get_auth_token() -> Result<String, anyhow::Error>{
    let service_account_key : ServiceAccountKey = read_service_account_key(Path::new("../../../systems-cs-pub-ro-497f6e6f3774.json")).await?;
    let authenticator = ServiceAccountAuthenticator::builder(service_account_key).build().await.expect("Failed to create authentication");
    let scopes = &["https://www.googleapis.com/auth/drive.readonly"];

    let result = authenticator.token(scopes).await?;
    match result.token(){
        Some(t) => Ok(t.to_owned()),
        None => Err(anyhow::anyhow!("No token found")),
    }
}

/// List all files inside a folder, identified by a folder_id
async fn list_files(auth_token: String, folder_id: &str) -> Result<Value, anyhow::Error>{
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
async fn convert_content(file_id: &str) -> Result<Value, anyhow::Error>{
    let endpoint = "https://www.googleapis.com";
    let url = format!("{endpoint}/drive/v3/files/{file_id}/export?key={API_KEY}&mimeType=text/csv");
    // println!("{}", url);

    let csv_file = reqwest::get(&url).await?.text().await?; // Fetch raw csv content

    let filename = "csv_text.csv";
    let file = File::create(filename)?;
    let _touchfile = Command::new("echo")       // Declare 'echo' command
        .arg(csv_file)                          // Pass text to echo
        .stdout(Stdio::from(file))              // Pass file to pipe output to
        .spawn()                                // Run the command
        .expect("failed to create new file");   

    let csv2json = Command::new("csv2json")     // Declare 'csv2json' command
        .arg("--in")                            // Pass input file
        .arg(filename)
        .output()                               // Pipe the output to current variable
        .expect("csv2json command failed to start");

    let json_text = String::from_utf8(csv2json.stdout)?;
    let json : Value = serde_json::from_str(json_text.as_str())?;
    Ok(json)
}

#[tokio::main]
async fn main() {
    let token = get_auth_token().await.unwrap();
    let folder_list = list_files(token, FOLDER_ID).await.unwrap();
    let all_files_id = Value::as_array(&folder_list["files"]).unwrap();

    let mut all_files_content = Map::<String, Value>::new();

    for i in all_files_id.iter(){
        let file_type = i["mimeType"].as_str().unwrap();

        if file_type == EXCEL_MIME_TYPE {
            let file_id = i["id"].as_str().unwrap();
            let file_content = convert_content(file_id).await.unwrap();
            let file_name = i["name"].as_str().unwrap();

            all_files_content.insert(file_name.to_owned(), file_content);
        }
    }

    let json_result = Value::Object(all_files_content);
    println!("{}", json_result);
}
