use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::Arc,
};

use essentials::{debug, info};
use ssl_manager::cloudflare::Cloudflare;

pub async fn run() {
    let cf_service_token = env::var("CF_SERVICE_TOKEN").unwrap();
    debug!("CF_SERVICE_TOKEN: {}", cf_service_token);
    let domain = "appka6.majksa.net".to_owned();
    let out_dir = env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./out/certs"));
    let out_dir = out_dir.join(&domain);
    fs::create_dir_all(&out_dir).unwrap();
    let csr_file = out_dir.join("csr.der");
    info!("Requesting certificate...");
    let cf_client = Arc::new(Cloudflare::new(cf_service_token).unwrap());
    let csr_der = if csr_file.is_file() {
        Some(fs::read(&csr_file).unwrap())
    } else {
        None
    };
    let certificate = cf_client
        .create_certificate(domain, csr_der.as_deref())
        .await
        .unwrap();
    let cert_file = out_dir.join("cert.pem");
    use ssl_manager::CertificateResult::*;
    match certificate {
        Refreshed(certificate) => {
            File::create(&cert_file)
                .unwrap()
                .write_all(certificate.as_bytes())
                .unwrap();
        }
        New(certificate) => {
            let key_file = out_dir.join("key.pem");
            File::create(&cert_file)
                .unwrap()
                .write_all(certificate.certificate.as_bytes())
                .unwrap();
            File::create(&key_file)
                .unwrap()
                .write_all(certificate.private_key.as_bytes())
                .unwrap();
            File::create(&csr_file)
                .unwrap()
                .write_all(&certificate.csr_der)
                .unwrap();
        }
    };
}
