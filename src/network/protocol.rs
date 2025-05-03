//! Network protocol for Lastrum Certifield
//! 
//! This module defines the message types and protocol for
//! communication between Lastrum nodes.
//! 
//! Note: This is a placeholder module for future development.

use serde::{Serialize, Deserialize};
use crate::certifield::model::Certificate;

/// Tipos de mensagens trocadas na rede
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// Mensagem de apresentação de um nó
    Hello,
    /// Solicitação de lista de peers conhecidos
    GetPeers,
    /// Resposta com lista de peers conhecidos
    Peers,
    /// Anúncio de novo certificado
    NewCertificate,
    /// Entrega do certificado completo
    Certificate,
    /// Solicitação de um certificado específico
    GetCertificate,
    /// Solicitação de validação de um certificado
    ValidateCertificate,
    /// Resposta de validação de um certificado
    ValidationResult,
    /// Solicitação de participação em consenso
    ConsensusRequest,
    /// Voto em um processo de consenso
    ConsensusVote,
    /// Resultado final de um processo de consenso
    ConsensusResult,
    /// Ping para verificar se um nó está ativo
    Ping,
    /// Resposta ao ping
    Pong,
    /// Mensagem de descoberta de rede
    Discover,
    /// Mensagem de anúncio de presença na rede
    Announce,
}

/// A network message sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Type of message
    pub msg_type: MessageType,
    /// Sender's node ID
    pub sender: String,
    /// Recipient's node ID (empty for broadcasts)
    pub recipient: Option<String>,
    /// Message payload
    pub payload: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Enum que representa o resultado da validação de um certificado
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationDecision {
    /// Certificado válido (assinatura verificada matematicamente)
    Valid,
    /// Certificado inválido (falha na verificação da assinatura)
    Invalid,
    /// Certificado rejeitado pelo consenso da rede
    Rejected,
    /// Certificado pendente de validação
    Pending,
}

/// Estrutura que representa um voto em um processo de consenso
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusVotePayload {
    /// ID do certificado sendo votado
    pub certificate_id: String,
    /// Decisão do voto
    pub decision: ValidationDecision,
    /// Comentário opcional (razão da rejeição, se aplicável)
    pub comment: Option<String>,
}

impl Message {
    /// Cria uma nova mensagem
    pub fn new(
        msg_type: MessageType,
        sender: String,
        recipient: Option<String>,
        payload: Option<String>,
    ) -> Self {
        Self {
            msg_type,
            sender,
            recipient,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Cria uma mensagem de certificado
    pub fn certificate(sender: String, certificate: &Certificate) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_string(certificate)?;
        
        Ok(Self {
            msg_type: MessageType::Certificate,
            sender,
            recipient: None, // broadcast
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Cria uma mensagem de apresentação
    pub fn hello(sender: String, name: &str, public_key: &str) -> Self {
        let payload = format!("{}:{}", name, public_key);
        
        Self {
            msg_type: MessageType::Hello,
            sender,
            recipient: None, // broadcast
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Cria uma mensagem de anúncio de novo certificado
    pub fn new_certificate(sender: String, certificate_id: &str) -> Self {
        Self {
            msg_type: MessageType::NewCertificate,
            sender,
            recipient: None, // broadcast
            payload: Some(certificate_id.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Cria uma mensagem de solicitação de validação de certificado
    pub fn validate_certificate(sender: String, recipient: Option<String>, certificate: &Certificate) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_string(certificate)?;
        
        Ok(Self {
            msg_type: MessageType::ValidateCertificate,
            sender,
            recipient,
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Cria uma mensagem de resultado de validação
    pub fn validation_result(sender: String, recipient: String, certificate_id: &str, decision: ValidationDecision) -> Self {
        let result = serde_json::json!({
            "certificate_id": certificate_id,
            "decision": decision,
        });
        
        Self {
            msg_type: MessageType::ValidationResult,
            sender,
            recipient: Some(recipient),
            payload: Some(result.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Cria uma mensagem de solicitação de consenso
    pub fn consensus_request(sender: String, certificate_id: &str) -> Self {
        Self {
            msg_type: MessageType::ConsensusRequest,
            sender,
            recipient: None, // broadcast para participantes do consenso
            payload: Some(certificate_id.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Cria uma mensagem de voto em um consenso
    pub fn consensus_vote(sender: String, coordinator: String, certificate_id: &str, decision: ValidationDecision) -> Result<Self, serde_json::Error> {
        let vote = ConsensusVotePayload {
            certificate_id: certificate_id.to_string(),
            decision,
            comment: None,
        };
        
        let payload = serde_json::to_string(&vote)?;
        
        Ok(Self {
            msg_type: MessageType::ConsensusVote,
            sender,
            recipient: Some(coordinator),
            payload: Some(payload),
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Cria uma mensagem com o resultado final de um consenso
    pub fn consensus_result(sender: String, certificate_id: &str, decision: ValidationDecision) -> Self {
        let result = serde_json::json!({
            "certificate_id": certificate_id,
            "final_decision": decision,
        });
        
        Self {
            msg_type: MessageType::ConsensusResult,
            sender,
            recipient: None, // broadcast para todos
            payload: Some(result.to_string()),
            timestamp: chrono::Utc::now(),
        }
    }
}
