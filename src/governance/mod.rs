// Módulo de Governança para o Lastrum Certifield
// Implementa mecanismos para tomada de decisão descentralizada na rede

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::LastrumError;

pub mod registry;
pub mod proposals;

/// Categoria principal de ativos
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetCategory {
    /// Metais preciosos (ouro, prata, etc.)
    Metal,
    /// Energias renováveis (solar, eólica, etc.)
    Energy,
    /// Unidades de tempo (horas de uso, etc.)
    Time,
    /// Propriedades físicas (imóveis, terrenos, etc.)
    Property,
    /// Ativos financeiros
    Financial,
    /// Recursos naturais (água, madeira, etc.)
    NaturalResource,
    /// Outras categorias não padronizadas
    Other,
}

impl AssetCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetCategory::Metal => "METAL",
            AssetCategory::Energy => "ENERGY",
            AssetCategory::Time => "TIME",
            AssetCategory::Property => "PROPERTY",
            AssetCategory::Financial => "FINANCIAL",
            AssetCategory::NaturalResource => "NATURAL_RESOURCE",
            AssetCategory::Other => "OTHER",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "METAL" => Some(AssetCategory::Metal),
            "ENERGY" => Some(AssetCategory::Energy),
            "TIME" => Some(AssetCategory::Time),
            "PROPERTY" => Some(AssetCategory::Property),
            "FINANCIAL" => Some(AssetCategory::Financial),
            "NATURAL_RESOURCE" => Some(AssetCategory::NaturalResource),
            "OTHER" => Some(AssetCategory::Other),
            _ => None,
        }
    }
}

/// Estrutura para definição de um tipo de ativo no registro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeDefinition {
    /// Código único (ex: "GOLD", "VEHICLE_TIME")
    pub code: String,
    /// Categoria principal do ativo
    pub category: AssetCategory,
    /// Nome amigável do ativo
    pub name: String,
    /// Unidade padrão para o ativo
    pub default_unit: String,
    /// Se requer valor de pureza
    pub requires_purity: bool,
    /// Data de aprovação
    pub created_at: DateTime<Utc>,
    /// ID da casa de custódia proponente
    pub proposed_by: String,
}

/// Estrutura para proposta de novos tipos de ativos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProposal {
    /// ID único da proposta
    pub proposal_id: String,
    /// Definição do ativo proposto
    pub asset_definition: AssetTypeDefinition,
    /// Casa de custódia -> voto (true=aprovado)
    pub votes: HashMap<String, bool>,
    /// Data da proposta
    pub proposed_at: DateTime<Utc>,
    /// Data final para votação
    pub voting_ends_at: DateTime<Utc>,
    /// Status da proposta
    pub status: ProposalStatus,
}

/// Status possíveis para uma proposta
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposta aberta para votação
    Pending,
    /// Proposta aprovada
    Approved,
    /// Proposta rejeitada
    Rejected,
    /// Votação encerrada por tempo sem atingir quórum
    Expired,
}

impl AssetProposal {
    /// Cria uma nova proposta de tipo de ativo
    pub fn new(
        asset_definition: AssetTypeDefinition,
        proposer_id: String,
        voting_period_days: u32,
    ) -> Self {
        let now = Utc::now();
        
        // Define o fim da votação baseado no período configurado
        let voting_ends_at = now + chrono::Duration::days(voting_period_days as i64);
        
        let mut votes = HashMap::new();
        // O proponente automaticamente vota a favor
        votes.insert(proposer_id, true);
        
        Self {
            proposal_id: Uuid::new_v4().to_string(),
            asset_definition,
            votes,
            proposed_at: now,
            voting_ends_at,
            status: ProposalStatus::Pending,
        }
    }
    
    /// Adiciona um voto à proposta
    pub fn add_vote(&mut self, custody_house_id: String, vote: bool) -> Result<(), LastrumError> {
        // Verifica se a proposta ainda está aberta para votação
        if self.status != ProposalStatus::Pending {
            return Err(LastrumError::ValidationError(format!(
                "Não é possível votar em uma proposta com status: {:?}", 
                self.status
            )));
        }
        
        // Verifica se já passou o prazo de votação
        if Utc::now() > self.voting_ends_at {
            self.status = ProposalStatus::Expired;
            return Err(LastrumError::ValidationError(
                "O prazo de votação para esta proposta já expirou".to_string()
            ));
        }
        
        // Registra o voto
        self.votes.insert(custody_house_id, vote);
        
        Ok(())
    }
    
    /// Calcula o resultado atual da votação
    pub fn calculate_result(&self, total_custody_houses: usize) -> (usize, usize, bool) {
        let approve_votes = self.votes.values().filter(|&&v| v).count();
        let reject_votes = self.votes.values().filter(|&&v| !v).count();
        
        // Verificação de aprovação por 51% dos participantes
        let approved = approve_votes > (total_custody_houses / 2);
        
        (approve_votes, reject_votes, approved)
    }
    
    /// Finaliza a proposta com um resultado
    pub fn finalize(&mut self, approved: bool) {
        self.status = if approved {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };
    }
}