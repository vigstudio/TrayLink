use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};

use rcgen::{CertificateParams, DnType, Ia5String, KeyPair, SanType};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

pub struct TlsMaterials {
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct TlsMeta {
    lan_ip: Option<String>,
}

fn tls_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("tls");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn load_or_create_tls_materials(
    app: &AppHandle,
    lan_ip: Option<&str>,
) -> Result<TlsMaterials, String> {
    let dir = tls_dir(app)?;
    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");
    let meta_path = dir.join("meta.json");

    if cert_path.is_file() && key_path.is_file() {
        let stored_ip: Option<String> = fs::read_to_string(&meta_path)
            .ok()
            .and_then(|raw| serde_json::from_str::<TlsMeta>(&raw).ok())
            .and_then(|meta| meta.lan_ip);
        if stored_ip.as_deref() == lan_ip {
            let cert_pem = fs::read(&cert_path).map_err(|e| e.to_string())?;
            let key_pem = fs::read(&key_path).map_err(|e| e.to_string())?;
            return Ok(TlsMaterials { cert_pem, key_pem });
        }
    }

    let materials = create_tls_materials(lan_ip)?;
    write_tls_files(&dir, &cert_path, &key_path, &meta_path, lan_ip, &materials)?;
    Ok(materials)
}

fn write_tls_files(
    dir: &Path,
    cert_path: &Path,
    key_path: &Path,
    meta_path: &Path,
    lan_ip: Option<&str>,
    materials: &TlsMaterials,
) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    fs::write(cert_path, &materials.cert_pem).map_err(|e| e.to_string())?;
    fs::write(key_path, &materials.key_pem).map_err(|e| e.to_string())?;
    let meta = TlsMeta {
        lan_ip: lan_ip.map(str::to_string),
    };
    fs::write(
        meta_path,
        serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_tls_materials(lan_ip: Option<&str>) -> Result<TlsMaterials, String> {
    let localhost = Ia5String::try_from("localhost").map_err(|e| e.to_string())?;
    let mut sans: Vec<SanType> = vec![
        SanType::DnsName(localhost),
        SanType::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST)),
    ];

    if let Some(ip) = lan_ip {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            sans.push(SanType::IpAddress(addr));
        }
    }

    let mut params = CertificateParams::default();
    params
        .distinguished_name
        .push(DnType::CommonName, "TrayLink");
    params.subject_alt_names = sans;

    let key_pair = KeyPair::generate().map_err(|e| e.to_string())?;
    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| e.to_string())?;

    Ok(TlsMaterials {
        cert_pem: cert.pem().into_bytes(),
        key_pem: key_pair.serialize_pem().into_bytes(),
    })
}
