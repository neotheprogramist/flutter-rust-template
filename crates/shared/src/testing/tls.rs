use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Datelike;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use smol::fs;

pub const ALPN_QUIC: &[&[u8]] = &[b"hq-29"];

#[derive(Debug, thiserror::Error)]
pub enum CertificateError {
    #[error(transparent)]
    Generation(#[from] rcgen::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PemParse(#[from] pem::PemError),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Debug)]
pub struct SelfSignedCertificate {
    cert: Certificate,
    key: KeyPair,
}

impl SelfSignedCertificate {
    pub fn generate_with_validity(days: i64) -> Result<Self, CertificateError> {
        let key = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "localhost");
        params.distinguished_name = dn;
        params.subject_alt_names = vec![
            SanType::DnsName("localhost".try_into()?),
            SanType::IpAddress(std::net::Ipv4Addr::LOCALHOST.into()),
            SanType::IpAddress(std::net::Ipv6Addr::LOCALHOST.into()),
        ];
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::days(days);
        params.not_before = rcgen::date_time_ymd(now.year(), now.month() as u8, now.day() as u8);
        params.not_after = rcgen::date_time_ymd(exp.year(), exp.month() as u8, exp.day() as u8);
        Ok(Self {
            cert: params.self_signed(&key)?,
            key,
        })
    }

    #[must_use]
    pub fn cert_der(&self) -> Vec<u8> {
        self.cert.der().to_vec()
    }
    #[must_use]
    pub fn key_der(&self) -> Vec<u8> {
        self.key.serialize_der()
    }
    #[must_use]
    pub fn cert_pem(&self) -> String {
        pem::encode(&pem::Pem::new("CERTIFICATE", self.cert_der()))
    }
    #[must_use]
    pub fn key_pem(&self) -> String {
        pem::encode(&pem::Pem::new("PRIVATE KEY", self.key_der()))
    }
    #[must_use]
    pub fn to_rustls(&self) -> (PrivateKeyDer<'static>, CertificateDer<'static>) {
        (
            PrivateKeyDer::Pkcs8(self.key_der().into()),
            CertificateDer::from(self.cert_der()),
        )
    }

    pub async fn save<P: AsRef<Path>>(&self, cert: P, key: P) -> Result<(), CertificateError> {
        for p in [&cert, &key] {
            if let Some(d) = p.as_ref().parent().filter(|d| !d.as_os_str().is_empty()) {
                fs::create_dir_all(d).await?;
            }
        }
        fs::write(cert.as_ref(), self.cert_pem()).await?;
        fs::write(key.as_ref(), self.key_pem()).await?;
        Ok(())
    }
}

pub fn decode_b64_pem(b64: &str) -> Result<String, CertificateError> {
    Ok(String::from_utf8(STANDARD.decode(b64)?)?)
}

pub fn parse_rustls_from_pem(
    cert: &str,
    key: &str,
) -> Result<(PrivateKeyDer<'static>, CertificateDer<'static>), CertificateError> {
    Ok((
        PrivateKeyDer::Pkcs8(pem::parse(key)?.contents().to_vec().into()),
        CertificateDer::from(pem::parse(cert)?.contents().to_vec()),
    ))
}
