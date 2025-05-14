// Jackson Coxson

use futures_util::{StreamExt, pin_mut};
use log::{debug, warn};
use mdns::{Record, RecordKind};
use std::{net::IpAddr, time::Duration};
use tokio::sync::mpsc::UnboundedSender;

use crate::IdeviceCommands;

const SERVICE_NAME: &str = "apple-mobdev2";
const SERVICE_PROTOCOL: &str = "tcp";

pub async fn start_discover(sender: UnboundedSender<IdeviceCommands>) {
    let service_name = format!("_{}._{}.local", SERVICE_NAME, SERVICE_PROTOCOL);
    println!("Starting mDNS discovery for {} with mdns", service_name);

    let stream = mdns::discover::all(&service_name, Duration::from_secs(1))
        .expect("Unable to start mDNS discover stream")
        .listen();
    pin_mut!(stream);

    while let Some(Ok(response)) = stream.next().await {
        let addr = response.records().filter_map(self::to_ip_addr).next();

        if let Some(mut addr) = addr {
            let mut mac_addr = None;
            for i in response.records() {
                if let RecordKind::A(addr4) = i.kind {
                    addr = std::net::IpAddr::V4(addr4)
                }
                if i.name.contains(&service_name) && i.name.contains('@') {
                    mac_addr = Some(i.name.split('@').collect::<Vec<&str>>()[0]);
                }
            }

            // Look through paired devices for mac address
            let mac_addr = match mac_addr {
                Some(m) => m,
                None => {
                    warn!("Unable to get mac address for mDNS record");
                    continue;
                }
            };

            debug!("Discovered {mac_addr} at {addr}");
            sender
                .send(IdeviceCommands::DiscoveredDevice((
                    addr,
                    mac_addr.to_string(),
                )))
                .unwrap();
        }
    }
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
