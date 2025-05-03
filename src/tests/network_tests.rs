//! Network tests for Lastrum Certifield
//! 
//! Este módulo contém testes para a camada de rede do aplicativo,
//! incluindo comunicação P2P, consenso e sincronização.

use crate::certifield::{
    builder::CertificateBuilder,
    model::{Asset, AssetType},
    storage::CertificateStorage,
};
use crate::core::{
    identity::Identity,
    signer::Signer,
    validator::Validator,
};
use crate::network::{
    peer::{Peer, PeerManager},
    protocol::{Message, MessageType, ValidationDecision},
    consensus::ConsensusManager,
    broadcaster::Broadcaster,
};

/// Testa a criação de peers e gerenciamento básico
#[test]
fn test_peer_management() {
    // Cria um gerenciador de peers
    let mut peer_manager = PeerManager::new();
    
    // Cria alguns peers para testar
    let peer1 = Peer::new(
        "node1".to_string(),
        "Casa de Custódia 1".to_string(),
        "chave_publica_1".to_string(),
        "192.168.1.1:8080".to_string(),
    );
    
    let peer2 = Peer::new(
        "node2".to_string(),
        "Casa de Custódia 2".to_string(),
        "chave_publica_2".to_string(),
        "192.168.1.2:8080".to_string(),
    );
    
    // Adiciona os peers ao gerenciador
    peer_manager.add_peer(peer1);
    peer_manager.add_peer(peer2);
    
    // Verifica se os peers foram adicionados corretamente
    let peers = peer_manager.get_peers();
    assert_eq!(peers.len(), 2);
    
    // Verifica se podemos recuperar um peer específico
    let found_peer = peer_manager.get_peer_by_id("node1");
    assert!(found_peer.is_some());
    assert_eq!(found_peer.unwrap().name, "Casa de Custódia 1");
    
    // Tenta adicionar o mesmo peer novamente
    let duplicate_peer = Peer::new(
        "node1".to_string(),
        "Duplicado".to_string(),
        "outra_chave".to_string(),
        "192.168.1.3:8080".to_string(),
    );
    
    peer_manager.add_peer(duplicate_peer);
    
    // Verifica que não foi adicionado como duplicado
    let peers = peer_manager.get_peers();
    assert_eq!(peers.len(), 2);
}

/// Testa a criação e verificação de mensagens do protocolo
#[test]
fn test_protocol_messages() {
    // Cria uma identidade para testes
    let identity = Identity::new("Casa de Testes".to_string()).unwrap();
    let node_id = identity.node_hash().to_string();
    
    // Testa a criação de diferentes tipos de mensagens
    let hello_msg = Message::hello(
        node_id.clone(),
        "Casa de Testes",
        &identity.keypair().public_key_hex(),
    );
    
    assert_eq!(hello_msg.msg_type, MessageType::Hello);
    assert!(hello_msg.payload.is_some());
    
    // Cria um asset e certificado de teste
    let asset = Asset::new(
        AssetType::Gold,
        10.0,
        0.9999,
        "TEST123456".to_string(),
    );
    
    let certificate = CertificateBuilder::new()
        .with_issuer("Casa de Testes".to_string())
        .with_issuer_public_key(identity.keypair().public_key_hex())
        .with_asset(asset)
        .build()
        .unwrap();
    
    // Testa a criação de uma mensagem de certificado
    let cert_msg = Message::certificate(node_id.clone(), &certificate).unwrap();
    
    assert_eq!(cert_msg.msg_type, MessageType::Certificate);
    assert!(cert_msg.payload.is_some());
    
    // Testa a criação de uma mensagem de anúncio de certificado
    let announce_msg = Message::new(
        MessageType::NewCertificate,
        node_id.clone(),
        None,
        Some(certificate.id.clone()),
    );
    
    assert_eq!(announce_msg.msg_type, MessageType::NewCertificate);
    assert_eq!(announce_msg.payload, Some(certificate.id.clone()));
}

/// Testa o módulo de consenso
#[test]
fn test_consensus_manager() {
    // Cria um gerenciador de peers para o teste
    let mut peer_manager = PeerManager::new();
    
    // Cria alguns peers para simular uma rede
    for i in 1..6 {
        let peer = Peer::new(
            format!("node{}", i),
            format!("Casa de Custódia {}", i),
            format!("chave_publica_{}", i),
            format!("192.168.1.{}:8080", i),
        );
        
        peer_manager.add_peer(peer);
    }
    
    // Cria um gerenciador de consenso
    let consensus_manager = ConsensusManager::new(peer_manager);
    
    // Cria uma identidade para testes (não utilizada neste teste)
    let _identity = Identity::new("Casa de Testes".to_string()).unwrap();
    
    // Cria um asset e certificado de teste
    let asset = Asset::new(
        AssetType::Gold,
        10.0,
        0.9999,
        "TEST123456".to_string(),
    );
    
    let certificate = CertificateBuilder::new()
        .with_issuer("Casa de Custódia 1".to_string()) // Nome correspondente a um peer na rede
        .with_issuer_public_key("chave_publica_1".to_string()) // Chave correspondente ao peer
        .with_asset(asset)
        .build()
        .unwrap();
    
    // Não podemos simular completamente o consenso em um teste unitário,
    // mas podemos verificar se o processo inicia e executa sem erros
    let result = consensus_manager.validate_certificate(&certificate);
    assert!(result.is_ok());
}

/// Testa o módulo de broadcasting
#[test]
fn test_broadcaster() {
    // Cria uma identidade para testes
    let identity = Identity::new("Casa de Testes".to_string()).unwrap();
    let node_id = identity.node_hash().to_string();
    
    // Cria um broadcaster
    let broadcaster = Broadcaster::new(node_id.clone());
    
    // Cria um asset e certificado de teste
    let asset = Asset::new(
        AssetType::Gold,
        10.0,
        0.9999,
        "TEST123456".to_string(),
    );
    
    let certificate = CertificateBuilder::new()
        .with_issuer("Casa de Testes".to_string())
        .with_issuer_public_key(identity.keypair().public_key_hex())
        .with_asset(asset)
        .build()
        .unwrap();
    
    // Assina o certificado
    let signer = Signer::new(identity.keypair().clone());
    let signed_certificate = signer.sign_certificate(certificate).unwrap();
    
    // Testa o broadcasting (simulado)
    let broadcast_result = broadcaster.broadcast_certificate(&signed_certificate);
    assert!(broadcast_result.is_ok());
    
    // Testa a criação de uma mensagem de anúncio
    let announcement = broadcaster.create_announcement(&signed_certificate.id);
    assert_eq!(announcement.msg_type, MessageType::NewCertificate);
    assert_eq!(announcement.payload, Some(signed_certificate.id.clone()));
}