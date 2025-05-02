//! Peer management for the Lastrum network
//! 
//! This module handles peer discovery and management.
//! 
//! Note: This is a placeholder module for future development.

use serde::{Serialize, Deserialize};

/// Represents a peer node in the Lastrum network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Unique identifier for this peer
    pub id: String,
    /// Name of the custody house
    pub name: String,
    /// Public key of the peer
    pub public_key: String,
    /// Network address of the peer
    pub address: String,
    /// Last time this peer was seen
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

impl Peer {
    /// Create a new peer
    pub fn new(id: String, name: String, public_key: String, address: String) -> Self {
        Self {
            id,
            name,
            public_key,
            address,
            last_seen: None,
        }
    }
    
    /// Update the last seen timestamp
    pub fn mark_seen(&mut self) {
        self.last_seen = Some(chrono::Utc::now());
    }
}

/// Manager for peer connections
pub struct PeerManager {
    peers: Vec<Peer>,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new() -> Self {
        Self {
            peers: Vec::new(),
        }
    }
    
    /// Add a peer to the manager
    pub fn add_peer(&mut self, peer: Peer) {
        // Check if we already know this peer
        if !self.peers.iter().any(|p| p.id == peer.id) {
            self.peers.push(peer);
        }
    }
    
    /// Get all known peers
    pub fn get_peers(&self) -> &[Peer] {
        &self.peers
    }
    
    /// Get a peer by ID
    pub fn get_peer_by_id(&self, id: &str) -> Option<&Peer> {
        self.peers.iter().find(|p| p.id == id)
    }
}
