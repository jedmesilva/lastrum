//! Network protocol for Lastrum Certifield
//! 
//! This module defines the message types and protocol for
//! communication between Lastrum nodes.
//! 
//! Note: This is a placeholder module for future development.

use serde::{Serialize, Deserialize};
use crate::certifield::model::Certificate;

/// Types of messages that can be sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Hello message to introduce a node
    Hello,
    /// Request for peers
    GetPeers,
    /// Response with peer list
    Peers,
    /// New certificate broadcast
    Certificate,
    /// Request for a specific certificate
    GetCertificate,
    /// Ping to check if a node is alive
    Ping,
    /// Pong response to a ping
    Pong,
}

/// A network message sent between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Message {
    /// Create a new message
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
    
    /// Create a certificate message
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
    
    /// Create a hello message
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
}
