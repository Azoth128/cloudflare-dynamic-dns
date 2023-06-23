use regex::Regex;
use reqwest::blocking::Client;

use crate::errors::Error;

pub fn get_public_ip_address() -> Result<String, Error> {
    //see https://wiki.ubuntuusers.de/FritzBox/Skripte/

    let resp = Client::new()
        .post("http://fritz.box:49000/igdupnp/control/WANIPConn1")
        .header("Content-Type", "text/xml; charset=utf-8")
        .header(
            "SoapAction",
            "urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress",
        )
        .body("<?xml version='1.0' encoding='utf-8'?> <s:Envelope s:encodingStyle='http://schemas.xmlsoap.org/soap/encoding/' xmlns:s='http://schemas.xmlsoap.org/soap/envelope/'> <s:Body> <u:GetExternalIPAddress xmlns:u='urn:schemas-upnp-org:service:WANIPConnection:1' /> </s:Body> </s:Envelope>")
        .send()
        .map_err(|_| Error::new("Error fetching result via Http"))?;

    if resp.status() != 200 {
        return Err(Error::new(&format!("Status: {}", resp.status())));
    }

    let xml = resp.text().map_err(|_| Error::new("No data"))?;

    let regex = Regex::new(r"(?:[0-9]{1,3}\.){3}[0-9]{1,3}")
        .map_err(|_| Error::new("Regex parsing error"))?;

    let text = regex
        .captures(&xml)
        .and_then(|cap| cap.get(0))
        .map(|cap| cap.as_str())
        .map(String::from)
        .ok_or(Error::new("Regex no match"));

    text
}
