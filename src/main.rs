use clap::{Parser, Subcommand};
use log::{error, info};
use simple_logger::SimpleLogger;
use std::process;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    network::{
        peer::{Peer, PeerManager},
        p2p::{P2PService, P2PConfig},
        consensus::ConsensusManager,
        sync::LedgerSyncManager,
        protocol::{Message, MessageType}
    },
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
        #[arg(short, long, default_value = "0.0")]
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
    /// Inicia um nó na rede Lastrum (modo servidor)
    StartNode {
        /// Caminho para o arquivo de identidade a ser usado pelo nó
        #[arg(short, long)]
        identity: String,
        /// Porta para receber conexões (padrão: 8080)
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Lista de nós iniciais para se conectar (separados por vírgula)
        #[arg(short, long, default_value = "")]
        bootstrap: String,
    },
    /// Adiciona um peer à lista de peers conhecidos
    AddPeer {
        /// Endereço do peer (formato: host:porta)
        #[arg(short, long)]
        address: String,
    },
    /// Lista todos os peers conhecidos
    ListPeers,
    /// Broadcast um certificado para a rede
    BroadcastCertificate {
        /// ID do certificado a ser propagado
        #[arg(short, long)]
        certificate: String,
    },
    /// Sincroniza certificados com a rede
    SyncCertificates,
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
            info!("Registered at: {}", identity.registered_at());
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
                "BIOGAS_METANO" => AssetType::BiogasMetano,
                "ENERGIA_SOLAR" => AssetType::EnergiaSolar,
                "ENERGIA_EOLICA" => AssetType::EnergiaEolica,
                "ENERGIA_HIDRICA" => AssetType::EnergiaHidrica,
                "HIDROGENIO_VERDE" => AssetType::HidrogenioVerde,
                _ => return Err(LastrumError::InvalidAssetType(asset_type)),
            };
            
            // Determina a unidade apropriada baseada no tipo de ativo
            let unit = match asset_type {
                AssetType::Gold | AssetType::Silver | AssetType::Platinum | AssetType::Palladium => "g".to_string(),
                AssetType::BiogasMetano => "m3".to_string(),
                AssetType::EnergiaSolar | AssetType::EnergiaEolica | AssetType::EnergiaHidrica => "kWh".to_string(),
                AssetType::HidrogenioVerde => "kg".to_string(),
                _ => "unit".to_string(),
            };
            
            // Cria um asset baseado no tipo
            let unit_str = unit.clone(); // Clona a string para uso posterior
            let asset = match asset_type {
                AssetType::Gold | AssetType::Silver | AssetType::Platinum | AssetType::Palladium => {
                    Asset::new_with_purity(asset_type, weight, unit.clone(), purity, serial)
                },
                _ => Asset::new(asset_type, weight, unit, serial),
            };
            
            // Descrição padrão baseada no tipo de ativo
            let description = format!("Certificado de {} {} de {} sob custódia.", 
                weight, 
                unit_str,
                asset_type.as_str().to_lowercase());
            
            let certificate = CertificateBuilder::new()
                .with_custody_house_id(identity.name().to_string())
                .with_custody_house_hash(identity.keypair().public_key_hex())
                .with_asset(asset)
                .with_description(description)
                .build()?;
            
            // Sign the certificate
            let signer = Signer::new(identity.keypair().clone());
            let signed_certificate = signer.sign_certificate(certificate)?;
            
            // Save the certificate
            let storage = CertificateStorage::new()?;
            let certificate_id = storage.store(&signed_certificate)?;
            
            info!("Certificate issued with ID: {}", certificate_id);
            info!("Asset type: {}", signed_certificate.asset.asset_type_str());
            
            // Exibe a quantidade e a unidade
            info!("Quantity: {} {}", signed_certificate.asset.quantity, signed_certificate.asset.unit);
            
            // Exibe a pureza se disponível
            if let Some(purity) = signed_certificate.asset.purity {
                info!("Purity: {}", purity);
            }
            
            info!("Serial: {}", signed_certificate.asset.serial);
            info!("Description: {}", signed_certificate.description);
            
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
                    info!("Custody House: {}", cert.custody_house_id);
                    
                    // Exibe a quantidade e a unidade
                    info!("Asset: {} - {} {}", 
                        cert.asset.asset_type_str(), 
                        cert.asset.quantity, 
                        cert.asset.unit);
                    
                    // Exibe a pureza se disponível
                    if let Some(purity) = cert.asset.purity {
                        info!("Purity: {}", purity);
                    }
                    
                    info!("Serial: {}", cert.asset.serial);
                    info!("Description: {}", cert.description);
                    info!("Issued at: {}", cert.issued_at);
                    
                    // Exibe a data de expiração se disponível
                    if let Some(expires_at) = cert.expires_at {
                        info!("Expires at: {}", expires_at);
                        if cert.is_expired() {
                            info!("⚠️ Certificate is expired!");
                        }
                    }
                    
                    // Exibe informações sobre as carteiras autorizadas
                    if let Some(wallets) = &cert.authorized_wallets {
                        info!("Authorized wallets: {}", wallets.len());
                        for (i, wallet) in wallets.iter().enumerate() {
                            info!("  {}. {}", i+1, wallet);
                        }
                    } else {
                        info!("No specific wallets authorized (any wallet with the certificate private key can issue tokens)");
                    }
                    
                    // Exibe informações do ledger de tokens se houver
                    if !cert.token_ledgers.is_empty() {
                        info!("Token ledger entries: {}", cert.token_ledgers.len());
                        for (i, entry) in cert.token_ledgers.iter().enumerate() {
                            info!("  {}. Token: {}", i+1, entry.token_hash);
                            info!("     Issued by: {}", entry.issuer_wallet);
                            info!("     Issued at: {}", entry.issued_at);
                            if let Some(consumed_at) = entry.consumed_at {
                                info!("     Consumed at: {}", consumed_at);
                                if let Some(consumer) = &entry.consumer_wallet {
                                    info!("     Consumed by: {}", consumer);
                                }
                            }
                        }
                    } else {
                        info!("No tokens issued yet for this certificate");
                    }
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
                info!("   Custody House: {}", cert.custody_house_id);
                
                // Exibe a quantidade e a unidade
                info!("   Asset: {} - {} {}", 
                    cert.asset.asset_type_str(), 
                    cert.asset.quantity, 
                    cert.asset.unit);
                
                // Exibe a pureza se disponível
                if let Some(purity) = cert.asset.purity {
                    info!("   Purity: {}", purity);
                }
                
                info!("   Serial: {}", cert.asset.serial);
                info!("   Issued at: {}", cert.issued_at);
                
                // Exibe a validade
                if let Some(expires_at) = cert.expires_at {
                    let expired = cert.is_expired();
                    let status = if expired { "EXPIRED" } else { "Valid" };
                    info!("   Status: {} (Expires: {})", status, expires_at);
                } else {
                    info!("   Status: Valid (No expiration)");
                }
                
                // Exibe contagem de tokens emitidos
                if !cert.token_ledgers.is_empty() {
                    let consumed = cert.token_ledgers.iter()
                        .filter(|t| t.consumed_at.is_some())
                        .count();
                    info!("   Tokens: {} issued, {} consumed", cert.token_ledgers.len(), consumed);
                }
                
                info!("   ----------------------");
            }
            
            Ok(())
        },
        Commands::StartNode { identity, port, bootstrap } => {
            // Carrega a identidade que será usada por este nó
            info!("Iniciando nó com identidade de: {}", identity);
            let identity = Identity::load(&identity)?;
            
            // Inicializa o gerenciador de peers
            let peer_manager = PeerManager::new();
            let _peer_manager_arc = Arc::new(Mutex::new(peer_manager));
            
            // Cria a instância do gerenciador de consenso
            let consensus_manager = ConsensusManager::new(PeerManager::new());
            
            // Inicializa o gerenciador de sincronização
            let sync_manager = LedgerSyncManager::new(PeerManager::new(), consensus_manager)?;
            
            // Configura o serviço P2P
            let mut p2p_config = P2PConfig::default();
            p2p_config.listen_port = port;
            
            // Adiciona os nós de bootstrap
            if !bootstrap.is_empty() {
                p2p_config.bootstrap_nodes = bootstrap
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                
                info!("Configurados {} nós de bootstrap", p2p_config.bootstrap_nodes.len());
            }
            
            // Inicializa o serviço P2P
            let p2p_service = P2PService::new(
                identity.clone(),
                PeerManager::new(),
                sync_manager,
                p2p_config
            );
            
            // Inicia o serviço P2P
            info!("Iniciando serviço P2P na porta {}...", port);
            p2p_service.start()?;
            
            // Mantém o programa rodando (em um caso real, isso seria melhor implementado)
            info!("Nó iniciado e escutando. Pressione Ctrl+C para encerrar.");
            loop {
                thread::sleep(Duration::from_secs(60));
                info!("Nó continua em execução...");
            }
            
            #[allow(unreachable_code)]
            Ok(())
        },
        Commands::AddPeer { address: _ } => {
            info!("Esta funcionalidade requer um nó em execução.");
            info!("Por favor, inicie um nó com o comando 'start-node' e use a API REST para adicionar peers.");
            info!("Opcionalmente, inclua o peer na lista de bootstrap ao iniciar o nó.");
            
            Ok(())
        },
        Commands::ListPeers => {
            info!("Esta funcionalidade requer um nó em execução.");
            info!("Por favor, inicie um nó com o comando 'start-node' e use a API REST para listar peers.");
            
            Ok(())
        },
        Commands::BroadcastCertificate { certificate } => {
            // Carrega o certificado do armazenamento local
            let storage = CertificateStorage::new()?;
            let cert = storage.load_by_id(&certificate)?;
            
            info!("Esta funcionalidade requer um nó em execução.");
            info!("Por favor, inicie um nó com o comando 'start-node' para fazer broadcast de certificados.");
            info!("Certificado para broadcast: {}", cert.id);
            info!("Casa de Custódia: {}", cert.custody_house_id);
            info!("Asset: {} - {} {}", 
                cert.asset.asset_type_str(), 
                cert.asset.quantity, 
                cert.asset.unit);
            
            Ok(())
        },
        Commands::SyncCertificates => {
            info!("Esta funcionalidade requer um nó em execução.");
            info!("Por favor, inicie um nó com o comando 'start-node' para sincronizar certificados.");
            
            Ok(())
        }
    }
}
