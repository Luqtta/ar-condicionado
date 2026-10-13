// Port of greeclimate/discovery.py.
use super::crypto::{self, Version};
use super::device::PORT;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, timeout_at};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Found {
    pub ip: IpAddr,
    pub mac: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub ver: Option<String>,
    /// Cipher that decrypted the scan reply, tried first when binding.
    pub version: Version,
}

/// 255.255.255.255 plus each IPv4 interface's directed broadcast (Windows often
/// sends the global broadcast out of the wrong adapter).
fn broadcast_addrs() -> Vec<Ipv4Addr> {
    let mut out = vec![Ipv4Addr::BROADCAST];
    for iface in if_addrs::get_if_addrs().unwrap_or_default() {
        if let if_addrs::IfAddr::V4(a) = iface.addr {
            if a.ip.is_loopback() || a.netmask.to_bits().count_ones() >= 31 {
                continue;
            }
            let b = a.broadcast.unwrap_or(Ipv4Addr::from_bits(a.ip.to_bits() | !a.netmask.to_bits()));
            if !out.contains(&b) {
                out.push(b);
            }
        }
    }
    out
}

/// Decrypts the reply with the generic V1 key, falling back to V2.
fn parse_reply(data: &[u8], ip: IpAddr) -> Option<Found> {
    let reply: Value = serde_json::from_slice(data).ok()?;
    let pack = reply["pack"].as_str()?;
    let tag = reply["tag"].as_str();
    let (pack, version) = [Version::V1, Version::V2]
        .into_iter()
        .find_map(|v| crypto::decrypt(v, v.generic_key(), pack, tag).ok().map(|p| (p, v)))?;
    let s = |k: &str| pack[k].as_str().filter(|s| !s.is_empty()).map(String::from);
    let mac = s("mac").or_else(|| s("cid"))?;
    Some(Found { ip, name: s("name").unwrap_or_else(|| mac.replace(':', "")), mac, brand: s("brand"), model: s("model"), ver: s("ver"), version })
}

/// Broadcasts {"t":"scan"} and collects replies for `wait`.
pub async fn scan(wait: Duration) -> Result<Vec<Found>, String> {
    let sock = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| e.to_string())?;
    sock.set_broadcast(true).map_err(|e| e.to_string())?;
    let mut sent = false;
    for b in broadcast_addrs() {
        sent |= sock.send_to(br#"{"t":"scan"}"#, SocketAddr::new(b.into(), PORT)).await.is_ok();
    }
    if !sent {
        return Err("não foi possível enviar o broadcast de busca".into());
    }
    let deadline = Instant::now() + wait;
    let mut found: Vec<Found> = Vec::new();
    let mut buf = [0u8; 4096];
    while let Ok(r) = timeout_at(deadline, sock.recv_from(&mut buf)).await {
        let Ok((n, from)) = r else { continue };
        if let Some(f) = parse_reply(&buf[..n], from.ip()) {
            if !found.iter().any(|x| x.mac == f.mac) {
                found.push(f);
            }
        }
    }
    Ok(found)
}
