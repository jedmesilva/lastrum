// Módulo para gerenciamento do registro de tipos de ativos
// Implementa armazenamento e consulta de tipos de ativos aprovados

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::errors::LastrumError;
use crate::governance::{AssetTypeDefinition, AssetCategory};
use crate::certifield::model::AssetType;
use crate::utils::file;

/// Estrutura para armazenar e gerenciar o registro de tipos de ativos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTypeRegistry {
    /// Mapeamento de código para definição de tipo de ativo
    asset_types: HashMap<String, AssetTypeDefinition>,
    /// Última atualização do registro
    last_updated: chrono::DateTime<Utc>,
}

impl AssetTypeRegistry {
    /// Cria um novo registro vazio
    pub fn new() -> Self {
        Self {
            asset_types: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
    
    /// Cria um novo registro com tipos de ativos iniciais
    pub fn with_initial_types() -> Self {
        let mut registry = Self::new();
        
        // Adiciona tipos de metais preciosos
        registry.add_initial_type(
            "GOLD",
            AssetCategory::Metal,
            "Ouro",
            "g",
            true,
            "system"
        );
        
        registry.add_initial_type(
            "SILVER",
            AssetCategory::Metal,
            "Prata",
            "g",
            true,
            "system"
        );
        
        registry.add_initial_type(
            "PLATINUM",
            AssetCategory::Metal,
            "Platina",
            "g",
            true,
            "system"
        );
        
        registry.add_initial_type(
            "PALLADIUM",
            AssetCategory::Metal,
            "Paládio",
            "g",
            true,
            "system"
        );
        
        // Adiciona tipos de energia renovável
        registry.add_initial_type(
            "ENERGIA_SOLAR",
            AssetCategory::Energy,
            "Energia Solar",
            "kWh",
            false,
            "system"
        );
        
        registry.add_initial_type(
            "ENERGIA_EOLICA",
            AssetCategory::Energy,
            "Energia Eólica",
            "kWh",
            false,
            "system"
        );
        
        registry.add_initial_type(
            "ENERGIA_HIDRICA",
            AssetCategory::Energy,
            "Energia Hídrica",
            "kWh",
            false,
            "system"
        );
        
        registry.add_initial_type(
            "BIOGAS_METANO",
            AssetCategory::Energy,
            "Biogás Metano",
            "m3",
            false,
            "system"
        );
        
        registry.add_initial_type(
            "HIDROGENIO_VERDE",
            AssetCategory::Energy,
            "Hidrogênio Verde",
            "kg",
            false,
            "system"
        );
        
        // Adiciona tipo de tempo
        registry.add_initial_type(
            "TEMPO_VEICULO",
            AssetCategory::Time,
            "Tempo de Uso de Veículo",
            "horas",
            false,
            "system"
        );
        
        registry.add_initial_type(
            "TEMPO_IMOVEL",
            AssetCategory::Time,
            "Tempo de Uso de Imóvel",
            "dias",
            false,
            "system"
        );
        
        registry
    }
    
    /// Método interno para adicionar tipos iniciais
    fn add_initial_type(
        &mut self,
        code: &str,
        category: AssetCategory,
        name: &str,
        unit: &str,
        requires_purity: bool,
        proposer: &str,
    ) {
        let asset_type = AssetTypeDefinition {
            code: code.to_string(),
            category,
            name: name.to_string(),
            default_unit: unit.to_string(),
            requires_purity,
            created_at: Utc::now(),
            proposed_by: proposer.to_string(),
        };
        
        self.asset_types.insert(code.to_string(), asset_type);
    }
    
    /// Adiciona um novo tipo de ativo ao registro
    pub fn add_asset_type(&mut self, asset_type: AssetTypeDefinition) -> Result<(), LastrumError> {
        let code = asset_type.code.clone();
        
        // Verifica se já existe um tipo com este código
        if self.asset_types.contains_key(&code) {
            return Err(LastrumError::ValidationError(
                format!("Já existe um tipo de ativo com o código: {}", code)
            ));
        }
        
        // Adiciona ao registro
        self.asset_types.insert(code, asset_type);
        self.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Obtém uma definição de tipo de ativo pelo código
    pub fn get_asset_type(&self, code: &str) -> Option<&AssetTypeDefinition> {
        self.asset_types.get(code)
    }
    
    /// Lista todos os tipos de ativos registrados
    pub fn list_all(&self) -> Vec<&AssetTypeDefinition> {
        self.asset_types.values().collect()
    }
    
    /// Lista todos os tipos de ativos de uma categoria específica
    pub fn list_by_category(&self, category: AssetCategory) -> Vec<&AssetTypeDefinition> {
        self.asset_types
            .values()
            .filter(|&t| t.category == category)
            .collect()
    }
    
    /// Verifica se um tipo de ativo está registrado
    pub fn is_registered(&self, code: &str) -> bool {
        self.asset_types.contains_key(code)
    }
    
    /// Converte um código de ativo em um tipo de ativo certificado
    pub fn to_certified_asset_type(&self, code: &str) -> Result<AssetType, LastrumError> {
        // Primeiro verifica se temos este tipo no registro
        let definition = self.get_asset_type(code).ok_or_else(|| {
            LastrumError::ValidationError(format!("Tipo de ativo não registrado: {}", code))
        })?;
        
        // Faz a conversão para o tipo de ativo do certificado
        // Este mapeamento depende de como o AssetType é definido no seu sistema
        // Este é um exemplo simplificado
        let asset_type = match definition.category {
            AssetCategory::Metal => {
                match code {
                    "GOLD" => AssetType::Gold,
                    "SILVER" => AssetType::Silver,
                    "PLATINUM" => AssetType::Platinum,
                    "PALLADIUM" => AssetType::Palladium,
                    _ => return Err(LastrumError::ValidationError(
                        format!("Tipo de metal não suportado: {}", code)
                    )),
                }
            },
            AssetCategory::Energy => {
                match code {
                    "ENERGIA_SOLAR" => AssetType::EnergiaSolar,
                    "ENERGIA_EOLICA" => AssetType::EnergiaEolica,
                    "ENERGIA_HIDRICA" => AssetType::EnergiaHidrica,
                    "BIOGAS_METANO" => AssetType::BiogasMetano,
                    "HIDROGENIO_VERDE" => AssetType::HidrogenioVerde,
                    _ => return Err(LastrumError::ValidationError(
                        format!("Tipo de energia não suportado: {}", code)
                    )),
                }
            },
            // Para outros tipos, pode ser necessário expandir o enum AssetType
            _ => return Err(LastrumError::ValidationError(
                format!("Categoria não suportada para certificação: {:?}", definition.category)
            )),
        };
        
        Ok(asset_type)
    }
    
    /// Salva o registro em um arquivo JSON
    pub fn save(&self, path: &Path) -> Result<(), LastrumError> {
        let json = serde_json::to_string_pretty(&self)
            .map_err(|e| LastrumError::SerializationError(format!("Erro ao serializar registro: {}", e)))?;
        
        fs::write(path, json)
            .map_err(|e| LastrumError::IoError(format!("Erro ao salvar registro: {}", e)))?;
        
        Ok(())
    }
    
    /// Carrega o registro de um arquivo JSON
    pub fn load(path: &Path) -> Result<Self, LastrumError> {
        // Se o arquivo não existir, cria um novo registro com tipos iniciais
        if !path.exists() {
            let registry = Self::with_initial_types();
            registry.save(path)?;
            return Ok(registry);
        }
        
        let json = fs::read_to_string(path)
            .map_err(|e| LastrumError::IoError(format!("Erro ao ler arquivo de registro: {}", e)))?;
        
        let registry: Self = serde_json::from_str(&json)
            .map_err(|e| LastrumError::DeserializationError(format!("Erro ao deserializar registro: {}", e)))?;
        
        Ok(registry)
    }
}

/// Gerenciador de registro para acesso compartilhado com mutex
pub struct RegistryManager {
    registry: Arc<Mutex<AssetTypeRegistry>>,
    registry_path: PathBuf,
}

impl RegistryManager {
    /// Cria um novo gerenciador de registro
    pub fn new() -> Result<Self, LastrumError> {
        // Define o caminho para o arquivo de registro
        let base_dir = file::get_data_dir()?;
        let registry_path = base_dir.join("asset_registry.json");
        
        // Carrega ou cria o registro
        let registry = AssetTypeRegistry::load(&registry_path)?;
        
        Ok(Self {
            registry: Arc::new(Mutex::new(registry)),
            registry_path,
        })
    }
    
    /// Obtém uma referência ao registro
    pub fn get_registry(&self) -> Result<Arc<Mutex<AssetTypeRegistry>>, LastrumError> {
        Ok(self.registry.clone())
    }
    
    /// Salva o estado atual do registro
    pub fn save(&self) -> Result<(), LastrumError> {
        let registry = self.registry.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock do registro".to_string()))?;
        
        registry.save(&self.registry_path)?;
        
        Ok(())
    }
    
    /// Verifica se um tipo de ativo está registrado
    pub fn is_asset_type_registered(&self, code: &str) -> Result<bool, LastrumError> {
        let registry = self.registry.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock do registro".to_string()))?;
            
        Ok(registry.is_registered(code))
    }
    
    /// Adiciona um novo tipo de ativo ao registro
    pub fn add_asset_type(&self, asset_type: AssetTypeDefinition) -> Result<(), LastrumError> {
        let mut registry = self.registry.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock do registro".to_string()))?;
            
        registry.add_asset_type(asset_type)?;
        
        // Salva o registro atualizado
        self.save()?;
        
        Ok(())
    }
    
    /// Lista todos os tipos de ativos registrados
    pub fn list_all_asset_types(&self) -> Result<Vec<AssetTypeDefinition>, LastrumError> {
        let registry = self.registry.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock do registro".to_string()))?;
            
        Ok(registry.list_all().into_iter().cloned().collect())
    }
    
    /// Converte um código de ativo em um tipo de ativo certificado
    pub fn to_certified_asset_type(&self, code: &str) -> Result<AssetType, LastrumError> {
        let registry = self.registry.lock()
            .map_err(|_| LastrumError::ConcurrencyError("Erro ao adquirir lock do registro".to_string()))?;
            
        registry.to_certified_asset_type(code)
    }
}