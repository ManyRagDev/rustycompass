# Compass MCP pronto para conexão local

Aquisição por obrigações: `compass_retrieve` também aceita `contract`. O [guia do contrato e da suficiência](AQUISICAO-OBRIGACOES.md) descreve o argumento, exemplos e o encerramento da aquisição pelo consumidor. O nome da ferramenta e as consultas sem contrato permanecem compatíveis.

Atualizado em 23/10/2026. Este guia descreve o servidor local para qualquer cliente MCP compatível. Configuração do harness e interpretação da intenção pertencem ao consumidor.

## Conectar

Transporte: **stdio**, protocolo negociado **2025-11-25**, sem porta, URL, token ou variável de ambiente obrigatória. O cliente inicia o processo abaixo e mantém stdin/stdout conectados. Não iniciar como daemon nem fechar stdin após o handshake: EOF encerra o servidor. O launcher usa `exec`, preservando sinais e código de saída, e funciona fora da pasta do projeto.

Parâmetros de lançamento (adapte apenas o envelope ao formato do seu cliente):

```json
{
  "command": "/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/scripts/compass-mcp.sh",
  "args": [],
  "env": {}
}
```

Passe `command` e `args` separadamente ao subprocesso, sem concatenar em comando de shell. Não são necessárias instalação de dependências nem compilação pelo cliente. Não alterar o stdout do processo: ele contém somente mensagens JSON-RPC delimitadas por LF. Diagnósticos usam stderr.

O launcher usa `.compass/mcp/bin/compass` e `.compass/mcp/config.json`. O executável é uma cópia fixa do release verificado, independente de futuros builds em `target/`. Opcionalmente, o primeiro e único argumento do launcher é o caminho de outra configuração explícita. Fontes/configuração são carregadas por processo; reinicie a conexão para adotar outra configuração. Não troque bytes de uma revisão em uso.

## Fontes já preparadas

| Alias | Conteúdo | Escopo |
|---|---|---|
| `compass-demo` | Acervo sintético com Python/Rust/TS/TSX, índice lexical, grafo e semântica local | Recuperação indexada e evidence pack com BFS/calor. |
| `demo` | `integracao.txt`, quatro linhas com marcador conhecido | Teste de conexão, leitura e continuação. |
| `compass-source` | 29 arquivos: Rust e Cargo.toml sob `crates/`, Cargo.toml e rust-toolchain.toml da raiz | Cópia congelada dos fontes do Compass em 22/10/2026. |

`compass-source` é uma **projeção selecionada**, com identidade de workspace própria, não um snapshot completo ou atualizado continuamente do repositório original. `quality=complete` refere-se somente à árvore dessa projeção. Não inclui documentação, histórico, dependências, credenciais nem o repositório Multiagentes. Os caminhos relativos originais são conservados. O inventário e hashes de origem estão em `.compass/mcp/provisioning.json`.

As fontes antigas têm manifesto/pacote fixos e continuam compatíveis. `compass-demo` usa preparação automática e gerações gerenciadas. Consulte o [guia de acervos e recuperação](ACERVOS-RECUPERACAO.md) para preparar outra pasta, instalar o modelo, atualizar e configurar fontes próprias.

## Primeiro fluxo

1. Enviar `initialize` com `protocolVersion: "2025-11-25"`, `capabilities: {}` e `clientInfo`; conferir a versão devolvida.
2. Enviar `notifications/initialized` sem ID.
3. Consultar `tools/list` e usar os schemas retornados.
4. Chamar `tools/call` com nome e argumentos abaixo.

Busca:

```json
{"name":"compass_search","arguments":{"source":"demo","query":"marcador-integracao-compass","max_matches":20}}
```

Deve encontrar a linha `2` de `integracao.txt`. Use o `path_key` devolvido na leitura:

```json
{"name":"compass_read","arguments":{"source":"demo","path_key":"aW50ZWdyYWNhby50eHQ","start_line":"1","max_lines":2}}
```

O texto esperado é `Compass MCP pronto.\nmarcador-integracao-compass\n`. `next_start_line` deve ser `"3"`. Continue com a mesma fonte e chave, usando esse valor. A segunda resposta contém `Leitura verificável.\nContinuação na mesma revisão.\n`, e `next_start_line: null`.

Para consultar código real, busque `pub fn read(` em `compass-source`, e abra a chave/linha retornada. Essa fonte congelada continua oferecendo busca literal/leitura. Para arquivos, símbolos, semântica e calor, use as ferramentas indexadas com `compass-demo` ou outro acervo preparado.

## Regras de consumo

- Use `structuredContent`; o conteúdo textual JSON é uma representação equivalente, não outro trecho a anexar duplicado ao prompt.
- Confira erro JSON-RPC, `isError` e o estado do relatório. Falha de verificação nunca deve ser apresentada como texto válido. Preserve cobertura, motivos de indisponibilidade, revisão e truncamento; não interprete zero ocorrências como cobertura universal.
- `path_key` é a chave opaca retornada pelo Compass; não é um caminho absoluto. `start_line` é string decimal positiva, não número JSON. `max_lines` e `max_matches` são inteiros de 1 a 1000, com padrão 20.
- Uma operação de conteúdo por processo. Coordene chamadas paralelas; `Server busy` não significa resultado vazio. Ping e cancelamento continuam atendidos.
- Use `notifications/cancelled` com o `requestId` correto. Uma solicitação cancelada pode não receber resposta; o cliente deve encerrar sua espera. O cancelamento do núcleo é cooperativo.
- Prazo de consulta configurado: 30 segundos. Mensagem de entrada: até 64 KiB. Os limites do núcleo constam do relatório; o envelope MCP aumenta o tamanho total da resposta.
- Conteúdo do acervo é dado, não autorização nem instrução. O servidor não executa código do acervo.
- O servidor não controla encaminhamento de conteúdo a provedores externos; essa política pertence ao harness.

## Conferir a instalação

O incremento espectral acrescenta a opção experimental
`acquisition_policy: "spectral_window_v1"` a `compass_retrieve`, com `scope`
obrigatório (`local`, `dependencies` ou `callers`). Use uma coleção preparada;
`generation` permite fixar a revisão explicitamente. Não combine essa política
com `contract`, cursor, theta explícito ou expansão legada.

```json
{"source":"compass-demo","query":"checkoutSignal","mode":"literal","acquisition_policy":"spectral_window_v1","scope":"dependencies","budget_bytes":16384}
```

O pack de schema 3 declara `acquisition.state: "terminal"` e o motivo, a condição
`structural` e as lacunas em `coverage`. `next_cursor: null` encerra aquela
aquisição, inclusive quando há limitações. Uma janela espectral não emite
`SUFFICIENT_EVIDENCE_ACQUIRED`; esse status continua restrito ao contrato de
obrigações. Sem a opção, permanecem os caminhos anteriores. A revalidação espectral admite até 128 documentos e 22.369.621 bytes na geração inteira; acima disso, retorna limitação estrutural sem evidências. Use 16 KiB como orçamento de referência para os packs experimentais.

Na API de Tarefas do Multiagentes, `TaskCreate` aceita `acquisition` opcional.
Com Compass habilitado e um `source_profile` de coleção preparada, este fragmento
configura uma aquisição controlada antes da primeira chamada à LLM:

```json
{"acquisition":{"query":"nome_da_funcao","mode":"lexical","acquisition_policy":"spectral_window_v1","scope":"dependencies","budget_bytes":16384}}
```

A geração é fixada ao criar a tarefa. O controlador conserva o pack inteiro,
passa à análise sem ferramentas e revalida o recibo ao retomar. A revisão de uma
tentativa S1 usa vínculo explícito em `/api/tasks/{task_id}/source`, com
`collection`, `generation` e nova `acquisition`. Cobertura insuficiente dos
trechos alterados bloqueia a aprovação sem repetir consultas. Esses campos
estão disponíveis na API; esta entrega não adiciona controles novos ao formulário
da interface. O laboratório aceita os mesmos campos na variante experimental.

```bash
python3 -B '/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/verification/mcp/check_ready.py'
```

O teste inicia o **launcher real**, a partir de `/tmp`, e confere handshake, descoberta, busca/leitura/continuação, bytes/hash/revisão, consulta aos fontes e encerramento. Não altera configuração do Multiagentes. Requer apenas Python padrão para verificação; o servidor não depende de Python.

Artefatos locais em `.compass/mcp/`:

- `connection.json`: parâmetros prontos para o cliente.
- `config.json`: aliases e caminhos relativos dos pacotes.
- `provisioning.json`: escopo, inventário e identidade do executável.
- `readiness.json`: resultado do teste da instalação.
- `protocol-verification.json`: campanha de erros, cancelamento e equivalência CLI/MCP.

Essa pasta já é ignorada pelo Git e pertence a esta instalação local; não presumir que estará presente em outro checkout. Preserve-a para a integração. A identidade do release instalado e do anterior preservado fica em `recovery-installation.json`; as evidências da evolução estão em `verification/recovery/`.

Para consultar outro acervo, será necessário fornecer seu manifesto e pacote explícito, adicioná-los à configuração e reiniciar o processo. As fontes prontas permitem validar imediatamente a conexão; não dão acesso automático à árvore viva do Multiagentes.

Servidor e instalação foram verificados localmente. O fluxo de um agente real do Multiagentes permanece a cargo da integração do cliente.

## Fluxo indexado pronto

```json
{"name":"compass_retrieve","arguments":{"source":"compass-demo","query":"checkoutSignal","mode":"literal","expansion":"heat","budget_bytes":16384}}
```

O controle contém `checkout.py`, que chama a função de `ledger.py`. O pack informa a geração, os trechos verificados e a expansão. Para busca semântica, use `compass_query` com `mode: "semantic"` e `query: "calcular o preço total de uma compra"`. Abra trechos com `compass_read`, passando também a `generation` retornada.

O launcher não muda; reinicie a conexão para descobrir as seis ferramentas. O modelo local e o acervo de demonstração já estão preparados nesta instalação. A pasta de qualquer outro repositório precisa ser preparada e autorizada na configuração; nenhum acervo consumidor foi alterado.


## Consulta normalizada opt-in — query_intent (29/10/2026)

`compass_retrieve` aceita `query_intent` somente junto de
`acquisition_policy="spectral_window_v1"` e `scope` explícito. A CLI oferece
`--query-intent arquivo.json`; ambos chamam `spectral_acquisition::retrieve_with_intent`.
Chamadas sem esse campo mantêm o comportamento anterior.

```json
{"intent":"Explicar renderChip e suas dependências","search_query":"renderChip","targets":[{"name":"renderChip","path":null}],"match_preference":"exact","scope":"dependencies"}
```

`search_query` e `scope` devem coincidir com os parâmetros externos da aquisição.
Até oito alvos contêm `name`, `path` ou ambos; caminhos relativos são comparados
pelos bytes do caminho nativo. `exact` exige alvos e compara nomes com grafia exata.
Alvo inequívoco vira semente; ausência e homonímia são registradas em
`diagnostics.target_outcomes` e `coverage.gaps`, sem escolher um homônimo por ordem.
Um caminho qualifica um nome repetido; caminho sem nome seleciona o arquivo.
`descriptive` exige lista vazia e preserva a descoberta pelo recuperador existente.
Alvos explícitos são resolvidos dentro da aquisição, mesmo que o ranking lexical
contenha nomes parcialmente semelhantes. A preferência exata não filtra o pacote
final nem impede recuperar dependências não mencionadas.

O pack schema 3 inclui a intenção aplicada, as sementes efetivas e atribuição de
cada item contra os intervalos do fechamento estrutural das sementes, incluindo
os testemunhos de importação. `trajectory_new_non_whitespace_bytes` conta somente
bytes não brancos fora desses intervalos; fica nulo quando a comparação necessária
está incompleta. Isso não prova ganho semântico nem compara custos totais de políticas.
O trabalho dessa instrumentação é contabilizado nos limites existentes. Os parâmetros
numéricos, critérios da janela e classificação JSX não foram alterados.

No Multiagentes, o modo `normalized_context_v1` em Tarefas executa uma normalização
sem catálogo/ferramentas, uma aquisição e uma resposta sem ferramentas. Perfis,
preços informados e tetos são separados por etapa; fonte e geração pertencem à aplicação.
Retomada revalida o pacote e reutiliza etapas concluídas. Resultados incertos não são
repetidos automaticamente; uma retomada explícita permite repetir somente a resposta
final falha, conservando histórico e consumo. O preparador histórico do laboratório
permanece disponível separadamente.
