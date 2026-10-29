# Contrato proposto de estados e revisões do Compass Foundation

> Registro histórico da proposta anterior à implementação. O [contrato ativo 0.1.0](../../compass-foundation/research/contracts/FOUNDATION.md) e o [estado operacional](../../compass-foundation/COMPASS_STATE.md) incorporam as decisões e verificações da primeira entrega autorizada. O texto abaixo é preservado para rastrear a proposta.

Proposta 0.1, de 14 de outubro de 2026, para revisão de Emanuel. Concretiza as seções 7, 17 e 20 do [documento principal](../../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md). As regras abaixo são escolhas propostas nesta análise; não são uma descrição de software existente nem uma alteração automática das decisões da edição 1.0.

## 1. Capacidade entregue

Registrar uma observação de um acervo local e comparar duas observações persistidas, informando diferenças, revisões, abrangência e desconhecimento. A CLI e os testes usarão o mesmo núcleo Rust.

O primeiro alvo de validação proposto é Linux. O domínio não depende de parsing nem de linguagem de programação: arquivos regulares são lidos como bytes, incluindo vazios e binários. Windows e macOS exigem contratos e testes próprios de filesystem antes de receberem a mesma alegação de suporte.

## 2. Objetos e identidades

| Objeto | Regra proposta |
|---|---|
| Acervo | Uma raiz autorizada, identificada por `workspace_id` UUID v4 persistido em `.compass/workspace.json`. Inicialização repetida preserva o ID; arquivo existente inválido causa erro. Cópias desse arquivo não constituem prova criptográfica de origem. |
| Caminho | Sequência relativa de componentes nativos, sem raiz absoluta, NUL, `.` ou `..`. No Linux, preservar bytes e maiúsculas/minúsculas; separar componentes por `/`. Serializar a chave como base64url sem padding. Uma representação legível é auxiliar e nunca serve para igualdade. |
| Revisão de conteúdo | `sha256` dos bytes efetivamente lidos e aceitos, em hexadecimal minúsculo, acompanhado do tamanho em bytes. Não normalizar BOM, Unicode, CRLF/LF, espaços ou conteúdo binário. |
| Estado de arquivo | Caminho, revisão e indicador de executabilidade: verdadeiro quando qualquer bit `0o111` estiver presente. Ownership, mtime e demais permissões são observações operacionais, não equivalência de conteúdo. |
| Política | Conjunto versionado de inclusão, exclusão, leitura, limites e interpretação. `policy_id` é hash da forma canônica dessa configuração efetiva. |
| Snapshot | Manifesto de uma observação, com `snapshot_id` UUID v4, acervo, política, versão do observador, janela, entradas e cobertura. O UUID identifica a observação; não é o hash do estado. |
| Resumo do estado | `state_digest` determinístico para o conjunto de arquivos elegíveis plenamente observados. Parcialidade implica `null`; não se inventa digest de um estado completo desconhecido. |

Hardlinks têm identidades de caminho distintas. Dois arquivos iguais podem compartilhar revisão de conteúdo sem se tornarem o mesmo arquivo. Repetir uma observação inalterada gera outro `snapshot_id` e conserva `state_digest`.

## 3. Universo elegível

A política padrão proposta inclui arquivos regulares sob a raiz. Não interpreta automaticamente `.gitignore`: a existência de Git não altera silenciosamente o universo. Exclusões adicionais usam caminhos relativos exatos ou prefixos de diretórios, sem glob na versão inicial, e integram o `policy_id`.

São podados diretórios cujo componente seja `.git`, `.compass`, `.aws` ou `.ssh`. Arquivos com basename `.env` ou começando por `.env.` são excluídos. Essas regras são visíveis no manifesto. Não são um detector completo de segredos: o operador escolhe uma raiz autorizada e exclusões adicionais pertinentes. Diretórios de dependências/build, como `node_modules` e `target`, podem ser excluídos explicitamente; não presumir que todo nome de diretório conhecido seja irrelevante.

Symlinks são registrados como `excluded/symlink_not_followed`; o alvo não é aberto nem percorrido. FIFOs, sockets e dispositivos são registrados como `excluded/unsupported_kind`, sem leitura. Uma exclusão por diretório registra a fronteira podada, sem fingir inventário de seus descendentes. Diretórios vazios são registrados para cobertura, mas não compõem o estado de conteúdo.

A raiz é resolvida e fixada antes da observação. A travessia e abertura precisam impedir escape por links em qualquer componente e por substituição concorrente. A seleção concreta de APIs Linux pertence ao adaptador de filesystem e exige teste de corrida; uma sequência simples de `symlink_metadata` e `open` não satisfaz essa garantia. [Documentação Rust sobre TOCTOU](https://doc.rust-lang.org/std/fs/index.html)

## 4. Estados de observação

| Estado da entrada | Significado | Pode receber revisão aceita? |
|---|---|---|
| `observed` | Arquivo regular lido integralmente e sem instabilidade detectada nas verificações previstas | Sim |
| `excluded` | Fronteira ou entrada fora do universo pela política, link ou tipo não suportado | Não |
| `unreadable` | A entrada foi identificada, mas o conteúdo não pôde ser lido | Não |
| `unstable` | Mudança, substituição ou desaparecimento detectado durante observação | Não |
| `limit_exceeded` | Uma restrição de leitura impediu completar a observação | Não |

Diretórios recebem cobertura `complete`, `unreadable`, `unstable` ou `not_visited`. Cobertura descreve uma enumeração concluída sem instabilidade detectada, não uma fotografia atômica. Uma fronteira excluída é registrada separadamente; não é um erro de cobertura do universo contratado.

Entrada não encontrada na lista não implica ausência. Para afirmar ausência, é necessário encontrar a evidência de enumeração correspondente, inclusive no primeiro ancestral ausente. A falta dessa evidência produz `unknown` na comparação. Se um diretório inteiro não puder ser lido, o relatório preserva essa fronteira mesmo sem conhecer seus arquivos.

O snapshot tem `quality=complete` quando todas as regiões elegíveis foram percorridas e todos os arquivos elegíveis identificados foram observados. Qualquer falha ou interrupção por limite nessa área torna a observação `partial`. Falha que impede identificar a raiz ou interpretar sua configuração encerra o comando com erro, sem publicar snapshot.

## 5. Consistência e leitura concorrente

`consistency=observation_window`: o manifesto representa leituras feitas entre `started_at` e `finished_at`. Os arquivos não são declarados simultâneos. O hash identifica a sequência de bytes lida, não prova que essa sequência inteira existiu no disco em um único instante durante escrita concorrente.

O observador deve:

1. Enumerar cada diretório elegível, guardando identidade e conjunto de nomes; reenumerar ao concluir a região e comparar. Diferenças tornam essa cobertura instável.
2. Abrir cada arquivo dentro da raiz, manter o descritor e registrar identidade, tamanho, timestamps de modificação/mudança e executabilidade antes da leitura.
3. Calcular o hash por streaming. Conferir os mesmos metadados após leitura, contagem efetiva de bytes e correspondência do nome com o objeto aberto.
4. Se detectar mudança, permitir no máximo uma nova tentativa; duas tentativas no total. Se a segunda falhar ou permanecer instável, registrar o estado apropriado e não aceitar o hash parcial.
5. Registrar a ocorrência de nova tentativa. Um eventual sucesso posterior é uma observação daquela tentativa; não reconstrói um instante global.

Comparar metadados antes/depois ajuda a detectar mudanças comuns, mas não prova imutabilidade diante de todas as alterações concorrentes. Nem mesmo duas leituras iguais transformam a árvore viva em snapshot atômico. Uma garantia mais forte exigirá fonte imutável, snapshot do filesystem ou coordenação com escritores em outro contrato.

O baseline nunca avança automaticamente. Comparações identificam explicitamente os dois snapshots e não reescrevem registros antigos.

## 6. Limites de recursos

O primeiro observador usa um leitor de arquivo por vez e buffer de streaming limitado. A configuração registra `max_file_bytes`, `max_total_bytes_read`, `max_entries` e `max_duration_ms`; cada campo aceita inteiro positivo ou `null` para ausência de limite. Os valores operacionais serão escolhidos com a carga de referência, sem herdar limites térmicos do legado.

Ultrapassar um limite de arquivo torna a entrada `limit_exceeded`. Limite global encerra a observação como parcial e registra as regiões não visitadas. Bytes de tentativas repetidas contam no total de leitura. A comparação offline não depende da duração registrada nem de relógio para decidir mudanças.

Sem limites configurados, não se anuncia memória, duração ou disco máximos garantidos: o manifesto e sua ordenação ainda crescem com o número de entradas. Foundation mede esse custo antes da definição de limites para release.

## 7. Persistência e formato

Propõe-se `.compass/workspace.json` para identidade, `.compass/policy.json` para configuração e `.compass/snapshots/<snapshot_id>.json` para manifestos. Somente `init` cria o acervo; `snapshot` exige inicialização existente. A comparação recebe arquivos de snapshot explícitos e funciona sem acessar a árvore original.

Um snapshot contém, no mínimo, os campos obrigatórios abaixo. O schema de transporte e as fixtures de serialização devem ser congelados em F0, antes da persistência; esta proposta não equivale a um JSON Schema já validado.

| Grupo | Campos |
|---|---|
| Versões | `schema_version`, `contract_version`, `producer_version`, `producer_build` |
| Identidade | `workspace_id`, `snapshot_id`, `path_namespace=linux_bytes_v1` |
| Política | `policy`, `policy_id` |
| Observação | `started_at`, `finished_at`, `consistency`, `quality` |
| Dados | `entries`, `directory_coverage`, `issues`, `state_digest` |
| Contadores | Arquivos aceitos, excluídos e desconhecidos; bytes efetivamente lidos; tentativas; duração |

Cada entrada tem chave de caminho, tipo observado, estado e motivo quando necessário. Uma entrada `observed` inclui `content_sha256`, `size_bytes` e `executable`; nos demais estados esses campos de revisão são ausentes. Uma entrada aceita pode ter diagnóstico de tentativa anterior, sem herdar hash daquela tentativa. Eventos de diagnóstico são ordenados por caminho e ordinal local, não pela ordem de chegada de threads.

Inteiros potencialmente grandes, como tamanhos e timestamps de filesystem, são strings decimais no transporte. Timestamps de janela são UTC com offset explícito. Rejeitar chaves duplicadas, caminhos inválidos, revisões malformadas, entradas duplicadas, schema desconhecido e combinações de estado/campos incompatíveis.

JSON canônico segue JCS, RFC 8785, para os objetos usados em hashes e a saída determinística. O formato prevê representação canônica para operações como hash e ordenação determinística de propriedades; a implementação escolhida deve ser conferida com vetores de referência. [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)

Definições propostas:

- `policy_id = SHA256("compass-policy-v1\0" || JCS(policy))`.
- `state_digest = SHA256("compass-state-v1\0" || JCS({workspace_id, path_namespace, policy_id, files}))`, somente quando `quality=complete`.
- `files` contém apenas `{path_key, content_sha256, size_bytes, executable}` de entradas aceitas, ordenadas pelos bytes decodificados do caminho.
- UUIDs, hashes e nomes de campo têm representação canônica; a janela, o `snapshot_id`, o caminho absoluto da máquina, duração e mtime não entram em `state_digest`.

Snapshot completo pode ter exclusões conhecidas. O digest cobre o universo elegível, não a árvore inteira. A leitura de um manifesto deve validar a estrutura e recalcular o digest quando aplicável; hashes não autenticam autoria nem protegem contra alteração maliciosa de todo o registro.

Publicação: validar, escrever temporário no mesmo filesystem, sincronizar e publicar sem substituir um destino existente. Serializar escritores por lock do sistema operacional, liberado ao encerrar o processo; não usar existência de PID como prova de posse. Falha ou cancelamento não pode deixar JSON truncado com nome final. Reabertura deve ignorar temporários e rejeitar registros inválidos.

Os manifestos não armazenam o conteúdo dos arquivos. Consequentemente, não permitem reconstruir os bytes antigos nem calcular diferenças de linhas depois que a fonte mudou. Antes de D0 textual, haverá decisão explícita sobre blobs locais, retenção, custo e privacidade; snapshots antigos sem bytes não se tornam fontes completas retroativamente.

## 8. Comparação

Pré-condições: versões suportadas e mesmo `workspace_id`, `path_namespace` e `policy_id`. Divergência produz erro de incompatibilidade; não comparar universos diferentes como se todos os arquivos tivessem sido removidos. Snapshots parciais podem produzir diferenças conhecidas e regiões desconhecidas, com resultado global parcial.

| Baseline | Atual | Resultado |
|---|---|---|
| Observado | Observado, mesma revisão e executabilidade | `unchanged` |
| Observado | Observado, revisão diferente | `modified`; informar também mudança de executabilidade, se houver |
| Observado | Observado, só executabilidade diferente | `metadata_changed` |
| Ausência demonstrada | Observado | `added` |
| Observado | Ausência demonstrada | `removed` |
| Observado | Exclusão conhecida por mudança de tipo, como arquivo virando link | `left_scope`; informar o motivo |
| Exclusão conhecida por tipo | Observado | `entered_scope` |
| Qualquer estado | Inacessível, instável ou não coberto | `unknown` |
| Inacessível, instável ou não coberto | Qualquer estado | `unknown` |

Nas duas últimas linhas, desconhecimento tem precedência. Exclusão nos dois lados fica no resumo de abrangência, sem mudança de conteúdo. Substituir arquivo por diretório significa saída daquele caminho do conjunto de arquivos; seus novos descendentes são comparados individualmente quando cobertos. Um arquivo legível no lugar de um diretório entra no conjunto; descendentes antigos ausentes seguem as mesmas regras de ausência demonstrada.

Renomeação não é inferida: caminho antigo removido e novo adicionado. A igualdade de hashes pode aparecer como dado, mas não muda a classificação. Arquivo vazio criado ou removido é diferença real, embora um sensor posterior baseado em linhas possa lhe atribuir peso zero.

O relatório tem versão de schema/contrato, IDs dos snapshots, política, `quality`, `relation`, alterações ordenadas, regiões desconhecidas e contadores. `relation` é `equal` ou `different` somente com observação completa dos dois estados; com parcialidade é `undetermined`, mesmo quando algumas mudanças são conhecidas. Não calcular D0 nem percentual de divergência nesta entrega.

Determinismo significa mesmos manifestos e contrato produzirem o mesmo JSON em bytes, inclusive ordenação. Não incluir relógio da execução ou duração da comparação nesse JSON. Diagnósticos operacionais variáveis ficam em stderr.

## 9. Interface proposta

Os comandos abaixo não existem ainda:

```text
compass init --root <diretorio> --json
compass snapshot --root <diretorio> --json
compass compare --baseline <snapshot-a.json> --current <snapshot-b.json> --json
```

`init` devolve identidade e localização da configuração. `snapshot` grava um manifesto e devolve ID, caminho e qualidade. `compare` escreve o relatório sem modificar os snapshots. O modo humano apresenta as mesmas informações essenciais do JSON.

| Código de saída | Significado |
|---|---|
| `0` | Operação completa; `compare` pode encontrar diferenças e ainda assim terminar com sucesso |
| `1` | Falha operacional ou registro corrompido; nenhum resultado completo |
| `2` | Argumentos/configuração inválidos ou snapshots incompatíveis |
| `3` | Snapshot ou comparação parcial, com resultado estruturado disponível |
| `130` | Cancelamento por SIGINT; nenhum manifesto final incompleto |

Com `--json`, stdout contém um único objeto JSON; mensagens humanas não são misturadas a ele. Erros também usam envelope JSON com `code` e `message`, sem conteúdo dos arquivos. O tratamento de outros sinais respeita o término do processo e não o converte em sucesso.

## 10. Casos independentes de aceite

| ID | Entrada ou intervenção | Esperado |
|---|---|---|
| F01 | Dois estados completos iguais | `equal`, zero mudanças, digest igual |
| F02 | Acrescentar arquivo `abc` | Uma adição com revisão dos três bytes |
| F03 | Remover arquivo observado e confirmar enumeração | Uma remoção |
| F04 | Criar/remover arquivo vazio | Adição/remoção; vazio não equivale a ausente |
| F05 | Trocar `abc` por `abd` mantendo tamanho | Modificação detectada por bytes |
| F06 | A → B → A, mesmas permissões de execução | Comparação dos extremos igual; IDs de observação diferentes |
| F07 | Renomear sem mudar bytes | Remoção e adição, sem rename heurístico |
| F08 | Negar leitura de arquivo | Estado parcial e `unknown`, nunca remoção ou igualdade global |
| F09 | Negar enumeração de diretório | Região desconhecida preservada; descendentes não viram removidos |
| F10 | Alterar arquivo durante as duas tentativas | `unstable`, sem revisão aceita |
| F11 | Link interno, externo e ciclo de links | Exclusões explícitas; nenhum alvo percorrido |
| F12 | Nome não UTF-8, nomes de caixa distinta e hardlinks | Chaves sem perda; identidade por caminho |
| F13 | Trocar CRLF/LF ou apenas BOM | Mudança de bytes detectada |
| F14 | Snapshot de outro acervo/política/schema | Erro de incompatibilidade apropriado, sem diff enganoso |
| F15 | Limite de leitura ou de entradas atingido | Parcialidade explícita, sem digest completo |
| F16 | Cancelar na escrita e iniciar escritores concorrentes | Nenhum final truncado, identidade estável e nenhuma sobrescrita |
| F17 | Trocar componente de caminho por symlink na abertura | Bloqueio do acesso ao alvo externo e falha explícita |
| F18 | JSON inválido, campo duplicado ou digest adulterado | Registro rejeitado antes da comparação |
| F19 | Repetir comparação dos mesmos manifestos; mudar só mtime; depois executabilidade | JSON idêntico na repetição; mtime não muda o estado; executabilidade gera `metadata_changed` |
| F20 | Acervo vazio; arquivo vira diretório/link e vice-versa | Vazio completo válido; transições de escopo e descendentes classificados sem confundir exclusão com falha |

Gabarito pequeno para o comparador: S0 contém `a.txt=abc` e `b.txt` vazio; S1 contém `a.txt=abd` e `c.txt` vazio, sob política e cobertura iguais. Esperado: `a.txt modified`, `b.txt removed`, `c.txt added`, nessa ordem por caminho. Esse resultado deve ser escrito antes do comparador.

As fixtures de hashing devem incluir vetores conhecidos de SHA-256, inclusive vazio e `abc`; a classificação de diferenças vem das entradas manuais. O laboratório Python não copia o algoritmo Rust. Testes negativos devem detectar, ao menos, comparador sempre igual, omissão de arquivos vazios e transformação de leitura falha em remoção.

## 11. Condição para iniciar e encerrar a implementação

A revisão deve confirmar ou ajustar as escolhas desta proposta, congelar o schema de transporte e os exemplos de referência em F0 e registrar a versão ativa. A execução então segue as fatias F1–F4 do [plano](../../docs/ANALISE_INICIAL.md).

O marco está encerrado quando a CLI realiza o fluxo demonstrável, os casos pertinentes passam e a entrega identifica código, toolchain, limitações e recursos observados. Nenhuma afirmação de utilidade científica de D0, difusão ou recuperação decorre desse aceite.
