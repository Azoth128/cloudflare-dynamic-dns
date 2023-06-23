mod errors;
mod public_ip;

use std::{env, thread::sleep, time::Duration};

use dotenv::dotenv;
use log::{error, info};
use public_ip::get_public_ip_address;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;

use crate::errors::Error;

fn main() {
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    dotenv().ok();
    let (token, zone_id, domain) = get_and_ensure_env_vars();

    loop {
        let result = update(&token, &zone_id, &domain);

        match result {
            Ok(_) => (),
            Err(err) => error!("{}", err.message),
        }
        sleep(Duration::from_secs(5 * 60));
    }
}

fn update(token: &str, zone_id: &str, domain: &str) -> Result<(), Error> {
    let (id, content) = get_id_and_content_of_dns(token, zone_id, domain)?;
    let current_ip_address = get_public_ip_address()?;

    if content == current_ip_address {
        info!("Not updating IP Address");
    } else {
        update_ip_address(token, zone_id, &current_ip_address, domain, &id)?;
    }

    Ok(())
}

fn get_id_and_content_of_dns(
    token: &str,
    zone_id: &str,
    domain: &str,
) -> Result<(String, String), Error> {
    let client = reqwest::blocking::Client::new();

    let resp = client
        .get(format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records"
        ))
        .header(AUTHORIZATION, format!("Bearer {token}",))
        .send()
        .map_err(|err| Error::new(err.to_string().as_str()))?;

    let content = resp
        .text()
        .map_err(|err| Error::new(err.to_string().as_str()))?;

    let json: Value =
        serde_json::from_str(&content).map_err(|err| Error::new(err.to_string().as_str()))?;
    let array = json["result"].clone();

    let array = array
        .as_array()
        .ok_or(Error::new("Could not parse array"))?;

    for value in array {
        if value["name"] == domain {
            return Ok((
                String::from(
                    value["id"]
                        .as_str()
                        .ok_or(Error::new("Could not parse id"))?,
                ),
                String::from(
                    value["content"]
                        .as_str()
                        .ok_or(Error::new("Could not parse content"))?,
                ),
            ));
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
) -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();
    client
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
        .map_err(|err| Error::new(err.to_string().as_str()))?;

    info!("Successfully updated IP Address");

    Ok(())
}

fn get_and_ensure_env_vars() -> (String, String, String) {
    let mut token: Option<String> = Option::None;
    let mut zone_id: Option<String> = Option::None;
    let mut domain: Option<String> = Option::None;

    env::vars().for_each(|(key, value)| match key.as_str() {
        "AUTH_BEARER" => token = Some(String::from(value)),
        "ZONE_ID" => zone_id = Some(String::from(value)),
        "DOMAIN" => domain = Some(String::from(value)),
        _ => {}
    });

    let token = token.expect("AUTH_BEARER not set");
    let zone_id = zone_id.expect("ZONE_ID not set");
    let domain = domain.expect("DOMAIN not set");

    (token, zone_id, domain)
}
