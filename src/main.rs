use log::{debug, error, info, LevelFilter, warn};
use std::{thread, time};
use reqwest::Client;
use simple_logger::SimpleLogger;

mod cloudflare_api;
mod config;
mod constants;

use crate::cloudflare_api::{CloudflareDnsRecord, CloudflareDnsResult, CloudflareResponse};
use crate::config::Config;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .env()
        .with_utc_timestamps()
        .with_colors(true)
        .init()
        .unwrap();
    debug!("Initialized logging.");

    debug!("Initializing configuration variables.");
    let config = Config::load();

    let general_config = config.general.unwrap();
    let wait_duration = general_config.wait_duration.unwrap();

    let cloudflare_config = config.cloudflare.unwrap();
    let cloudflare_zone_id = cloudflare_config.zone_id.unwrap();
    let cloudflare_api_token = cloudflare_config.api_token.unwrap();
    let cloudflare_dns_record_id = cloudflare_config.dns_record_id.unwrap();
    debug!("Configuration loaded.");

    debug!("Initializing reqwest client.");
    let client = reqwest::Client::new();

    let mut current_cloudflare_dns_record: Option<CloudflareResponse<CloudflareDnsResult>> = None;

    debug!("Starting main loop.");
    loop {
        debug!("Waiting {}s before next iteration.", wait_duration);
        thread::sleep(time::Duration::from_secs(wait_duration));

        debug!("Starting iteration.");

        if current_cloudflare_dns_record.is_none() {
            debug!("Getting the current Cloudflare DNS entry IP.");
            current_cloudflare_dns_record = get_current_cloudflare_dns_record(&client,
                cloudflare_zone_id.as_str(),
                cloudflare_api_token.as_str(),
                cloudflare_dns_record_id.as_str())
                .await;
        }

        // Get the current public IP.
        debug!("Getting the current public IP.");
        let current_public_ip = get_current_public_ip(&client)
            .await;

        if current_public_ip.is_none() || current_cloudflare_dns_record.is_none() {
            continue;
        }

        // If the IPs match, then skip this iteration.
        let current_public_ip_result = current_public_ip.unwrap();
        let current_cloudflare_dns_record_result = current_cloudflare_dns_record.as_ref().unwrap();

        debug!("Current public IP: {}", current_public_ip_result.trim());
        debug!("Current Cloudflare DNS IP: {}", current_cloudflare_dns_record_result.result.content.trim());

        if current_public_ip_result.trim() == current_cloudflare_dns_record_result.result.content.trim() {
            debug!("IP addresses are the same.");
            continue;
        }

        // If the IPs do not match, then update the new IP with Cloudflare.
        info!("IP changed from {} to {}. Updating with Cloudflare.",
            current_cloudflare_dns_record_result.result.content,
            current_public_ip_result);

        let new_dns_record = CloudflareDnsRecord {
            dns_type: current_cloudflare_dns_record_result.result.dns_type.clone(),
            name: current_cloudflare_dns_record_result.result.name.clone(),
            content: current_public_ip_result,
            ttl: current_cloudflare_dns_record_result.result.ttl,
            proxied: current_cloudflare_dns_record_result.result.proxied
        };

        current_cloudflare_dns_record = update_cloudflare_dns_record(&client,
            cloudflare_zone_id.as_str(),
            cloudflare_api_token.as_str(),
            cloudflare_dns_record_id.as_str(),
            &new_dns_record)
            .await;
    }
}

/// Gets the current public IP address.
async fn get_current_public_ip(client: &Client) -> Option<String> {
    let body = client.get("https://checkip.amazonaws.com")
        .send()
        .await;

    match body {
        Ok(r) => {
            match r.text().await {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Error deserializing current IP: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            warn!("Issue trying to get current IP: {:?}", e);
            None
        }
    }
}

/// Gets the current IP address set to the provided DNS record.
async fn get_current_cloudflare_dns_record(client: &Client, zone_id: &str, api_token: &str, dns_record_id: &str) -> Option<CloudflareResponse<CloudflareDnsResult>> {
    let body = client.get(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, dns_record_id))
        .bearer_auth(api_token)
        .send()
        .await;

    match body {
        Ok(r) => {
            match r.json::<CloudflareResponse<CloudflareDnsResult>>().await {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Error deserializing current Cloudflare DNS entry: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            warn!("Issue trying to get Cloudflare IP: {:?}", e);
            None
        }
    }
}

/// Updates the provided DNS record with Cloudflare.
async fn update_cloudflare_dns_record(client: &Client, zone_id: &str, api_token: &str, dns_record_id: &str, dns_record: &CloudflareDnsRecord) -> Option<CloudflareResponse<CloudflareDnsResult>> {
    let body = client.post(format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", zone_id, dns_record_id))
        .bearer_auth(api_token)
        .json(dns_record)
        .send()
        .await;

    match body {
        Ok(r) => {
            match r.json::<CloudflareResponse<CloudflareDnsResult>>().await {
                Ok(v) => {
                    if !v.success {
                        error!("Cloudflare update was not successful: {:?}", v);
                        None
                    } else {
                        info!("Cloudflare DNS record updated successfully.");
                        Some(v)
                    }
                },
                Err(e) => {
                    error!("Error deserializing current Cloudflare DNS update response: {:?}", e);
                    None
                }
            }
        },
        Err(e) => {
            error!("Cloudflare DNS did not update successfully: {:?}", e);
            None
        }
    }
}
