use std::path::PathBuf;

use clap::Parser;
use shared::{CertificateError, SelfSignedCertificate};

#[derive(Parser)]
#[command(name = "gen_certs", about = "Generate self-signed certificates")]
struct Args {
    #[arg(short, long, env = "CERT_PATH", default_value = "./certs/cert.pem")]
    cert_path: PathBuf,
    #[arg(short, long, env = "KEY_PATH", default_value = "./certs/key.pem")]
    key_path: PathBuf,
    #[arg(short, long, default_value = "3650")]
    validity: i64,
}

fn main() -> Result<(), CertificateError> {
    shared::logging::init();
    let args = Args::parse();
    tracing::info!(
        validity_days = args.validity,
        "generating self-signed certificate"
    );
    let cert = SelfSignedCertificate::generate_with_validity(args.validity)?;
    smol::block_on(cert.save(&args.cert_path, &args.key_path))?;
    tracing::info!(cert = %args.cert_path.display(), key = %args.key_path.display(), "certificate saved");
    Ok(())
}
