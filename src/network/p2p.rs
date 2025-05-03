//! Módulo P2P para Lastrum Certifield
//! 
//! Este módulo implementa a comunicação peer-to-peer entre os nós da rede,
//! permitindo a descoberta e troca de mensagens entre casas de custódia.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

use crate::errors::LastrumError;
use crate::core::identity::Identity;
use crate::network::peer::{Peer, PeerManager};
use crate::network::protocol::{Message, MessageType};
use crate::network::sync::LedgerSyncManager;

/// Configuração para o serviço P2P
pub struct P2PConfig {
    /// Porta para o serviço de escuta
    pub listen_port: u16,
    /// Lista de nós iniciais para se conectar (endereços)
    pub bootstrap_nodes: Vec<String>,
    /// Intervalo de atualização da lista de peers (em segundos)
    pub peer_refresh_interval: u64,
    /// Tempo máximo de inatividade para desconexão (em segundos)
    pub peer_timeout: u64,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_port: 8080,
            bootstrap_nodes: Vec::new(),
            peer_refresh_interval: 300, // 5 minutos
            peer_timeout: 600, // 10 minutos
        }
    }
}

/// Gerenciador do serviço P2P
pub struct P2PService {
    /// Identidade do nó local
    identity: Identity,
    /// Gerenciador de peers
    peer_manager: Arc<Mutex<PeerManager>>,
    /// Gerenciador de sincronização do ledger
    sync_manager: Arc<Mutex<LedgerSyncManager>>,
    /// Configuração do serviço
    config: P2PConfig,
    /// Flag para controlar a execução do serviço
    running: Arc<Mutex<bool>>,
}

impl P2PService {
    /// Cria um novo serviço P2P
    pub fn new(
        identity: Identity,
        peer_manager: PeerManager,
        sync_manager: LedgerSyncManager,
        config: P2PConfig,
    ) -> Self {
        Self {
            identity,
            peer_manager: Arc::new(Mutex::new(peer_manager)),
            sync_manager: Arc::new(Mutex::new(sync_manager)),
            config,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Inicia o serviço P2P
    pub fn start(&self) -> Result<(), LastrumError> {
        // Marca o serviço como em execução
        let mut running = self.running.lock().unwrap();
        *running = true;
        drop(running);
        
        // Clone os arcs necessários para os threads
        let peer_manager = Arc::clone(&self.peer_manager);
        let sync_manager = Arc::clone(&self.sync_manager);
        let running = Arc::clone(&self.running);
        
        // Inicia o thread de escuta de conexões
        let listen_address = format!("0.0.0.0:{}", self.config.listen_port);
        let _listener_thread = thread::spawn(move || {
            match TcpListener::bind(&listen_address) {
                Ok(listener) => {
                    log::info!("Serviço P2P escutando em: {}", listen_address);
                    
                    // Configura o listener para não bloquear
                    listener.set_nonblocking(true).unwrap();
                    
                    // Loop principal do listener
                    while *running.lock().unwrap() {
                        // Tenta aceitar uma conexão
                        match listener.accept() {
                            Ok((stream, addr)) => {
                                log::info!("Nova conexão recebida de: {}", addr);
                                
                                // Cria clones dos gerenciadores para o thread de processamento
                                let peer_manager_clone = Arc::clone(&peer_manager);
                                let sync_manager_clone = Arc::clone(&sync_manager);
                                
                                // Cria um thread para processar a conexão
                                thread::spawn(move || {
                                    if let Err(e) = handle_incoming_connection(stream, addr, peer_manager_clone, sync_manager_clone) {
                                        log::error!("Erro ao processar conexão de {}: {}", addr, e);
                                    }
                                });
                            },
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // Nenhuma conexão para aceitar, espera um pouco
                                thread::sleep(Duration::from_millis(100));
                            },
                            Err(e) => {
                                log::error!("Erro ao aceitar conexão: {}", e);
                                // Pequena pausa para evitar consumo excessivo de CPU em caso de erro
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Falha ao iniciar serviço P2P: {}", e);
                }
            }
        });
        
        // Inicia o thread de descoberta de peers
        let bootstrap_nodes = self.config.bootstrap_nodes.clone();
        let peer_manager_disc = Arc::clone(&self.peer_manager);
        let running_disc = Arc::clone(&self.running);
        let refresh_interval = self.config.peer_refresh_interval;
        
        let _discovery_thread = thread::spawn(move || {
            // Conecta-se aos nós iniciais
            for node in bootstrap_nodes {
                if let Err(e) = connect_to_bootstrap_node(&node, Arc::clone(&peer_manager_disc)) {
                    log::error!("Erro ao conectar ao nó inicial {}: {}", node, e);
                }
            }
            
            // Loop principal de descoberta de peers
            while *running_disc.lock().unwrap() {
                // Aqui iria a lógica para descobrir novos peers e manter conexões existentes
                
                // Pausa entre atualizações
                thread::sleep(Duration::from_secs(refresh_interval));
            }
        });
        
        // Não bloqueia, permite que o serviço execute em background
        log::info!("Serviço P2P iniciado com sucesso");
        
        Ok(())
    }
    
    /// Para o serviço P2P
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        log::info!("Serviço P2P sendo encerrado");
    }
    
    /// Envia uma mensagem para um peer específico
    pub fn send_message_to_peer(&self, peer_id: &str, message: Message) -> Result<(), LastrumError> {
        let peer_manager = self.peer_manager.lock().unwrap();
        
        // Busca o peer pelo ID
        if let Some(peer) = peer_manager.get_peer_by_id(peer_id) {
            // Em uma implementação real, isso abriria uma conexão com o peer
            // e enviaria a mensagem serializada
            log::debug!("Enviando mensagem para peer {}: {:?}", peer.name, message.msg_type);
            
            // Serializa a mensagem
            let json = serde_json::to_string(&message)
                .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
            
            // Simula o envio
            log::debug!("Mensagem (simulação): {}", json);
            
            Ok(())
        } else {
            Err(LastrumError::NetworkError(format!("Peer não encontrado: {}", peer_id)))
        }
    }
    
    /// Envia uma mensagem em broadcast para todos os peers
    pub fn broadcast_message(&self, message: Message) -> Result<(), LastrumError> {
        let peer_manager = self.peer_manager.lock().unwrap();
        let peers = peer_manager.get_peers();
        
        // Serializa a mensagem (para logging)
        let _json = serde_json::to_string(&message)
            .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
        
        log::debug!("Broadcasting mensagem para {} peers: {:?}", peers.len(), message.msg_type);
        
        // Envia para cada peer
        for peer in peers {
            // Em uma implementação real, isso abriria uma conexão com cada peer
            // e enviaria a mensagem serializada
            log::debug!("Enviando para peer {}", peer.name);
        }
        
        Ok(())
    }
}

/// Processa uma conexão recebida
fn handle_incoming_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    peer_manager: Arc<Mutex<PeerManager>>,
    sync_manager: Arc<Mutex<LedgerSyncManager>>,
) -> Result<(), LastrumError> {
    // Em uma implementação real, isso leria a mensagem da stream,
    // desserializaria e a processaria adequadamente
    
    // Define um timeout para a leitura
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    
    // Buffer para leitura
    let mut buffer = [0; 4096];
    
    // Tenta ler da stream
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                // Converte o buffer para uma string
                let message_str = std::str::from_utf8(&buffer[0..bytes_read])
                    .map_err(|e| LastrumError::DeserializationError(e.to_string()))?;
                
                // Desserializa a mensagem
                let message: Message = serde_json::from_str(message_str)
                    .map_err(|e| LastrumError::DeserializationError(e.to_string()))?;
                
                log::debug!("Mensagem recebida de {}: {:?}", addr, message.msg_type);
                
                // Processa a mensagem de acordo com seu tipo
                match message.msg_type {
                    MessageType::Hello => {
                        // Extrai as informações do payload
                        if let Some(payload) = &message.payload {
                            let parts: Vec<&str> = payload.split(':').collect();
                            if parts.len() == 2 {
                                let name = parts[0].to_string();
                                let public_key = parts[1].to_string();
                                
                                // Cria um novo peer
                                let peer = Peer::new(
                                    message.sender.clone(),
                                    name,
                                    public_key,
                                    addr.to_string(),
                                );
                                
                                // Adiciona ao gerenciador de peers
                                let mut peer_manager = peer_manager.lock().unwrap();
                                peer_manager.add_peer(peer);
                                
                                // Envia uma resposta
                                let response = Message::new(
                                    MessageType::Pong,
                                    "self_id".to_string(), // Em uma implementação real, seria o ID do nó
                                    Some(message.sender.clone()),
                                    None,
                                );
                                
                                // Serializa e envia a resposta
                                let response_json = serde_json::to_string(&response)
                                    .map_err(|e| LastrumError::SerializationError(e.to_string()))?;
                                
                                stream.write_all(response_json.as_bytes())?;
                            }
                        }
                    },
                    MessageType::Certificate => {
                        // Processa o certificado recebido
                        let mut sync_manager = sync_manager.lock().unwrap();
                        sync_manager.process_message(message)?;
                    },
                    // Outros tipos de mensagens...
                    _ => {
                        log::debug!("Tipo de mensagem não tratado: {:?}", message.msg_type);
                    }
                }
            }
        },
        Err(e) => {
            return Err(LastrumError::NetworkError(format!("Erro ao ler da stream: {}", e)));
        }
    }
    
    Ok(())
}

/// Conecta-se a um nó inicial
fn connect_to_bootstrap_node(
    node_address: &str,
    peer_manager: Arc<Mutex<PeerManager>>,
) -> Result<(), LastrumError> {
    // Em uma implementação real, isso abriria uma conexão TCP com o nó
    // e trocaria mensagens para estabelecer a conexão
    
    log::info!("Conectando ao nó inicial: {}", node_address);
    
    // Simula a criação de um peer
    let peer = Peer::new(
        format!("simulated_id_{}", node_address.replace(":", "_")),
        format!("Bootstrap Node {}", node_address),
        "simulated_public_key".to_string(),
        node_address.to_string(),
    );
    
    // Adiciona ao gerenciador de peers
    let mut peer_manager = peer_manager.lock().unwrap();
    peer_manager.add_peer(peer);
    
    Ok(())
}