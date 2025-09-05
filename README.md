# LoadForge Worker-WebSocket

O Worker-WebSocket é um servidor WebSocket desenvolvido em Rust usando o framework Actix-Web, projetado para gerenciar e executar testes de carga de forma assíncrona e em tempo real.

## Características Principais

- Interface WebSocket para controle remoto de testes de carga
- Suporte a múltiplos métodos HTTP (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- Autenticação via token de segurança
- Monitoramento em tempo real de métricas
- Limitação de conexões simultâneas para controle de recursos

## Requisitos

- Rust 1.65 ou superior
- OpenSSL (para suporte a HTTPS)
- Variáveis de ambiente configuradas

## Instalação

1. Clone o repositório:
   ```bash
   git clone https://github.com/loadforge/worker-websocket.git
   cd worker-websocket
   ```

2. Crie um arquivo `.env` baseado no exemplo:
   ```bash
   cp .env.example .env
   ```
   Edite o arquivo `.env` e defina um token secreto seguro.

3. Construa o projeto:
   ```bash
   cargo build --release
   ```

4. Execute o servidor:
   ```bash
   ./target/release/worker-websocket
   ```

## Configuração

### Variáveis de Ambiente

Crie um arquivo `.env` na raiz do projeto com:

```env
WS_SECRET_TOKEN=seu_token_seguro_aqui
```

## Uso

### Conexão WebSocket

Conecte-se ao WebSocket em:
```
ws://localhost:8080/ws?token=seu_token_seguro_aqui
```

### Comandos Suportados

#### Iniciar Teste de Carga
```json
{
  "type": "start_test",
  "config": {
    "name": "Teste de Carga API",
    "target": "https://api.exemplo.com/endpoint",
    "method": "POST",
    "concurrency": 10,
    "duration": 300,
    "headers": {
      "Content-Type": "application/json"
    },
    "body": {
      "chave": "valor"
    },
    "auth": {
      "type": "bearer",
      "token": "seu_token_aqui"
    }
  }
}
```

#### Parar Teste
```json
{
  "type": "stop_test"
}
```

### Respostas do Servidor

#### Atualização de Métricas
```json
{
  "type": "metrics_update",
  "metrics": {
    "total_requests": 1000,
    "successful_requests": 980,
    "failed_requests": 20,
    "fastest_response_ms": 12.5,
    "slowest_response_ms": 1200.0,
    "average_response_ms": 45.3,
    "requests_per_second": 50.2,
    "status_codes": {
      "200": 980,
      "500": 20
    }
  }
}
```

## Segurança

- Todas as conexões WebSocket devem incluir um token de autenticação válido
- O token deve ser configurado na variável de ambiente WS_SECRET_TOKEN
- Apenas uma conexão ativa por vez é permitida por padrão

## Limitações

- Suporta apenas uma conexão WebSocket ativa por vez
- O número máximo de requisições simultâneas é limitado pelos recursos do sistema
- Não há persistência de dados entre reinicializações do servidor

## Solução de Problemas

### Erro de Conexão
- Verifique se a porta 8080 está disponível
- Confirme se o token de autenticação está correto
- Verifique os logs do servidor para mensagens de erro

### Autenticação Falhando
- Confirme se o token na URL corresponde ao configurado no .env
- Verifique se o token está sendo enviado corretamente como parâmetro de consulta

## Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para abrir issues e enviar pull requests.

## Licença

Este projeto está licenciado sob a [Licença MIT](LICENSE).