// Módulo para gerenciamento de propostas de novos tipos de ativos
// Implementa armazenamento, votação e processamento de propostas

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

use crate::errors::LastrumError;
use crate::governance::{AssetProposal, AssetTypeDefinition, ProposalStatus};
use crate::governance::registry::RegistryManager;
use crate::utils::file;

/// Gerenciador de propostas de tipos de ativos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalManager {
    /// Propostas abertas para votação
    pending_proposals: HashMap<String, AssetProposal>,
    /// Propostas finalizadas (aprovadas/rejeitadas)
    finalized_proposals: HashMap<String, AssetProposal>,
    /// Última atualização do registro de propostas
    last_updated: chrono::DateTime<Utc>,
}

impl ProposalManager {
    /// Cria um novo gerenciador de propostas vazio
    pub fn new() -> Self {
        Self {
            pending_proposals: HashMap::new(),
            finalized_proposals: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
    
    /// Adiciona uma nova proposta
    pub fn add_proposal(&mut self, proposal: AssetProposal) -> Result<(), LastrumError> {
        let id = proposal.proposal_id.clone();
        println!("ProposalManager::add_proposal - Adicionando proposta com ID: {}", id);
        
        // Verifica se já existe uma proposta com este ID
        if self.pending_proposals.contains_key(&id) {
            println!("ProposalManager::add_proposal - Proposta já existe em pending_proposals");
            return Err(LastrumError::ValidationError(
                format!("Já existe uma proposta pendente com o ID: {}", id)
            ));
        }
        
        if self.finalized_proposals.contains_key(&id) {
            println!("ProposalManager::add_proposal - Proposta já existe em finalized_proposals");
            return Err(LastrumError::ValidationError(
                format!("Já existe uma proposta finalizada com o ID: {}", id)
            ));
        }
        
        // Adiciona à lista de propostas pendentes
        println!("ProposalManager::add_proposal - Inserindo proposta no mapa");
        self.pending_proposals.insert(id.clone(), proposal);
        println!("ProposalManager::add_proposal - Proposta inserida, agora temos {} propostas pendentes", 
                 self.pending_proposals.len());
        
        // Atualiza a data de atualização
        self.last_updated = Utc::now();
        
        // Testa a serialização
        match serde_json::to_string_pretty(&self) {
            Ok(json) => println!("ProposalManager::add_proposal - Serialização de teste bem-sucedida: {} bytes", json.len()),
            Err(e) => println!("ProposalManager::add_proposal - ERRO ao serializar: {:?}", e),
        }
        
        Ok(())
    }
    
    /// Obtém uma proposta pelo ID
    pub fn get_proposal(&self, id: &str) -> Option<&AssetProposal> {
        self.pending_proposals.get(id).or_else(|| self.finalized_proposals.get(id))
    }
    
    /// Obtém uma proposta pendente pelo ID para modificação
    pub fn get_pending_proposal_mut(&mut self, id: &str) -> Option<&mut AssetProposal> {
        self.pending_proposals.get_mut(id)
    }
    
    /// Lista todas as propostas pendentes
    pub fn list_pending_proposals(&self) -> Vec<&AssetProposal> {
        self.pending_proposals.values().collect()
    }
    
    /// Lista todas as propostas finalizadas
    pub fn list_finalized_proposals(&self) -> Vec<&AssetProposal> {
        self.finalized_proposals.values().collect()
    }
    
    /// Adiciona um voto a uma proposta
    pub fn add_vote(&mut self, proposal_id: &str, custody_house_id: String, vote: bool) -> Result<(), LastrumError> {
        // Verifica se a proposta existe e está pendente
        let proposal = self.pending_proposals.get_mut(proposal_id)
            .ok_or_else(|| LastrumError::NotFound(format!("Proposta não encontrada: {}", proposal_id)))?;
        
        // Adiciona o voto
        proposal.add_vote(custody_house_id, vote)?;
        
        // Atualiza timestamp
        self.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Finaliza uma proposta (após aprovação ou rejeição)
    pub fn finalize_proposal(&mut self, proposal_id: &str, approved: bool) -> Result<AssetTypeDefinition, LastrumError> {
        // Verifica se a proposta existe e está pendente
        let proposal = self.pending_proposals.remove(proposal_id)
            .ok_or_else(|| LastrumError::NotFound(format!("Proposta pendente não encontrada: {}", proposal_id)))?;
        
        // Cria uma cópia mutável para finalizar
        let mut proposal_to_finalize = proposal;
        
        // Finaliza a proposta
        proposal_to_finalize.finalize(approved);
        
        // Se foi aprovada, retorna a definição do tipo de ativo
        let asset_definition = if approved {
            Some(proposal_to_finalize.asset_definition.clone())
        } else {
            None
        };
        
        // Adiciona à lista de propostas finalizadas
        self.finalized_proposals.insert(proposal_id.to_string(), proposal_to_finalize);
        
        // Atualiza timestamp
        self.last_updated = Utc::now();
        
        // Retorna a definição se a proposta foi aprovada
        asset_definition.ok_or_else(|| LastrumError::ValidationError("Proposta foi rejeitada".to_string()))
    }
    
    /// Verifica propostas com prazo expirado
    pub fn check_expired_proposals(&mut self, total_custody_houses: usize) -> Vec<(String, bool)> {
        let now = Utc::now();
        let mut expired_results = Vec::new();
        let mut proposals_to_finalize = Vec::new();
        
        // Verifica cada proposta pendente
        for (id, proposal) in &self.pending_proposals {
            // Se o prazo já expirou
            if now > proposal.voting_ends_at {
                // Calcula o resultado da votação
                let (approve_votes, _, approved) = proposal.calculate_result(total_custody_houses);
                
                // Adiciona à lista para finalização
                proposals_to_finalize.push((id.clone(), approved));
            }
        }
        
        // Finaliza as propostas expiradas
        for (id, approved) in proposals_to_finalize {
            if let Ok(asset_definition) = self.finalize_proposal(&id, approved) {
                // Adiciona ao resultado
                expired_results.push((id, approved));
            }
        }
        
        expired_results
    }
    
    /// Salva o gerenciador de propostas em um arquivo JSON usando uma abordagem simplificada
    pub fn save(&self, path: &Path) -> Result<(), LastrumError> {
        println!("ProposalManager::save - Iniciando salvamento em: {:?}", path);
        println!("ProposalManager::save - Propostas pendentes: {}", self.pending_proposals.len());
        
        // Cria uma estrutura simplificada para serialização
        let simplified_data = SimplifiedProposalData {
            pending: self.pending_proposals.len(),
            finalized: self.finalized_proposals.len(),
            timestamp: Utc::now().to_string(),
            proposal_ids: self.pending_proposals.keys().cloned().collect(),
        };
        
        println!("ProposalManager::save - Criada estrutura simplificada com {} propostas", simplified_data.pending);
        
        // Serializa a estrutura simplificada
        let json = match serde_json::to_string_pretty(&simplified_data) {
            Ok(json_str) => {
                println!("ProposalManager::save - Serialização bem-sucedida, tamanho: {} bytes", json_str.len());
                json_str
            },
            Err(e) => {
                println!("ProposalManager::save - Erro na serialização: {:?}", e);
                return Err(LastrumError::SerializationError(format!("Erro ao serializar propostas: {}", e)));
            }
        };
        
        // Cria o diretório pai se não existir
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                println!("ProposalManager::save - Criando diretório pai: {:?}", parent);
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("ProposalManager::save - Erro ao criar diretório: {:?}", e);
                    return Err(LastrumError::IoError(format!("Erro ao criar diretório: {}", e)));
                }
            }
        }
        
        // Escreve o arquivo temporário primeiro
        let tmp_path = path.with_extension("tmp");
        println!("ProposalManager::save - Escrevendo arquivo temporário: {:?}", tmp_path);
        
        if let Err(e) = fs::write(&tmp_path, &json) {
            println!("ProposalManager::save - Erro ao escrever arquivo temporário: {:?}", e);
            return Err(LastrumError::IoError(format!("Erro ao salvar propostas temporárias: {}", e)));
        }
        
        // Renomeia o arquivo temporário para o nome final (operação atômica)
        println!("ProposalManager::save - Renomeando arquivo temporário para final");
        if let Err(e) = fs::rename(&tmp_path, path) {
            println!("ProposalManager::save - Erro ao renomear arquivo: {:?}", e);
            return Err(LastrumError::IoError(format!("Erro ao finalizar salvamento: {}", e)));
        }
        
        println!("ProposalManager::save - Salvamento concluído com sucesso!");
        
        Ok(())
    }
    
    // Estrutura simplificada para serialização
    #[derive(Serialize, Deserialize)]
    struct SimplifiedProposalData {
        pending: usize,
        finalized: usize,
        timestamp: String,
        proposal_ids: Vec<String>,
    }
    
    /// Carrega o gerenciador de propostas de um arquivo JSON
    pub fn load(path: &Path) -> Result<Self, LastrumError> {
        // Se o arquivo não existir, cria um novo gerenciador vazio
        if !path.exists() {
            let manager = Self::new();
            manager.save(path)?;
            return Ok(manager);
        }
        
        let json = fs::read_to_string(path)
            .map_err(|e| LastrumError::IoError(format!("Erro ao ler arquivo de propostas: {}", e)))?;
        
        let manager: Self = serde_json::from_str(&json)
            .map_err(|e| LastrumError::DeserializationError(format!("Erro ao deserializar propostas: {}", e)))?;
        
        Ok(manager)
    }
}

/// Gerenciador de propostas para acesso compartilhado com mutex
pub struct ProposalService {
    proposals: Arc<Mutex<ProposalManager>>,
    registry_manager: Arc<RegistryManager>,
    proposals_path: PathBuf,
    total_custody_houses: usize, // Idealmente, isso viria de um serviço de identidade
}

impl ProposalService {
    /// Cria um novo serviço de propostas
    pub fn new(registry_manager: Arc<RegistryManager>, total_custody_houses: usize) -> Result<Self, LastrumError> {
        // Define o caminho para o arquivo de propostas
        let base_dir = file::get_data_dir()?;
        let proposals_path = base_dir.join("asset_proposals.json");
        
        // Carrega ou cria o gerenciador de propostas
        let proposals = ProposalManager::load(&proposals_path)?;
        
        Ok(Self {
            proposals: Arc::new(Mutex::new(proposals)),
            registry_manager,
            proposals_path,
            total_custody_houses,
        })
    }
    
    /// Obtém uma referência ao gerenciador de propostas
    pub fn get_proposals(&self) -> Result<Arc<Mutex<ProposalManager>>, LastrumError> {
        Ok(self.proposals.clone())
    }
    
    /// Salva o estado atual das propostas
    pub fn save(&self) -> Result<(), LastrumError> {
        println!("Iniciando salvamento de propostas...");
        println!("Caminho do arquivo: {:?}", self.proposals_path);
        
        let lock_result = self.proposals.lock();
        if let Err(e) = lock_result {
            println!("Erro ao adquirir lock para salvamento: {:?}", e);
            return Err(LastrumError::ConcurrencyError(format!("Erro ao adquirir lock para salvamento: {:?}", e)));
        }
        
        let proposals = lock_result.unwrap();
        println!("Lock adquirido para salvamento");
        
        // Conta quantas propostas pendentes e finalizadas existem
        let num_pending = proposals.pending_proposals.len();
        let num_finalized = proposals.finalized_proposals.len();
        println!("Número de propostas pendentes: {}", num_pending);
        println!("Número de propostas finalizadas: {}", num_finalized);
        
        // Tenta salvar e captura qualquer erro
        let save_result = proposals.save(&self.proposals_path);
        if let Err(e) = save_result {
            println!("Erro ao salvar propostas: {:?}", e);
            return Err(e);
        }
        
        println!("Propostas salvas com sucesso!");
        
        // Verifica se o arquivo foi realmente atualizado
        if let Ok(metadata) = std::fs::metadata(&self.proposals_path) {
            if let Ok(modified) = metadata.modified() {
                println!("Arquivo atualizado em: {:?}", modified);
            }
        }
        
        Ok(())
    }
    
    /// Cria uma nova proposta de tipo de ativo
    pub fn propose_asset_type(
        &self,
        code: String,
        category: String,
        name: String,
        description: String,
        unit: String,
        requires_purity: bool,
        proposer_id: String,
        voting_period_days: u32,
    ) -> Result<String, LastrumError> {
        // Log inicial para debugging
        println!("Iniciando criação de proposta para tipo de ativo: {} ({})", name, code);
        println!("Categoria: {}, Proponente: {}", category, proposer_id);
        
        // Verificar se o código já existe no registro
        if let Err(e) = self.registry_manager.is_asset_type_registered(&code) {
            println!("Erro ao verificar se o tipo de ativo já existe: {:?}", e);
            return Err(e);
        }
        
        if self.registry_manager.is_asset_type_registered(&code)? {
            println!("Código já registrado: {}", code);
            return Err(LastrumError::ValidationError(
                format!("Já existe um tipo de ativo registrado com o código: {}", code)
            ));
        }
        
        // Converter a categoria
        let asset_category_option = crate::governance::AssetCategory::from_str(&category);
        if asset_category_option.is_none() {
            println!("Categoria inválida: {}", category);
            return Err(LastrumError::ValidationError(format!("Categoria inválida: {}", category)));
        }
        let asset_category = asset_category_option.unwrap();
        println!("Categoria válida: {:?}", asset_category);
        
        // Criar a definição do tipo de ativo
        let asset_definition = AssetTypeDefinition {
            code: code.clone(),
            category: asset_category,
            name,
            description,
            default_unit: unit,
            requires_purity,
            created_at: Utc::now(),
            proposed_by: proposer_id.clone(),
        };
        
        println!("Criando proposta para definição: {:?}", asset_definition);
        
        // Criar a proposta
        let proposal = AssetProposal::new(
            asset_definition,
            proposer_id,
            voting_period_days,
        );
        
        let proposal_id = proposal.proposal_id.clone();
        println!("ID da proposta gerado: {}", proposal_id);
        
        // Adicionar ao gerenciador de propostas
        println!("Adquirindo lock para adicionar proposta...");
        let lock_result = self.proposals.lock();
        if let Err(e) = lock_result {
            println!("Erro ao adquirir lock: {:?}", e);
            return Err(LastrumError::ConcurrencyError(format!("Erro ao adquirir lock das propostas: {:?}", e)));
        }
        
        let mut proposals = lock_result.unwrap();
        println!("Lock adquirido com sucesso");
        
        if let Err(e) = proposals.add_proposal(proposal.clone()) {
            println!("Erro ao adicionar proposta: {:?}", e);
            return Err(e);
        }
        println!("Proposta adicionada com sucesso");
        
        // Salvar as alterações
        println!("Salvando alterações...");
        if let Err(e) = self.save() {
            println!("Erro ao salvar alterações: {:?}", e);
            return Err(e);
        }
        println!("Alterações salvas com sucesso");
        
        Ok(proposal_id)
    }
    
    /// Adiciona um voto a uma proposta
    pub fn vote_on_proposal(&self, proposal_id: &str, custody_house_id: String, vote: bool) -> Result<(), LastrumError> {
        let mut proposals = self.proposals.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock das propostas".to_string()))?;
        
        proposals.add_vote(proposal_id, custody_house_id, vote)?;
        
        // Verifica se a proposta atingiu o limite para aprovação/rejeição
        if let Some(proposal) = proposals.get_proposal(proposal_id) {
            let (approve_votes, reject_votes, approved) = proposal.calculate_result(self.total_custody_houses);
            
            // Se já temos votos suficientes para determinar o resultado
            let majority_reached = approve_votes > (self.total_custody_houses / 2) ||
                                  reject_votes >= (self.total_custody_houses / 2);
            
            if majority_reached {
                // Finaliza a proposta
                let asset_definition = proposals.finalize_proposal(proposal_id, approved)?;
                
                // Se foi aprovada, adiciona ao registro
                if approved {
                    self.registry_manager.add_asset_type(asset_definition)?;
                }
            }
        }
        
        // Salva as alterações
        self.save()?;
        
        Ok(())
    }
    
    /// Lista todas as propostas pendentes
    pub fn list_pending_proposals(&self) -> Result<Vec<AssetProposal>, LastrumError> {
        let proposals = self.proposals.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock das propostas".to_string()))?;
            
        Ok(proposals.list_pending_proposals().into_iter().cloned().collect())
    }
    
    /// Lista todas as propostas finalizadas
    pub fn list_finalized_proposals(&self) -> Result<Vec<AssetProposal>, LastrumError> {
        let proposals = self.proposals.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock das propostas".to_string()))?;
            
        Ok(proposals.list_finalized_proposals().into_iter().cloned().collect())
    }
    
    /// Verifica propostas com prazo expirado
    pub fn check_expired_proposals(&self) -> Result<Vec<(String, bool)>, LastrumError> {
        let mut proposals = self.proposals.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock das propostas".to_string()))?;
        
        let expired_results = proposals.check_expired_proposals(self.total_custody_houses);
        
        // Para cada proposta aprovada, adiciona ao registro
        for (id, approved) in &expired_results {
            if *approved {
                if let Some(proposal) = proposals.get_proposal(id) {
                    if proposal.status == ProposalStatus::Approved {
                        self.registry_manager.add_asset_type(proposal.asset_definition.clone())?;
                    }
                }
            }
        }
        
        // Salva as alterações
        self.save()?;
        
        Ok(expired_results)
    }
}