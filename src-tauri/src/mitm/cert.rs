use std::path::Path;

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair,
    KeyUsagePurpose,
};
use tokio::fs;

pub struct RootCert {
    pub cert: Certificate,
    pub key_pair: KeyPair,
}

impl RootCert {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let mut params = CertificateParams::default();

        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(DnType::CommonName, name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        anyhow::Ok(Self { cert, key_pair })
    }

    pub async fn read_from_dir<T>(dir_path: T) -> Option<Self>
    where
        T: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();

        let key_path = dir_path.join("ca.key");
        let key_pem = fs::read_to_string(key_path).await.ok()?;
        let key_pair = KeyPair::from_pem(&key_pem).ok()?;

        let cert_path = dir_path.join("ca.crt");
        let cert_pem = fs::read_to_string(cert_path).await.ok()?;
        let cert_params = CertificateParams::from_ca_cert_pem(&cert_pem).ok()?;
        let cert = cert_params.self_signed(&key_pair).ok()?;

        Some(Self { cert, key_pair })
    }

    pub async fn save_to_dir<T>(&self, dir_path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();

        fs::create_dir_all(dir_path).await?;

        let cert_path = dir_path.join("ca.crt");
        fs::write(cert_path, self.cert.pem()).await?;

        let key_path = dir_path.join("ca.key");
        fs::write(key_path, self.key_pair.serialize_pem()).await?;

        anyhow::Ok(())
    }
}
