use std::{path::Path, process::Command};

use tokio::{fs, join};

const CA_NAME: &str = "Devya Mitm CA";

const CERT_NAME: &str = "root.crt";
const KEY_NAME: &str = "root.key";

pub async fn detect_cert<T: AsRef<Path>>(dir_path: &T) -> bool {
    let cert_pair = (
        dir_path.as_ref().join(CERT_NAME),
        dir_path.as_ref().join(KEY_NAME),
    );
    match join!(fs::metadata(&cert_pair.0), fs::metadata(&cert_pair.1)) {
        (Ok(crt_meta), Ok(key_meta)) => {
            if crt_meta.is_file() && key_meta.is_file() {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

// TODO: make it optional
pub async fn make_root_cert<T: AsRef<Path>>(dir_path: &T) -> rcgen::CertifiedKey {
    let mut param = rcgen::CertificateParams::default();

    param.distinguished_name = rcgen::DistinguishedName::new();
    param.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String(CA_NAME.to_string()),
    );
    param.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];
    param.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    let key_pair = rcgen::KeyPair::generate().unwrap();
    let cert = param.self_signed(&key_pair).unwrap();

    fs::create_dir_all(dir_path.as_ref())
        .await
        .expect("Failed to create certs dir");
    fs::write(
        format!("{}/{}", dir_path.as_ref().display(), CERT_NAME),
        cert.pem(),
    )
    .await
    .expect("Failed to write root.crt");
    fs::write(
        format!("{}/{}", dir_path.as_ref().display(), KEY_NAME),
        key_pair.serialize_pem(),
    )
    .await
    .expect("Failed to write root.key");

    rcgen::CertifiedKey { cert, key_pair }
}

// TODO: make it optional
pub fn read_root_cert<T: AsRef<Path>>(cert_pair: &(T, T)) -> rcgen::CertifiedKey {
    let param =
        rcgen::CertificateParams::from_ca_cert_pem(&std::fs::read_to_string(&cert_pair.0).unwrap())
            .unwrap();
    let key_pair =
        rcgen::KeyPair::from_pem(&std::fs::read_to_string(&cert_pair.1).unwrap()).unwrap();
    let cert = param.self_signed(&key_pair).unwrap();

    rcgen::CertifiedKey { cert, key_pair }
}

pub async fn install_cert<T: AsRef<Path>>(dir_path: &T) {
    if !detect_cert(dir_path).await {
        make_root_cert(dir_path).await;
    }

    if cfg!(target_os = "macos") {
        let default_keychain = String::from_utf8_lossy(
            &Command::new("security")
                .arg("default-keychain")
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string()
        .replace(r#"""#, "");

        let cert_path = dir_path.as_ref().join(CERT_NAME).display().to_string();

        Command::new("security")
            .arg("add-trusted-cert")
            .arg("-k")
            .arg(default_keychain)
            .arg(cert_path)
            .spawn()
            .unwrap();
    }
}
