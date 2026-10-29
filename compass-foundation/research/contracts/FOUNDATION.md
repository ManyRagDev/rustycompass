# Contrato ativo de estados e revisões do Compass Foundation

Contrato 0.1.0, schema 1, ativado em 14 de outubro de 2026 pela autorização de prosseguir com a direção alinhada à documentação original. Concretiza as seções 7, 17 e 20 do [documento principal](../../../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md). O escopo é Foundation em Linux; nenhum cálculo D0, grafo ou recuperação foi antecipado. A seção 12 fixa o transporte antes dos consumidores.

## 1. Capacidade entregue

Registrar uma observação de um acervo local e comparar duas observações persistidas, informando diferenças, revisões, abrangência e desconhecimento. A CLI e os testes usarão o mesmo núcleo Rust.

O primeiro alvo de validação é Linux. O domínio não depende de parsing nem de linguagem de programação: arquivos regulares são lidos como bytes, incluindo vazios e binários. Windows e macOS exigem contratos e testes próprios de filesystem antes de receberem a mesma alegação de suporte.

## 2. Objetos e identidades

| Objeto | Regra ativa |
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

A política padrão inclui arquivos regulares sob a raiz. Não interpreta automaticamente `.gitignore`: a existência de Git não altera silenciosamente o universo. Exclusões adicionais usam caminhos relativos exatos ou prefixos de diretórios, sem glob na versão inicial, e integram o `policy_id`.

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

`max_duration_ms` e cancelamento são cooperativos, verificados entre operações. Não interrompem uma syscall de filesystem bloqueada, por exemplo em NFS; portanto não representam prazo rígido de resposta. A leitura por descritor não elimina essa limitação.

Sem limites configurados, não se anuncia memória, duração ou disco máximos garantidos: o manifesto e sua ordenação ainda crescem com o número de entradas. Foundation mede esse custo antes da definição de limites para release.

## 7. Persistência e formato

A persistência usa `.compass/workspace.json` para identidade, `.compass/policy.json` para configuração e `.compass/snapshots/<snapshot_id>.json` para manifestos. Somente `init` cria o acervo; `snapshot` exige inicialização existente. A comparação recebe arquivos de snapshot explícitos e funciona sem acessar a árvore original.

Um snapshot contém os campos obrigatórios abaixo. O schema de transporte e as fixtures de serialização foram fixados em F0, antes da implementação da persistência e comparação, e são detalhados na seção 12.

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

Definições ativas:

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

## 9. Interface

A implementação disponibiliza os comandos:

```text
compass init --root <diretorio> --json
compass snapshot --root <diretorio> --json
compass compare --baseline <snapshot-a.json> --current <snapshot-b.json> --json
```

`init` devolve identidade e localização da configuração. `snapshot` grava um manifesto e devolve ID, caminho e qualidade. `compare` escreve o relatório sem modificar os snapshots. O modo humano apresenta as mesmas informações essenciais do JSON.

Os recibos JSON preservam os caminhos nativos absolutos mesmo quando a raiz contém nomes não UTF-8. `init` inclui `workspace_path_base64` e `policy_path_base64`; `snapshot` inclui `snapshot_path_base64`. Cada campo contém os bytes do caminho absoluto correspondente, codificados em base64url canônico sem padding. Os campos legíveis existentes (`workspace_path`, `policy_path`, `snapshot_path`) são auxiliares para exibição e podem substituir bytes não UTF-8; não servem para recuperar o caminho original nem para igualdade. Consumidores que abrem o arquivo indicado devem decodificar o campo base64 correspondente sem conversão obrigatória para UTF-8.

Esses caminhos absolutos pertencem somente ao recibo local da CLI. Não são as chaves relativas `path_key` do manifesto, não entram em `policy_id` ou `state_digest` e não alteram o formato de snapshots ou comparações. A extensão foi explicitada na revisão da primeira entrega para preservar o contrato de nomes nativos também na passagem do recibo ao próximo comando.

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

A autorização de prosseguir foi conferida contra o escopo vigente do documento principal. O transporte e os exemplos de referência foram fixados em F0 nesta versão ativa. A implementação segue as fatias F1–F4 do [plano](../../../docs/ANALISE_INICIAL.md); alterações futuras de semântica exigem registro explícito e atualização conjunta de contrato, consumidores e exemplos, sem ajustar gabaritos para acomodar bugs.

O marco está encerrado quando a CLI realiza o fluxo demonstrável, os casos pertinentes passam e a entrega identifica código, toolchain, limitações e recursos observados. Nenhuma afirmação de utilidade científica de D0, difusão ou recuperação decorre desse aceite.


## 12. Transporte congelado antes da implementação

`transport.schema.json` descreve os registros de workspace, política, snapshot e comparação. Todos os objetos rejeitam propriedades desconhecidas. O parser rejeita propriedades duplicadas em qualquer profundidade antes da desserialização. A validação de domínio complementa o schema: representação canônica, coerência da árvore, contadores, qualidade e hashes são obrigatórios.

- Versões são strings `"1"` e `"0.1.0"`; UUIDs v4 minúsculos; SHA-256 em 64 dígitos hexadecimais minúsculos. Janela usa RFC3339 em UTC (`Z` ou `+00:00`), com fim não anterior ao início.
- Todo contador/tamanho/limite/ordinal é string decimal canônica entre 0 e u64::MAX. Limites, quando presentes, são positivos. `read_attempts` da política é `"2"`. Limites e `state_digest` usam `null` explícito; revisão e motivo ausentes são omitidos.
- A política explicita `exclude_directory_names`, `exclude_file_names`, `exclude_file_prefixes` (nomes UTF-8), `exclude_paths` (chaves binárias exatas e subárvores), e os quatro limites. As listas são estritamente ordenadas e sem duplicatas. Padrão: diretórios `.aws`, `.compass`, `.git`, `.ssh`; arquivo `.env`; prefixo `.env.`; demais exclusões vazias; limites ausentes. Não há normalização de nomes.
- Tipos de entrada: `file`, `directory`, `symlink`, `other`, `unknown`. Somente arquivos podem ser `observed`. Diretórios percorridos aparecem em `directory_coverage`, incluindo raiz com chave vazia; diretórios podados aparecem somente como entrada `excluded`. Entradas de falha não contêm revisão. Motivos de exclusão: `policy_excluded`, `symlink_not_followed`, `unsupported_kind`.
- Entradas e cobertura estão ordenadas estritamente pelos bytes do caminho; nenhum caminho se repete entre as duas listas. Toda entrada/cobertura não-raiz exige cobertura do pai. Assim, a árvore não pode alegar um descendente conhecido sem registrar sua fronteira. Diagnósticos usam caminho, ordinal local, código e mensagem; são ordenados por bytes de caminho e ordinal numérico.
- `stats` contém `observed_files`, `excluded_entries`, `unknown_entries`, `bytes_read`, `read_attempts`, `duration_ms`. Os três primeiros devem corresponder às entradas (falhas de cobertura não contam como entradas desconhecidas). Bytes aceitos não excedem bytes lidos. Tentativas incluem leituras que falharam e repetidas; a contagem não pode ser menor que arquivos aceitos.
- A qualidade é derivada: qualquer entrada de falha ou cobertura incompleta torna `partial`, com digest nulo. Cobertura incompleta em qualquer ancestral prevalece sobre classificações de mudança naquele caminho. Uma entrada `observed` no ancestral demonstra que seus descendentes não existem como arquivos; uma fronteira excluída produz transição de escopo; um ancestral ausente sob cobertura completa demonstra ausência.
- Comparação contém `changes` (somente diferenças/unknown, sem unchanged), `unknown_regions` por lado e `counts`. Cada change contém `path_key`, `classification`, `before`, `after`, `content_changed`, `executable_changed`; before/after são a entrada original ou null. Transições de escopo incluem `reason` para preservar o motivo inclusive quando a fronteira excluída é ancestral. Diretório conhecido no lugar de arquivo significa `removed`; a transição inversa significa `added`. Counts contém `unchanged`, `added`, `removed`, `modified`, `metadata_changed`, `left_scope`, `entered_scope`, `unknown`. Exclusões em ambos os lados não produzem mudança de conteúdo. Qualquer snapshot parcial determina `quality=partial, relation=undetermined`; completa determina equal/different pelas mudanças conhecidas. Não há relógio no relatório.

As fixtures `fixtures/baseline.json`, `current.json` e `expected-comparison.json` foram escritas antes do comparador. O resultado esperado foi fixado manualmente: `a.txt modified`, `b.txt removed`, `c.txt added`; os hashes foram calculados independentemente com hashlib. A fixture `hash-vectors.json` contém os vetores conhecidos SHA-256 vazio e abc, além do JCS de propriedades fora de ordem.
