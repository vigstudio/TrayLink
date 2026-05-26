use std::net::{IpAddr, Ipv4Addr};

use rcgen::{CertificateParams, DnType, Ia5String, KeyPair, SanType};

pub struct TlsMaterials {
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
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
