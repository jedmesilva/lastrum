//! Módulo de rede para Lastrum Certifield
//! 
//! Este módulo contém o código de rede para comunicação com
//! outros nós na rede Lastrum, implementando a infraestrutura
//! descentralizada para emissão e validação de certificados.

pub mod peer;
pub mod protocol;
pub mod broadcaster;
pub mod consensus;
pub mod sync;
pub mod p2p;
