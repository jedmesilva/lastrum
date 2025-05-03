// Módulo de Governança para o Lastrum Certifield
// Implementa mecanismos para tomada de decisão descentralizada na rede

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::LastrumError;
use crate::core::signer::Signer;

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
    /// Descrição detalhada do tipo de ativo
    #[serde(default)]
    pub description: String,
    /// Unidade padrão para o ativo
    pub default_unit: String,
    /// Se requer valor de pureza
    pub requires_purity: bool,
    /// Data de aprovação
    pub created_at: DateTime<Utc>,
    /// ID da casa de custódia proponente
    pub proposed_by: String,
}

/// Estrutura para um voto assinado criptograficamente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedVote {
    /// ID da casa de custódia que emitiu o voto
    pub custody_house_id: String,
    /// Valor do voto (true = aprova, false = rejeita)
    pub approve: bool,
    /// Momento em que o voto foi registrado
    pub timestamp: DateTime<Utc>,
    /// Hash do voto (inclui proposal_id + custody_house_id + approve + timestamp)
    pub vote_hash: String,
    /// Assinatura digital do hash com a chave privada da casa de custódia
    pub signature: String,
}

impl SignedVote {
    /// Cria um novo voto assinado
    pub fn new(proposal_id: &str, custody_house_id: &str, approve: bool, signer: &Signer) -> Result<Self, LastrumError> {
        let timestamp = Utc::now();
        
        // Cria o conteúdo do hash
        let vote_content = format!(
            "{}:{}:{}:{}",
            proposal_id,
            custody_house_id,
            approve,
            timestamp.to_rfc3339()
        );
        
        // Gera o hash do voto
        let vote_hash = signer.hash(&vote_content)?;
        
        // Assina o hash
        let signature = signer.sign_to_hex(vote_hash.as_bytes())?;
        
        Ok(Self {
            custody_house_id: custody_house_id.to_string(),
            approve,
            timestamp,
            vote_hash,
            signature,
        })
    }
    
    /// Verifica se a assinatura do voto é válida
    pub fn verify(&self, signer: &Signer) -> Result<bool, LastrumError> {
        signer.verify(&self.vote_hash, &self.signature)
    }
}

/// Estrutura para proposta de novos tipos de ativos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetProposal {
    /// ID único da proposta
    pub proposal_id: String,
    /// Definição do ativo proposto
    pub asset_definition: AssetTypeDefinition,
    /// Lista de votos assinados
    pub signed_votes: Vec<SignedVote>,
    /// Mapa para acesso rápido aos votos (Casa de custódia -> voto)
    #[serde(skip_serializing, skip_deserializing)]
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
        // O proponente automaticamente vota a favor, mas o voto assinado
        // será adicionado separadamente
        votes.insert(proposer_id.clone(), true);
        
        Self {
            proposal_id: Uuid::new_v4().to_string(),
            asset_definition,
            signed_votes: Vec::new(),
            votes,
            proposed_at: now,
            voting_ends_at,
            status: ProposalStatus::Pending,
        }
    }
    
    /// Adiciona o voto inicial do proponente (assinado)
    pub fn add_proposer_vote(&mut self, signer: &Signer) -> Result<(), LastrumError> {
        let proposer_id = self.asset_definition.proposed_by.clone();
        self.add_signed_vote(proposer_id, true, signer)
    }
    
    /// Adiciona um voto assinado à proposta
    pub fn add_signed_vote(&mut self, custody_house_id: String, approve: bool, signer: &Signer) -> Result<(), LastrumError> {
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
        
        // Verifica se a casa de custódia já votou
        if self.votes.contains_key(&custody_house_id) {
            return Err(LastrumError::ValidationError(
                "Esta casa de custódia já votou nesta proposta".to_string()
            ));
        }
        
        // Cria um voto assinado
        let signed_vote = SignedVote::new(&self.proposal_id, &custody_house_id, approve, signer)?;
        
        // Adiciona o voto ao registro
        self.votes.insert(custody_house_id, approve);
        // Adiciona o voto assinado à lista de votos
        self.signed_votes.push(signed_vote);
        
        Ok(())
    }
    
    /// Verifica a validade de todos os votos assinados
    pub fn verify_votes(&self, signer: &Signer) -> Result<bool, LastrumError> {
        for vote in &self.signed_votes {
            if !vote.verify(signer)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// Reconstrói o mapa de votos a partir dos votos assinados
    pub fn rebuild_votes_map(&mut self) {
        self.votes = HashMap::new();
        for vote in &self.signed_votes {
            self.votes.insert(vote.custody_house_id.clone(), vote.approve);
        }
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
    
    /// Adiciona um voto sem assinatura (compatibilidade com código existente)
    /// Usado apenas para migração de dados ou testes
    #[deprecated(note = "Use add_signed_vote instead")]
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
}