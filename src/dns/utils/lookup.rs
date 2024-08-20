use std::time::Duration;

use anyhow::{bail, Result};
use essentials::debug;

use rustdns::types::*;
use rustdns::Message;
use std::net::UdpSocket;

pub async fn check_txt_value(domain: &str, value: &str, mut retries: usize) -> Result<()> {
    // Setup a UDP socket for sending to a DNS server.
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::new(5, 0)))?;
    socket.connect("8.8.8.8:53")?; // Google's Public DNS Servers

    let mut timeout = Duration::from_secs(1);
    loop {
        tokio::time::sleep(timeout).await;
        let records = lookup(&socket, domain, Type::TXT)?;
        if records.iter().any(|v| v == value) {
            return Ok(());
        }
        if retries == 0 {
            bail!(
                "Failed to find TXT record for domain={} value={}",
                domain,
                value
            );
        }
        retries -= 1;
        timeout *= 2;
    }
}

fn send_dns_message(socket: &UdpSocket, message: &Message) -> Result<Vec<Record>> {
    let question = message.to_vec()?;
    socket.send(&question)?;

    let mut resp = [0; 4096];
    let len = socket.recv(&mut resp)?;

    // Take the response bytes and turn it into another DNS Message.
    let answer = Message::from_slice(&resp[0..len])?;
    debug!(?answer, "Received response");
    Ok(answer.answers)
}

fn lookup(socket: &UdpSocket, domain: &str, rtype: Type) -> Result<Vec<String>> {
    debug!("lookup: domain={} rtype={:?}", domain, rtype);
    let mut message = Message::default();
    message.add_question(domain, Type::TXT, Class::Internet);
    debug!(?message, "Sending message");
    let answer = send_dns_message(socket, &message)?;
    Ok(answer
        .into_iter()
        .filter_map(|record| {
            if let Resource::TXT(txt) = record.resource {
                Some(txt)
            } else {
                None
            }
        })
        .flat_map(|txt| {
            txt.0
                .iter()
                .filter_map(|s| std::str::from_utf8(s).ok())
                .map(|s| s.to_string())
                .collect::<Box<[_]>>()
        })
        .collect())
}
