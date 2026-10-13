//! Scan + bind + status against the real unit, printing everything as JSON.
//! cargo run --example gree_debug [mac]
use ar_condicionado_lib::gree::{device, discovery};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), String> {
    let want = std::env::args().nth(1);
    let found = discovery::scan(Duration::from_secs(3)).await?;
    println!("scan: {}", serde_json::to_string_pretty(&found).unwrap());
    let Some(f) = found
        .iter()
        .find(|f| want.as_ref().is_none_or(|m| m.eq_ignore_ascii_case(&f.mac)))
    else {
        return Err("nenhum aparelho encontrado".into());
    };
    let dev = device::bind(f.ip, &f.mac, f.version).await?;
    println!("bind: {}", serde_json::to_string_pretty(&dev).unwrap());
    let status = dev.status(device::STATUS_COLS).await?;
    println!("status: {}", serde_json::to_string_pretty(&status).unwrap());
    println!("room temp: {:?}", device::room_temp(&status, &mut None));
    Ok(())
}
