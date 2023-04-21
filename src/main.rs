mod errors;
mod public_ip;

#[macro_use]
extern crate dotenv_codegen;

use log::{error, info};
use public_ip::get_public_ip_address;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

fn main() {
    let token = dotenv!("AUTH_BEARER");
    let zone_id = dotenv!("ZONE_ID");
    let domain = dotenv!("DOMAIN");

    let (id, content) = get_id_and_content_of_dns(token, zone_id, domain);
    match get_public_ip_address() {
        Ok(current_ip_address) => {
            if content != current_ip_address {
                update_ip_address(token, zone_id, &current_ip_address, domain, &id);
            } else {
                info!("Not updating IP Address");
            }
        }
        Err(err) => error!("{}", err.message),
    }
}

fn get_id_and_content_of_dns(token: &str, zone_id: &str, domain: &str) -> (String, String) {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records"
        ))
        .header(AUTHORIZATION, format!("Bearer {token}",))
        .send()
        .unwrap();

    let content = resp.text().unwrap();

    let json: Value = serde_json::from_str(&content).unwrap();
    let array = json["result"].clone();

    let array = array.as_array().unwrap();
    for value in array {
        if value["name"] == domain {
            return (
                String::from(value["id"].as_str().unwrap()),
                String::from(value["content"].as_str().unwrap()),
            );
        }
    }

    panic!();
}

fn update_ip_address(
    api_token: &str,
    zone_id: &str,
    current_ip_address: &str,
    domain: &str,
    dns_id: &str,
) {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .put(format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{dns_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {api_token}",))
        .header(CONTENT_TYPE, "application/json")
        .body(format!(
            "{{
            \"content\": \"{current_ip_address}\",
            \"name\": \"{domain}\",
            \"type\": \"A\"
          }}"
        ))
        .send()
        .unwrap();

    println!("{}", resp.text().unwrap());

    // if (resp.status()) != 200 {
    //     panic!()
    // }
}
