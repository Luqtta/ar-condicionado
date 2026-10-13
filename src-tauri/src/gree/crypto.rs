// Port of greeclimate/cipher.py (CipherV1 = AES-128-ECB, CipherV2 = AES-128-GCM).
use aes::Aes128;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use aes_gcm::{AeadInPlace, Aes128Gcm, Nonce, Tag};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const GCM_NONCE: [u8; 12] = [0x54, 0x40, 0x78, 0x44, 0x49, 0x67, 0x5a, 0x51, 0x6c, 0x5e, 0x63, 0x13];
const GCM_AAD: &[u8] = b"qualcomm-test";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    V1,
    V2,
}

impl Version {
    pub fn generic_key(self) -> &'static str {
        match self {
            Version::V1 => "a3K8Bx%2r8Y7#xDh",
            Version::V2 => "{yxAHAY_Lm6pbC/<",
        }
    }

    pub fn other(self) -> Version {
        match self {
            Version::V1 => Version::V2,
            Version::V2 => Version::V1,
        }
    }
}

fn key16(key: &str) -> Result<[u8; 16], String> {
    key.as_bytes().try_into().map_err(|_| format!("chave inválida (precisa ter 16 bytes): {key:?}"))
}

/// Returns (base64 pack, base64 tag). The tag only exists for V2.
pub fn encrypt(v: Version, key: &str, plain: &str) -> Result<(String, Option<String>), String> {
    let k = key16(key)?;
    let mut buf = plain.as_bytes().to_vec();
    match v {
        Version::V1 => {
            // PKCS#7, same as CipherV1.__pad
            let n = 16 - buf.len() % 16;
            buf.extend(std::iter::repeat_n(n as u8, n));
            let aes = Aes128::new(&k.into());
            for block in buf.chunks_mut(16) {
                aes.encrypt_block(GenericArray::from_mut_slice(block));
            }
            Ok((B64.encode(buf), None))
        }
        Version::V2 => {
            let tag = Aes128Gcm::new(&k.into())
                .encrypt_in_place_detached(Nonce::from_slice(&GCM_NONCE), GCM_AAD, &mut buf)
                .map_err(|_| "falha ao criptografar (GCM)".to_string())?;
            Ok((B64.encode(buf), Some(B64.encode(tag))))
        }
    }
}

pub fn decrypt(v: Version, key: &str, pack: &str, tag: Option<&str>) -> Result<Value, String> {
    let k = key16(key)?;
    let mut buf = B64.decode(pack).map_err(|e| format!("base64 inválido: {e}"))?;
    match v {
        Version::V1 => {
            if buf.is_empty() || buf.len() % 16 != 0 {
                return Err("pack ECB com tamanho inválido".into());
            }
            let aes = Aes128::new(&k.into());
            for block in buf.chunks_mut(16) {
                aes.decrypt_block(GenericArray::from_mut_slice(block));
            }
        }
        Version::V2 => {
            let tag = B64.decode(tag.ok_or("resposta GCM sem tag")?).map_err(|e| format!("tag inválida: {e}"))?;
            if tag.len() != 16 {
                return Err("tag GCM com tamanho inválido".into());
            }
            Aes128Gcm::new(&k.into())
                .decrypt_in_place_detached(Nonce::from_slice(&GCM_NONCE), GCM_AAD, &mut buf, Tag::from_slice(&tag))
                .map_err(|_| "falha ao descriptografar (GCM)".to_string())?;
        }
    }
    // Like greeclimate: drop everything after the last '}' (padding / junk).
    let end = buf.iter().rposition(|&b| b == b'}').ok_or("pack descriptografado não é JSON")? + 1;
    let text = std::str::from_utf8(&buf[..end]).map_err(|_| "pack descriptografado não é UTF-8")?;
    serde_json::from_str(text).map_err(|e| format!("JSON inválido no pack: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vectors generated independently with Node's crypto (aes-128-ecb / aes-128-gcm).
    const PLAIN: &str = r#"{"mac":"aabbccddeeff","t":"bind","uid":0}"#;
    const ECB: &str = "c8PTBZFeSHNh578FAb4WtmFccN4zX/O9gaCBjJM5EI+j9mrs/0p7GUTdWxmNaBUt";
    const GCM: &str = "JtoT1XUt89Ky557VOSoRU8ok7oUAzgWyIl52X7OFwurdNHgGiUNkepU=";
    const GCM_TAG: &str = "7OLw15QmZQQLFkRgI9whIA==";

    #[test]
    fn known_vectors() {
        let v1 = Version::V1.generic_key();
        let v2 = Version::V2.generic_key();
        assert_eq!(encrypt(Version::V1, v1, PLAIN).unwrap(), (ECB.into(), None));
        assert_eq!(encrypt(Version::V2, v2, PLAIN).unwrap(), (GCM.into(), Some(GCM_TAG.into())));
        let want: Value = serde_json::from_str(PLAIN).unwrap();
        assert_eq!(decrypt(Version::V1, v1, ECB, None).unwrap(), want);
        assert_eq!(decrypt(Version::V2, v2, GCM, Some(GCM_TAG)).unwrap(), want);
        // wrong cipher must fail, that's how v1/v2 get auto-detected
        assert!(decrypt(Version::V1, v1, GCM, None).is_err());
        assert!(decrypt(Version::V2, v2, ECB, Some(GCM_TAG)).is_err());
    }
}
