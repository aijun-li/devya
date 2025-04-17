use std::{path::Path, process::Command, vec};

use anyhow::anyhow;
use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use tokio::fs;

pub struct RootCert {
    pub cert: Certificate,
    pub key_pair: KeyPair,
}

pub struct SignedCert {
    pub cert: Certificate,
    pub key_pair: KeyPair,
}

const CERT_NAME: &str = "ca.crt";
const KEY_PAIR_NAME: &str = "ca.key";

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

        let key_path = dir_path.join(KEY_PAIR_NAME);
        let key_pem = fs::read_to_string(key_path).await.ok()?;
        let key_pair = KeyPair::from_pem(&key_pem).ok()?;

        let cert_path = dir_path.join(CERT_NAME);
        let cert_pem = fs::read_to_string(cert_path).await.ok()?;
        let cert_params = CertificateParams::from_ca_cert_pem(&cert_pem).ok()?;
        let cert = cert_params.self_signed(&key_pair).ok()?;

        Some(Self { cert, key_pair })
    }

    pub fn install<T>(dir_path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let cert_path = dir_path.as_ref().join(CERT_NAME).display().to_string();

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

    pub async fn save_to_dir<T>(&self, dir_path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let dir_path = dir_path.as_ref();

        fs::create_dir_all(dir_path).await?;

        let cert_path = dir_path.join(CERT_NAME);
        fs::write(cert_path, self.cert.pem()).await?;

        let key_path = dir_path.join(KEY_PAIR_NAME);
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

        anyhow::Ok(SignedCert { cert, key_pair })
    }
}
