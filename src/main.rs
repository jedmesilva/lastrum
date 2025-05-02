use clap::{Parser, Subcommand};
use log::{error, info};
use simple_logger::SimpleLogger;
use std::process;

use lastrum_certifield::{
    certifield::{
        builder::CertificateBuilder,
        model::{Asset, AssetType, Certificate},
        storage::CertificateStorage,
    },
    core::{
        identity::Identity,
        keypair::KeyPair,
        signer::Signer,
        validator::Validator,
    },
    errors::LastrumError,
};

#[derive(Parser)]
#[command(name = "Lastrum Certifield")]
#[command(author = "Lastrum Team")]
#[command(version = "0.1.0")]
#[command(about = "A decentralized system for custody houses to issue and validate certificates for physical assets", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new identity for a custody house
    GenerateIdentity {
        /// Name of the custody house
        #[arg(short, long)]
        name: String,
    },
    /// Issue a new certificate for an asset
    IssueCertificate {
        /// The identity file path of the issuer
        #[arg(short, long)]
        identity: String,
        /// Asset type (GOLD, SILVER, etc.)
        #[arg(short, long)]
        asset_type: String,
        /// Asset weight in grams
        #[arg(short, long)]
        weight: f64,
        /// Asset purity (0.0-1.0)
        #[arg(short, long)]
        purity: f64,
        /// Serial number of the asset
        #[arg(short, long)]
        serial: String,
    },
    /// Verify a certificate
    VerifyCertificate {
        /// Path to the certificate file
        #[arg(short, long)]
        certificate: String,
    },
    /// List all certificates in storage
    ListCertificates,
}

fn main() {
    // Initialize the logger
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), LastrumError> {
    match cli.command {
        Commands::GenerateIdentity { name } => {
            info!("Generating new identity for custody house: {}", name);
            let identity = Identity::new(name)?;
            let path = identity.save()?;
            info!("Identity saved to: {}", path);
            info!("Public key: {}", identity.keypair().public_key_hex());
            info!("Node hash: {}", identity.node_hash());
            Ok(())
        }
        Commands::IssueCertificate { identity, asset_type, weight, purity, serial } => {
            info!("Loading identity from: {}", identity);
            let identity = Identity::load(&identity)?;
            
            let asset_type = match asset_type.to_uppercase().as_str() {
                "GOLD" => AssetType::Gold,
                "SILVER" => AssetType::Silver,
                "PLATINUM" => AssetType::Platinum,
                "PALLADIUM" => AssetType::Palladium,
                _ => return Err(LastrumError::InvalidAssetType(asset_type)),
            };
            
            let asset = Asset::new(asset_type, weight, purity, serial);
            
            let certificate = CertificateBuilder::new()
                .with_issuer(identity.name().to_string())
                .with_issuer_public_key(identity.keypair().public_key_hex())
                .with_asset(asset)
                .build()?;
            
            // Sign the certificate
            let signer = Signer::new(identity.keypair().clone());
            let signed_certificate = signer.sign_certificate(certificate)?;
            
            // Save the certificate
            let storage = CertificateStorage::new()?;
            let certificate_id = storage.store(&signed_certificate)?;
            
            info!("Certificate issued with ID: {}", certificate_id);
            info!("Asset type: {}", signed_certificate.asset.asset_type_str());
            info!("Weight: {}g, Purity: {}", signed_certificate.asset.weight, signed_certificate.asset.purity);
            info!("Serial: {}", signed_certificate.asset.serial);
            
            Ok(())
        }
        Commands::VerifyCertificate { certificate } => {
            let storage = CertificateStorage::new()?;
            let cert = storage.load_by_id(&certificate)?;
            
            info!("Verifying certificate: {}", cert.id);
            
            let validator = Validator::new();
            match validator.verify_certificate(&cert) {
                Ok(true) => {
                    info!("✅ Certificate is valid!");
                    info!("Issuer: {}", cert.issuer);
                    info!("Asset: {} - {}g, purity: {}", cert.asset.asset_type_str(), cert.asset.weight, cert.asset.purity);
                    info!("Serial: {}", cert.asset.serial);
                    info!("Issued at: {}", cert.issued_at);
                }
                Ok(false) => {
                    info!("❌ Certificate is invalid! Signature doesn't match the data.");
                }
                Err(e) => {
                    return Err(e);
                }
            }
            
            Ok(())
        }
        Commands::ListCertificates => {
            let storage = CertificateStorage::new()?;
            let certificates = storage.list_all()?;
            
            if certificates.is_empty() {
                info!("No certificates found in storage.");
                return Ok(());
            }
            
            info!("Found {} certificates:", certificates.len());
            for (i, cert) in certificates.iter().enumerate() {
                info!("{}. ID: {}", i + 1, cert.id);
                info!("   Issuer: {}", cert.issuer);
                info!("   Asset: {} - {}g, purity: {}", cert.asset.asset_type_str(), cert.asset.weight, cert.asset.purity);
                info!("   Serial: {}", cert.asset.serial);
                info!("   Issued at: {}", cert.issued_at);
                info!("   ----------------------");
            }
            
            Ok(())
        }
    }
}
