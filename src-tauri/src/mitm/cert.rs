use std::{path::Path, process::Command, vec};

use anyhow::anyhow;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use time::{Duration, OffsetDateTime};
use tokio::fs;
use tracing::{debug, info};

pub struct RootCA {
    pub cert: Certificate,
    pub key_pair: KeyPair,
}

#[derive(Clone)]
pub struct SignedCert {
    pub cert: Vec<u8>,
    pub key_pair: Vec<u8>,
}

impl RootCA {
    pub fn new(name: &str, days_until_expiry: i64) -> anyhow::Result<Self> {
        let mut params = CertificateParams::default();

        params.distinguished_name = DistinguishedName::new();
        params.distinguished_name.push(DnType::CommonName, name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        params.not_after = OffsetDateTime::now_utc() + Duration::days(days_until_expiry);

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        anyhow::Ok(Self { cert, key_pair })
    }

    pub async fn read_from_file<T>(cert_path: T, key_path: T) -> Option<Self>
    where
        T: AsRef<Path>,
    {
        let key_pem = fs::read_to_string(key_path).await.ok()?;
        let key_pair = KeyPair::from_pem(&key_pem).ok()?;

        let cert_pem = fs::read_to_string(cert_path).await.ok()?;
        let cert_params = CertificateParams::from_ca_cert_pem(&cert_pem).ok()?;
        let cert = cert_params.self_signed(&key_pair).ok()?;

        Some(Self { cert, key_pair })
    }

    pub fn install<T>(cert_path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let cert_path = cert_path.as_ref().display().to_string();

        if cfg!(target_os = "macos") {
            let default_keychain = String::from_utf8_lossy(
                &Command::new("security")
                    .arg("default-keychain")
                    .output()?
                    .stdout,
            )
            .trim()
            .to_string()
            .replace(r#"""#, "");

            if Command::new("security")
                .args(["add-trusted-cert", "-k", &default_keychain, &cert_path])
                .output()
                .map_err(|_| anyhow!("Failed to install cert"))?
                .status
                .success()
            {
                Ok(())
            } else {
                Err(anyhow!("Failed to install cert"))
            }
        } else if cfg!(target_os = "windows") {
            if Command::new("certutil")
                .arg("-addstore")
                .arg("-user")
                .arg("Root")
                .arg(cert_path)
                .output()
                .map_err(|_| anyhow!("Failed to install cert"))?
                .status
                .success()
            {
                Ok(())
            } else {
                Err(anyhow!("Failed to install cert"))
            }
        } else {
            Err(anyhow!("Unsupported platform"))
        }
    }

    pub fn check_installed<T>(cert_path: T) -> anyhow::Result<bool>
    where
        T: AsRef<Path>,
    {
        let cert_path = cert_path.as_ref().display().to_string();
        if cfg!(target_os = "macos") {
            let default_keychain = String::from_utf8_lossy(
                &Command::new("security")
                    .arg("default-keychain")
                    .output()?
                    .stdout,
            )
            .trim()
            .to_string()
            .replace(r#"""#, "");

            let output = Command::new("security")
                .args(["verify-cert", "-k", &default_keychain, "-c", &cert_path])
                .output()
                .map_err(|_| anyhow!("Failed to install cert"))?;

            debug!("verify cert output {:?}", output);

            Ok(output.status.success())
        } else if cfg!(target_os = "windows") {
            todo!("Windows not supported yet")
        } else {
            Err(anyhow!("Unsupported platform"))
        }
    }

    pub async fn save_to_file<T>(&self, cert_path: T, key_path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let cert_path = cert_path.as_ref();
        if let Some(parent_dir) = cert_path.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).await?;
            }
        }
        fs::write(cert_path, self.cert.pem()).await?;

        let key_path = key_path.as_ref();
        if let Some(parent_dir) = key_path.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(parent_dir).await?;
            }
        }
        fs::write(key_path, self.key_pair.serialize_pem()).await?;

        anyhow::Ok(())
    }

    pub fn sign(&self, host: &str) -> anyhow::Result<SignedCert> {
        let mut params = CertificateParams::new(vec![host.to_string()])?;

        // 设置证书参数
        params.distinguished_name.push(DnType::CommonName, host);
        params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
        params.is_ca = IsCa::NoCa;

        // 生成新密钥对
        let key_pair = KeyPair::generate()?;

        // 用根证书签发
        let cert = params.signed_by(&key_pair, &self.cert, &self.key_pair)?;

        anyhow::Ok(SignedCert {
            cert: cert.der().to_vec(),
            key_pair: key_pair.serialize_der(),
        })
    }
}
