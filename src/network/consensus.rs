//! Consensus module for Lastrum Certifield
//! 
//! Este módulo implementa o mecanismo de consenso para a rede Lastrum,
//! permitindo a validação automática de certificados emitidos.

use rand::seq::SliceRandom;
use crate::certifield::model::Certificate;
use crate::certifield::storage::CertificateStorage;
use crate::core::validator::Validator;
use crate::core::identity::Identity;
use crate::errors::LastrumError;
use crate::network::peer::{Peer, PeerManager};

/// Resultado de um processo de consenso
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    /// Consenso alcançado, certificado válido
    Approved,
    /// Consenso não alcançado, certificado inválido
    Rejected(String),
    /// Erro durante o processo de consenso
    Error(String),
}

/// Gerenciador de consenso para a rede Lastrum
pub struct ConsensusManager {
    /// Referência ao gerenciador de peers
    peer_manager: PeerManager,
    /// Referência ao validador de certificados
    validator: Validator,
    /// Quantidade mínima de peers para consenso (51%)
    consensus_threshold: f32,
}

impl ConsensusManager {
    /// Cria um novo gerenciador de consenso
    pub fn new(peer_manager: PeerManager) -> Self {
        Self {
            peer_manager,
            validator: Validator::new(),
            consensus_threshold: 0.51,
        }
    }
    
    /// Inicia o processo de consenso para um certificado
    pub fn validate_certificate(&self, certificate: &Certificate) -> Result<ConsensusResult, LastrumError> {
        // Verifica se a casa de custódia emissora existe na rede
        let issuer_exists = self.verify_issuer_exists(certificate)?;
        if !issuer_exists {
            return Ok(ConsensusResult::Rejected(
                format!("Casa de custódia emissora '{}' não encontrada na rede", certificate.issuer)
            ));
        }
        
        // Valida a assinatura do certificado
        let is_valid = self.validator.verify_certificate(certificate)?;
        if !is_valid {
            return Ok(ConsensusResult::Rejected(
                "Assinatura do certificado inválida".to_string()
            ));
        }
        
        // Seleciona um subconjunto aleatório de peers para validação (51%)
        let peers = self.select_random_validators()?;
        if peers.is_empty() {
            return Ok(ConsensusResult::Error(
                "Não há peers suficientes na rede para validação".to_string()
            ));
        }
        
        // Simula a validação pelos peers selecionados
        // Em uma implementação real, isso enviaria mensagens aos peers e aguardaria respostas
        
        // Por enquanto, assumimos que todos os peers selecionados aprovariam o certificado
        // já que verificamos a validade da assinatura anteriormente
        let approvals = peers.len();
        
        // Calcula o percentual de aprovação
        let approval_ratio = approvals as f32 / peers.len() as f32;
        
        // Verifica se alcançou o threshold de consenso
        if approval_ratio >= self.consensus_threshold {
            // Consenso alcançado, armazena o certificado no ledger local
            self.store_certificate(certificate)?;
            
            Ok(ConsensusResult::Approved)
        } else {
            Ok(ConsensusResult::Rejected(
                format!("Certificado rejeitado: aprovação de {}% (mínimo {}%)", 
                    (approval_ratio * 100.0) as u32, 
                    (self.consensus_threshold * 100.0) as u32
                )
            ))
        }
    }
    
    /// Verifica se a casa de custódia emissora existe na rede
    fn verify_issuer_exists(&self, certificate: &Certificate) -> Result<bool, LastrumError> {
        // Verifica se existe algum peer com o nome correspondente ao emissor do certificado
        let peers = self.peer_manager.get_peers();
        let issuer_exists = peers.iter().any(|p| p.name == certificate.issuer);
        
        // Verifica também se a chave pública corresponde
        let public_key_valid = if issuer_exists {
            peers.iter()
                .find(|p| p.name == certificate.issuer)
                .map(|p| p.public_key == certificate.issuer_public_key)
                .unwrap_or(false)
        } else {
            false
        };
        
        Ok(issuer_exists && public_key_valid)
    }
    
    /// Seleciona validadores aleatórios para o consenso
    fn select_random_validators(&self) -> Result<Vec<&Peer>, LastrumError> {
        let peers = self.peer_manager.get_peers();
        
        // Se não houver peers suficientes, retorna erro
        if peers.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut rng = rand::thread_rng();
        
        // Determina quantos peers devem participar (51% do total, no mínimo 1)
        let validation_count = (peers.len() as f32 * self.consensus_threshold).ceil() as usize;
        let validation_count = std::cmp::max(1, validation_count);
        
        // Seleciona aleatoriamente os peers
        let mut selected_peers: Vec<&Peer> = Vec::with_capacity(validation_count);
        let mut indices: Vec<usize> = (0..peers.len()).collect();
        indices.shuffle(&mut rng);
        
        for i in 0..std::cmp::min(validation_count, peers.len()) {
            selected_peers.push(&peers[indices[i]]);
        }
        
        Ok(selected_peers)
    }
    
    /// Armazena o certificado validado no ledger local
    fn store_certificate(&self, certificate: &Certificate) -> Result<(), LastrumError> {
        let storage = CertificateStorage::new()?;
        storage.store(certificate)?;
        
        Ok(())
    }
}