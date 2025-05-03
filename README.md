# Lastrum Certifield

Um sistema descentralizado para casas de custódia emitirem e validarem certificados para ativos físicos (metais preciosos, energias renováveis e outros recursos).

## Visão Geral

Lastrum Certifield é uma plataforma descentralizada para a emissão e verificação de certificados de ativos físicos. Utilizando tecnologia de assinatura digital e criptografia avançada, permite que casas de custódia emitam certificados para seus ativos e que esses certificados sejam verificados por qualquer participante da rede. O sistema opera sem autoridade central, utilizando um mecanismo de consenso especializado onde 51% dos nós selecionados aleatoriamente validam certificados e propostas.

O sistema suporta diversos tipos de ativos, incluindo metais preciosos (ouro, prata, platina, paládio, cobre), certificados de energia renovável (solar, eólica, hídrica, biogás e hidrogênio verde), e pode ser estendido para novas categorias através do mecanismo de governança descentralizada.

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

## O Papel das Casas de Custódia

No ecossistema Lastrum, as casas de custódia são entidades que mantêm ativos físicos sob sua guarda e emitem certificados digitais que representam esses ativos. Cada casa de custódia:

1. **Gera sua identidade digital** com chaves criptográficas públicas e privadas
2. **Emite certificados** para os ativos sob sua custódia, assinando-os digitalmente
3. **Participa da governança da rede** propondo novos tipos de ativos e votando em propostas
4. **Mantém operações de nó** na rede P2P para validar certificados de outras casas
5. **Gerencia o ciclo de vida dos certificados** incluindo emissão, atualização e revogação

As casas de custódia formam coletivamente uma Organização Autônoma Descentralizada (DAO), onde decisões são tomadas por meio de votação criptograficamente verificável.

## Sistema de Votação Segura

O Lastrum implementa um sistema de votação criptograficamente seguro:

1. Cada voto é assinado com a chave privada da casa de custódia
2. A assinatura é verificada matematicamente para validar a autenticidade do voto
3. Os votos são imutáveis após assinados e não podem ser alterados
4. A identidade do votante é verificada pela sua chave pública
5. O sistema mantém registros auditáveis de todas as votações

Este mecanismo garante integridade total no processo de governança, tornando impossível a falsificação de votos.

## Mecanismo de Consenso

O Lastrum implementa um mecanismo de consenso especializado para evitar discriminação entre casas de custódia:

1. Certificados são assinados digitalmente pelo emissor usando criptografia Ed25519
2. Quando um certificado é propagado na rede, 51% dos nós são selecionados aleatoriamente para validação
3. A validação é puramente matemática: verificação da assinatura digital
4. Não existe possibilidade de rejeição arbitrária; a validação é determinística
5. Certificados validados são armazenados no ledger compartilhado
6. O mesmo método é aplicado para validação de propostas de novos tipos de ativos

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

O sistema suporta ativos de diversas categorias, e novos tipos podem ser adicionados através do mecanismo de governança descentralizada:

### Metais Preciosos e Industriais
- **GOLD**: Ouro (requer pureza)
- **SILVER**: Prata (requer pureza)
- **PLATINUM**: Platina (requer pureza)
- **PALLADIUM**: Paládio (requer pureza)
- **COPPER**: Cobre (requer pureza)
- **LITHIUM**: Lítio (requer pureza)

### Energias Renováveis
- **ENERGIA_SOLAR**: Certificados de energia solar
- **ENERGIA_EOLICA**: Certificados de energia eólica
- **ENERGIA_HIDRICA**: Certificados de energia hídrica
- **BIOGAS_METANO**: Certificados de biogás/metano
- **HYDRO**: Hidrogênio Verde (requer pureza)

### Tempo de Uso
- **TEMPO_VEICULO**: Certificados de tempo de uso de veículos (horas)
- **TEMPO_IMOVEL**: Certificados de tempo de uso de imóveis (dias)
- **CPU_TIME**: Tempo de processamento computacional (horas)

### Recursos Naturais
- **CARBON_CREDITS**: Créditos de carbono
- **WATER_RIGHTS**: Direitos de água

## Segurança Criptográfica

O Lastrum utiliza diversos mecanismos de segurança criptográfica:

1. **Chaves Assimétricas Ed25519**: Para assinaturas digitais de alta segurança e velocidade
2. **Verificação de Assinaturas**: Validação matemática da autenticidade dos certificados e votos
3. **Hashing Criptográfico**: Para integridade de dados e vinculação de certificados
4. **Autenticação de Identidade**: Verificação criptográfica da identidade de casas de custódia
5. **Assinatura de Votos**: Sistema de votação com verificação criptográfica
6. **Proteção contra Adulteração**: Dados assinados são imutáveis e qualquer modificação invalida a assinatura

Estes mecanismos garantem a integridade e autenticidade de todas as operações no sistema.

## Tecnologias

- Linguagem Rust para segurança e performance
- Criptografia assimétrica Ed25519 para assinaturas digitais
- Hash criptográfico SHA-256 para integridade dos dados
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
4. Cada voto é criptograficamente assinado para garantir autenticidade e não-repúdio
5. Ao final do período de votação, ou quando alcançar consenso:
   - Se aprovada (>50% dos votos), o tipo de ativo é adicionado ao registro
   - Se rejeitada, a proposta é arquivada
6. Novos certificados podem ser emitidos utilizando o tipo de ativo aprovado

## Sistema de Votação Criptograficamente Assinado

O Lastrum implementa um sistema avançado de votação criptograficamente assinado que garante a integridade do processo de governança:

1. **Estrutura de Voto Segura**: Cada voto é armazenado em uma estrutura `SignedVote` que contém:
   - O valor do voto (approve/reject)
   - O ID da casa de custódia votante
   - Um hash criptográfico dos dados do voto
   - Uma assinatura digital criada com a chave privada da casa de custódia

2. **Processo de Votação**:
   - A casa de custódia assina o voto com sua chave privada
   - O sistema verifica a assinatura usando a chave pública da casa de custódia
   - Apenas votos com assinaturas válidas são registrados e contabilizados
   - O sistema impede votos duplicados da mesma casa de custódia

3. **Verificação Criptográfica**:
   - Cada assinatura é verificada matematicamente usando o algoritmo Ed25519
   - Qualquer tentativa de manipulação ou falsificação de voto é detectada e rejeitada
   - O histórico de votação é à prova de adulteração e auditável

4. **Finalização de Propostas**:
   - As propostas são finalizadas automaticamente quando atingem maioria (>50%)
   - Ou quando expira o período de votação definido durante a criação da proposta

Este sistema garante que o processo de governança seja totalmente transparente, auditável e matematicamente verificável, evitando qualquer possibilidade de manipulação ou fraude.