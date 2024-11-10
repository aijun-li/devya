use std::path::Path;

use tokio::fs;

const CA_NAME: &str = "Devya Mitm CA";

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
        format!("{}/root.crt", dir_path.as_ref().display()),
        cert.pem(),
    )
    .await
    .expect("Failed to write root.crt");
    fs::write(
        format!("{}/root.key", dir_path.as_ref().display()),
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
