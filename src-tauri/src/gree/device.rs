// Port of greeclimate/network.py (DeviceProtocol2) + device.py (bind/status/cmd).
use super::crypto::{self, Version};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, timeout_at};

pub const PORT: u16 = 7000;
const TIMEOUT: Duration = Duration::from_secs(2);
const RETRIES: usize = 2;

pub const STATUS_COLS: &[&str] = &[
    "Pow",
    "Mod",
    "SetTem",
    "TemUn",
    "TemSen",
    "WdSpd",
    "Tur",
    "Quiet",
    "SwUpDn",
    "SwingLfRig",
    "SwhSlp",
    "Blo",
    "Health",
    "Lig",
    "hid",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Device {
    pub ip: IpAddr,
    pub mac: String,
    pub key: String,
    pub version: Version,
}

pub fn is_timeout(e: &str) -> bool {
    e.starts_with("timeout")
}

/// Sends one UDP datagram to ip:7000 and waits for a reply from that ip.
/// 2s per attempt, 2 retries. A fresh socket per request so replies never cross.
pub async fn request(ip: IpAddr, packet: &Value) -> Result<Value, String> {
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;
    let data = packet.to_string();
    let mut buf = [0u8; 4096];
    for _ in 0..=RETRIES {
        // A send error (e.g. host unreachable after DHCP moved the unit) counts as a
        // silent attempt, so the caller still sees "timeout" and rescans by MAC.
        let _ = sock
            .send_to(data.as_bytes(), SocketAddr::new(ip, PORT))
            .await;
        let deadline = Instant::now() + TIMEOUT;
        while let Ok(r) = timeout_at(deadline, sock.recv_from(&mut buf)).await {
            // Windows surfaces ICMP "port unreachable" as a recv error; treat it like silence.
            let Ok((n, from)) = r else { continue };
            if from.ip() == ip
                && let Ok(v) = serde_json::from_slice::<Value>(&buf[..n])
            {
                return Ok(v);
            }
        }
    }
    Err(format!("timeout: {ip} não respondeu"))
}

/// Encrypts `pack`, wraps it like greeclimate's _generate_payload and returns the decrypted reply pack.
async fn send_pack(
    ip: IpAddr,
    mac: &str,
    v: Version,
    key: &str,
    bind: bool,
    mut pack: Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    pack.insert("mac".into(), json!(mac));
    let (enc, tag) = crypto::encrypt(v, key, &Value::Object(pack).to_string())?;
    let mut packet = json!({"cid": "app", "i": if bind { 1 } else { 0 }, "t": "pack", "uid": 0, "tcid": mac, "pack": enc});
    if let Some(tag) = tag {
        packet["tag"] = json!(tag);
    }
    let reply = request(ip, &packet).await?;
    let pack = reply["pack"].as_str().ok_or("resposta sem pack")?;
    match crypto::decrypt(v, key, pack, reply["tag"].as_str())? {
        Value::Object(m) => Ok(m),
        _ => Err("pack de resposta inesperado".into()),
    }
}

/// Bind with the generic key. Tries `hint` first (the cipher the scan reply used), then the other one.
pub async fn bind(ip: IpAddr, mac: &str, hint: Version) -> Result<Device, String> {
    let mut last = String::new();
    for v in [hint, hint.other()] {
        let pack = json!({"t": "bind", "uid": 0}).as_object().unwrap().clone();
        match send_pack(ip, mac, v, v.generic_key(), true, pack).await {
            Ok(r)
                if r.get("t")
                    .and_then(Value::as_str)
                    .is_some_and(|t| t.eq_ignore_ascii_case("bindok")) =>
            {
                let key = r
                    .get("key")
                    .and_then(Value::as_str)
                    .ok_or("bindok sem chave")?;
                return Ok(Device {
                    ip,
                    mac: mac.into(),
                    key: key.into(),
                    version: v,
                });
            }
            Ok(r) => last = format!("resposta inesperada ao bind: {}", Value::Object(r)),
            Err(e) => last = e,
        }
    }
    Err(last)
}

impl Device {
    /// {"cols":[...],"mac":..,"t":"status"} -> zip(cols, dat)
    pub async fn status(&self, cols: &[&str]) -> Result<Map<String, Value>, String> {
        let pack = json!({"t": "status", "cols": cols})
            .as_object()
            .unwrap()
            .clone();
        let r = send_pack(self.ip, &self.mac, self.version, &self.key, false, pack).await?;
        let (Some(Value::Array(c)), Some(Value::Array(d))) = (r.get("cols"), r.get("dat")) else {
            return Err(format!(
                "resposta de status inesperada: {}",
                Value::Object(r)
            ));
        };
        Ok(c.iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .zip(d.iter().cloned())
            .collect())
    }

    /// {"opt":[...],"p":[...],"t":"cmd"}
    pub async fn cmd(&self, props: &[(String, i64)]) -> Result<(), String> {
        let opt: Vec<&str> = props.iter().map(|(k, _)| k.as_str()).collect();
        let p: Vec<i64> = props.iter().map(|(_, v)| *v).collect();
        let pack = json!({"t": "cmd", "opt": opt, "p": p})
            .as_object()
            .unwrap()
            .clone();
        let r = send_pack(self.ip, &self.mac, self.version, &self.key, false, pack).await?;
        match r.get("r").and_then(Value::as_i64) {
            Some(200) | None => Ok(()),
            Some(code) => Err(format!("aparelho recusou o comando (r={code})")),
        }
    }
}

/// Firmware major version from hid, e.g. "362001000762+U-CS532AE(LT)V3.31.bin" -> 3.
fn hid_major(hid: &str) -> Option<u32> {
    let v = hid.strip_suffix(".bin")?.rsplit_once('V')?.1;
    if v.is_empty() || !v.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    v.split('.').next()?.parse().ok()
}

/// greeclimate's TemSen handling: firmware v4 (from hid, or when the first TemSen
/// reading of the session is below 40) reports °C as-is, older ones add 40.
/// `v4` caches the first-reading decision like greeclimate's check_version.
pub fn room_temp(status: &Map<String, Value>, v4: &mut Option<bool>) -> Option<i64> {
    let t = status.get("TemSen")?.as_i64()?;
    let hid_v4 = status
        .get("hid")
        .and_then(Value::as_str)
        .and_then(hid_major)
        == Some(4);
    let v4 = *v4.get_or_insert(t != 0 && t < 40) || hid_v4;
    match (v4, t) {
        (true, t) => Some(t),
        (false, 0) => None, // no sensor
        (false, t) => Some(t - 40),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn temsen_offset() {
        assert_eq!(hid_major("362001000762+U-CS532AE(LT)V3.31.bin"), Some(3));
        assert_eq!(hid_major("xxV4.0.bin"), Some(4));
        assert_eq!(hid_major("garbage"), None);

        let mut v4 = None;
        assert_eq!(room_temp(&st(json!({"TemSen": 65})), &mut v4), Some(25));
        assert_eq!(v4, Some(false));
        // issue 69: 40 must not flag v4
        let mut v4 = None;
        assert_eq!(room_temp(&st(json!({"TemSen": 40})), &mut v4), Some(0));
        // reading below 40 means no offset, and that sticks
        let mut v4 = None;
        assert_eq!(room_temp(&st(json!({"TemSen": 24})), &mut v4), Some(24));
        assert_eq!(room_temp(&st(json!({"TemSen": 41})), &mut v4), Some(41));
        // hid says v4
        let mut v4 = None;
        assert_eq!(
            room_temp(&st(json!({"TemSen": 50, "hid": "aV4.1.bin"})), &mut v4),
            Some(50)
        );
        let mut v4 = None;
        assert_eq!(room_temp(&st(json!({"TemSen": 0})), &mut v4), None);
    }
}
