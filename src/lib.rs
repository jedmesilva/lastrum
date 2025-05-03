//! Lastrum Certifield
//! 
//! Um sistema descentralizado para casas de custódia emitirem e
//! validarem certificados para ativos físicos como ouro e prata.
//! 
//! ## Principais componentes
//! 
//! - **Identidade**: Gerenciamento de identidades para casas de custódia
//! - **Certificados**: Emissão e verificação de certificados para ativos físicos
//! - **Rede Descentralizada**: Comunicação P2P entre nós da rede
//! - **Consenso**: Mecanismo de consenso que permite validação automática de certificados
//! - **Sincronização**: Mantém o ledger sincronizado entre todos os participantes
//! 
//! ## Mecânica de consenso
//! 
//! O sistema implementa um mecanismo de consenso especializado onde:
//! 
//! 1. Certificados são emitidos por casas de custódia com identidades verificáveis
//! 2. Quando um certificado é emitido, ele é propagado para a rede
//! 3. Um conjunto aleatório de nós (51% do total) é selecionado para validar o certificado
//! 4. A validação é puramente matemática: verificação da assinatura digital
//! 5. O consenso é alcançado automaticamente, sem intervenção humana
//! 
//! Este modelo garante que nenhuma casa de custódia possa ser discriminada
//! através de rejeições arbitrárias, já que o consenso é baseado apenas na
//! verificação criptográfica da assinatura.

pub mod config;
pub mod core;
pub mod certifield;
pub mod network;
pub mod utils;
pub mod errors;

#[cfg(test)]
mod tests;
