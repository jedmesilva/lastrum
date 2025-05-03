# Lastrum Certifield

Um sistema descentralizado para casas de custódia emitirem e validarem certificados para ativos físicos (metais preciosos e energias renováveis).

## Visão Geral

Lastrum Certifield é uma plataforma descentralizada para a emissão e verificação de certificados de ativos físicos. Utilizando tecnologia de assinatura digital, permite que casas de custódia emitam certificados para seus ativos e que esses certificados sejam verificados por qualquer participante da rede. O sistema suporta diversos tipos de ativos, incluindo metais preciosos (ouro, prata, platina e paládio) e certificados de energia renovável (solar, eólica, hídrica, biogás e hidrogênio verde).

## Características

- **Geração de Identidade**: Cada casa de custódia gera sua própria identidade e chaves criptográficas
- **Emissão de Certificados**: Criação de certificados assinados digitalmente para ativos físicos
- **Verificação de Certificados**: Validação criptográfica da autenticidade dos certificados
- **Rede Descentralizada**: Comunicação P2P entre nós sem autoridade central
- **Consenso Automático**: Validação automática através de consenso sem intervenção humana
- **Suporte Multi-Ativos**: Certificação de metais preciosos e energias renováveis
- **Tokenização**: Capacidade de emitir tokens representando frações dos ativos certificados
- **Migração de Certificados**: Compatibilidade com formatos legados e conversão automática
- **Governança Descentralizada**: Sistema de propostas e votação para novos tipos de ativos
- **Extensibilidade**: Mecanismo democrático para adicionar novas categorias de ativos

## Mecanismo de Consenso

O Lastrum implementa um mecanismo de consenso especializado para evitar discriminação entre casas de custódia:

1. Certificados são assinados digitalmente pelo emissor usando criptografia Ed25519
2. Quando um certificado é propagado na rede, 51% dos nós são selecionados aleatoriamente para validação
3. A validação é puramente matemática: verificação da assinatura digital
4. Não existe possibilidade de rejeição arbitrária; a validação é determinística
5. Certificados validados são armazenados no ledger compartilhado

## Uso

### Geração de Identidade

```bash
lastrum_certifield generate-identity --name "Nome da Casa de Custódia"
```

### Emissão de Certificado

#### Para Metais Preciosos (com Pureza)

```bash
lastrum_certifield issue-certificate \
    --identity /caminho/para/identidade.json \
    --asset-type GOLD \
    --weight 1.0 \
    --purity 0.9999 \
    --serial "GB1234567890"
```

#### Para Energia Renovável

```bash
lastrum_certifield issue-certificate \
    --identity /caminho/para/identidade.json \
    --asset-type ENERGIA_SOLAR \
    --weight 1000.0 \
    --serial "SOLAR-BR-1000-2025"
```

### Verificação de Certificado

```bash
lastrum_certifield verify-certificate --certificate "id-do-certificado"
```

### Listagem de Certificados

```bash
lastrum_certifield list-certificates
```

### Iniciando um Nó na Rede

```bash
lastrum_certifield start-node --identity /caminho/para/identidade.json --port 8080
```

### Sincronização de Certificados

```bash
lastrum_certifield sync-certificates
```

### Governança Descentralizada

#### Propor Novo Tipo de Ativo

```bash
lastrum_certifield propose-asset-type \
    --identity /caminho/para/identidade.json \
    --code "CPU_TIME" \
    --category "TIME" \
    --name "Horas de Computação" \
    --unit "h" \
    --requires-purity \
    --voting-period-days 7
```

#### Votar em uma Proposta

```bash
lastrum_certifield vote-asset-proposal \
    --identity /caminho/para/identidade.json \
    --proposal-id "id-da-proposta" \
    --approve
```

#### Listar Propostas Pendentes

```bash
lastrum_certifield list-asset-proposals
```

#### Listar Tipos de Ativos Registrados

```bash
lastrum_certifield list-asset-types
```

#### Verificar Propostas Expiradas

```bash
lastrum_certifield check-expired-proposals
```

## Tipos de Ativos Suportados

### Metais Preciosos
- **GOLD**: Ouro (requer pureza)
- **SILVER**: Prata (requer pureza)
- **PLATINUM**: Platina (requer pureza)
- **PALLADIUM**: Paládio (requer pureza)

### Energias Renováveis
- **ENERGIA_SOLAR**: Certificados de energia solar
- **ENERGIA_EOLICA**: Certificados de energia eólica
- **ENERGIA_HIDRICA**: Certificados de energia hídrica
- **BIOGAS_METANO**: Certificados de biogás/metano
- **HIDROGENIO_VERDE**: Certificados de hidrogênio verde

### Tempo de Uso
- **TEMPO_VEICULO**: Certificados de tempo de uso de veículos (horas)
- **TEMPO_IMOVEL**: Certificados de tempo de uso de imóveis (dias)

## Tecnologias

- Linguagem Rust para segurança e performance
- Criptografia assimétrica Ed25519 para assinaturas digitais
- Comunicação P2P para descentralização
- Armazenamento local em SQLite para persistência
- Sistema de tokens para representação fracionada de ativos
- Mecanismo de migração para compatibilidade de formatos

## Estrutura do Projeto

- `core/`: Componentes principais (identidade, chaves, assinaturas)
- `certifield/`: Gerenciamento de certificados (modelo, construtor, armazenamento)
- `network/`: Infraestrutura de rede descentralizada (P2P, consenso, sincronização)
- `governance/`: Sistema de governança descentralizada (propostas, votação, registro)
- `utils/`: Utilitários comuns
- `config/`: Configurações do sistema
- `errors/`: Definições de erro

## Armazenamento Local

Os dados são armazenados localmente em:

- **Certificados**: `/home/USER/.local/share/certifield/certificates.db` (SQLite)
- **Identidades**: `/home/USER/.local/share/certifield/identities/` (arquivos JSON)
- **Tipos de Ativos**: `/home/USER/.local/share/certifield/asset_registry.json` (JSON)
- **Propostas de Governança**: `/home/USER/.local/share/certifield/asset_proposals.json` (JSON)

## Fluxo de Certificação e Validação

1. Casa de custódia gera sua identidade única
2. Casa de custódia emite certificado para um ativo físico, assinando-o com sua chave privada
3. Certificado é armazenado localmente e pode ser propagado na rede
4. Qualquer participante pode verificar a autenticidade do certificado localmente
5. Quando necessário, tokens podem ser emitidos representando frações do ativo certificado

## Fluxo de Governança

1. Casa de custódia propõe um novo tipo de ativo através do comando `propose-asset-type`
2. A proposta entra em período de votação (definido em dias)
3. Outras casas de custódia votam na proposta (approve/reject) via `vote-asset-proposal`
4. Ao final do período de votação, ou quando alcançar consenso:
   - Se aprovada (>50% dos votos), o tipo de ativo é adicionado ao registro
   - Se rejeitada, a proposta é arquivada
5. Novos certificados podem ser emitidos utilizando o tipo de ativo aprovado