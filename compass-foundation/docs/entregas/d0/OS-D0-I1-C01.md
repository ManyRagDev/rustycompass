# OS-D0-I1-C01 — corrigir BUG-004: estabilidade de ambas as referências compartilhadas

```yaml
id: OS-D0-I1-C01
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: critico
origem: BUG-004 confirmado na AUDITORIA-OS-D0-I1
baseline: I1 v1 auditada; main sem HEAD; hashes abaixo
objetivo: impedir resultado integral quando a referência do segundo pacote compartilhado se torna instável
dependencias: contrato C2-A1 ativo; V1 aceita; auditoria desta ordem antes de executar
execucao: sequencial; mesma conversa do implementador I1; GPT-6.1 Sol / Alto
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.
Escopo pequeno, com uma causa confirmada, na fronteira de segurança de leitura. A ordem preserva o contrato existente e não exige decisão nova de produto. O porte crítico pede conferência prévia desta ordem; o aceite da correção depende de auditoria posterior do mesmo BUG/ACs.

## Baseline e leitura

Antes de escrever, inspecionar Git/diff, arquivos não rastreados e hashes. Salvar cópias/inventário inicial em `/tmp`; preservar alterações existentes. Não há commit HEAD. Conferir:

| Referência | SHA-256 antes da correção |
|---|---|
| `research/audits/AUDITORIA-OS-D0-I1.md` | `4a9000d6e75c9d3e30634113c918ea32db7e9e6a795c77630c170e932f975910` |
| `docs/entregas/d0/OS-D0-I1.md` | `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4` |
| `crates/compass-core/src/d0/source.rs` | `eaa6a88427b7b894b0f21d4a61572caa74ae0f0d1b106721827533c96b0e44a1` |
| `crates/compass-core/src/d0/report.rs` | `b118fa99fd95a67a75ed37378f549b7afcd77422803c7f3ffb60f72e2263e13b` |
| `crates/compass-core/src/d0/mod.rs` | `63517af44a11bdee2dfe1df460a56f35126566da15d3dec5697e58641f7259ec` |
| `verification/d0-i1/RESULTADO-D0-I1.md` | `8c12b18b927cc03c0a7f861ea33facaece13aea302fe91573a58ab1d2e942f42` |

`producer_build` auditado: `sha256:f532e4195ebce357e910b1e53a6b4a4aff334f8575c0a768569e06fc03aadb69`; binário auditado: `f3a0153b1ecdc026743c96e0e3595749ac072d0586ae13fae25213f78d47662f`. A correção terá novos hashes, capturados depois da última alteração de input do build.

Ler `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md), [OS-I1 original](OS-D0-I1.md), BUG-004 e os AC-03/04/07 na [auditoria I1](../../../research/audits/AUDITORIA-OS-D0-I1.md). Conferir §2 do contrato em `research/proposals/d0-c2/CONTRATO_CANDIDATO.md`, seus schemas de checks/disponibilidade/recursos, a seção de compartilhamento em `report.rs`, `Source/Root/BoundFile` em `source.rs`, os hooks e testes existentes. Consultar recursos/entrada/V1 apenas conforme a integração exigir.

Hashes normativos permanecem: índice C2-A1 `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`, 81 entradas; política default `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Se a árvore já contiver correção pertinente, identificá-la e verificá-la, sem sobrescrever ou inventar que a falha foi reproduzida nessa versão. Divergência não explicada da baseline exige relato com arquivo/hash, não reset automático.

## Contrato corretivo

Hoje `Source::shared` reconhece identidade física da raiz, descritor bruto e revisão. Depois da leitura da baseline, `report::unit` copia o check e os dados para current. A cadeia de referência própria de current deixa de passar pela reconferência de estabilidade feita na leitura da baseline.

**Cada referência participante da prova compartilhada deve ter a associação de nomes/componentes com seus handles fixados conferida antes de ser declarada válida.** Um check clonado da primeira fonte não é prova da estabilidade da segunda cadeia. Preservar a conferência já existente da baseline e acrescentar a que falta ao reutilizar sua evidência. A forma da função/helper interno fica a cargo do executor; não criar nova API pública ou camada de cache.

Se a referência de current tiver substituição detectada após sua preparação e durante a leitura da unidade, seu check deve ser `failed`, com motivo local `source_unstable`. A unidade não pode ter `calculation`, nem pode contribuir como texto disponível. Não transformar essa falha em erro genérico, `source_invalid` de preparação, `object_missing`, falso zero ou `diff_work_limit`.

Na reprodução com uma única unidade, baseline estável e current instável, esperar `measurement.status=unavailable`, exit 1, `value/line_mass/zero_basis/known_subset=null`, motivos globais `no_text_units` e `unresolved_units`; baseline permanece verified. Na unidade, motivo `source_unstable`, sem cálculo. Hash/tamanho/actual_revision só devem conservar fatos efetivamente comprovados conforme EOF e compartilhamento; não inventar flags ou apagar fatos comprovados apenas para simplificar o check.

A reconferência de referência não é uma segunda leitura do objeto. Preservar, para a revisão de dois bytes `a\n` da reprodução, **uma reserva de três bytes e leitura física de dois bytes**, tanto no compartilhamento estável quanto na intervenção depois dessa leitura ter sido admitida. Nenhuma reserva de células é necessária no caso bloqueado. No caso estável, o resultado continua completo `0/2`, zero células. Não desabilitar todo compartilhamento, proibir referências válidas com `..`, cobrar por caminho textual diferente ou mudar o ledger para fazer a regressão passar.

Continuam obrigatórias as condições anteriores de compartilhamento: mesma raiz fixada dev/ino, mesmo descritor bruto/binding e objeto esperado; limitado à unidade. Raízes distintas, hardlinks/reflinks ou hashes coincidentes não passam a compartilhar. Não acrescentar retries ou cache entre unidades. A conferência continua cooperativa, sem promessa de atomicidade de toda a observação.

**HIP-I1-01 está fora desta correção.** Não ampliar a política de modo/ctime/tamanho de diretórios nem reinterpretar silenciosamente o conjunto de metadados. O BUG-004 diz respeito à substituição comprovada de um componente e à conferência omitida, independentemente dessa hipótese. BUG-001/002/003 permanecem encerrados.

## Escrita autorizada — lista fechada

Somente estes arquivos existentes:

- `crates/compass-core/src/d0/source.rs`: reconferência da referência fixada e helpers internos estritamente necessários ao BUG-004; testes pertinentes.
- `crates/compass-core/src/d0/report.rs`: integração da reconferência no compartilhamento, estados/motivos e testes pertinentes.
- `crates/compass-core/src/d0/mod.rs`: somente testes da regressão e instrumentação privada `#[cfg(test)]` estritamente necessária; preservar o fluxo público de produção deste arquivo.
- `verification/d0-i1/RESULTADO-D0-I1.md`: acrescentar seção `OS-D0-I1-C01 / BUG-004`, preservando o resultado original e seus hashes históricos.
- `verification/d0-i1/acceptance.json` e `verification/d0-i1/mutations.json`: novas saídas exatas do adaptador contra o release corrigido; antes de regenerar, conservar cópias/hash dos resultados auditados em `/tmp` e registrar a transição no retorno.

Todos os demais arquivos são protegidos, inclusive **V1 inteira**, baseline JSON/tar, `run-v1.py`, OS-I1 original, ativação, contratos/schemas/fixtures, auditorias, documentos centrais, CLI, README, Cargo e demais módulos Rust/testes existentes. Não criar novos arquivos de projeto ou dependências. Temporários de teste, cópias, diffs e logs podem ficar em `/tmp`; Cargo pode escrever artefatos em `target/`.

A OS-I1 original deve permanecer idêntica: o adaptador confere seu hash na baseline histórica. Não regenerar a baseline nem alterar os esperados para acomodar esta correção. Esta OS nova autoriza os deltas acima e preserva os demais aceites de I1.

## Regressão obrigatória e aceites

Criar teste permanente com nome contendo `bug004` nos módulos autorizados, usando os hooks privados e thread/barreira para o ponto exato. Exercitar a operação D0 com manifesto/pacote reais artificiais, não apenas `Root::stable` isolado ou um bool de compartilhamento. Nenhuma flag pública de teste.

Receita autossuficiente: um manifesto válido da fixture `manifests/a.json`, usado bruto nos dois lados; pacote `T/package`, descritor vinculado ao arquivo exato e objeto `a\n`; diretório adicional `T/alias-component`. Baseline aponta a `T/package`; current a `T/alias-component/../package`. Pausar em `object_opened`, depois de descritores/handles fixados e antes da leitura. Outro thread renomeia `alias-component`, cria um novo no mesmo nome e libera o leitor. O componente troca de inode, mas a raiz final, objeto e hashes permanecem iguais. As referências devem ser conservadas como fornecidas; canonicalizá-las antes do teste removeria o estímulo do BUG.

Reproduzir o defeito com o teste novo **antes de alterar a lógica de produção**: controle estável passa; a asserção de current instável falha porque a implementação auditada o declara verified. Registrar a falha específica. Erro de compilação, setup, timeout ou import não conta como reprodução. A adição de teste já muda o hash do build; distinguir essa árvore com teste novo da baseline binária original.

| ID | Aceite observável |
|---|---|
| C01-AC-01 | A regressão de operação completa reproduz o falso complete antes do patch e passa depois; na intervenção de current, seu check é failed/source_unstable, baseline verified, unidade unavailable sem calculation e measurement/exits/nulls conforme o contrato corretivo. Não aceitar somente retorno não zero genérico. |
| C01-AC-02 | Controles sem intervenção, com mesma referência e com referências diferentes para a mesma raiz (`alias-component/../package`), continuam completos `0/2`; ambos verified, reserva 3, leitura 2 e zero células. A referência válida não é rejeitada para eliminar o caso difícil. |
| C01-AC-03 | Na intervenção sincronizada de current, reserva 3 e leitura 2 permanecem; não há segunda leitura de objeto, restituição indevida, débito de células ou fatos EOF inventados. Motivos únicos/ordenados e source_ref/expected_revision mantêm correspondência com seus inputs. |
| C01-AC-04 | A intervenção equivalente na baseline continua detectada e não publica integral. Trocas de objeto, nome, componente, ancestral e pós-EOF já testadas continuam detectadas; compartilhamento restrito à unidade, raízes distintas/hardlinks e fonte ausente/adulterada preservam os comportamentos anteriores. |
| C01-AC-05 | Testes completos Rust, fmt/Clippy/release, 23 testes e três mutações Foundation, campanha V1 real e 11 mutações D0 passam na árvore final; negativos continuam atingindo suas asserções. Registros de runtime são gerados pelo adaptador intacto contra o binário corrigido estável, sem alterar a V1. |
| C01-AC-06 | Inventário/diff/hash comprova somente escritas autorizadas e preservação dos inputs e baseline; producer_build recomputado dos inputs efetivos coincide com o emissor do novo binário. Retorno vincula C01/BUG-004/ACs aos testes e relata falhas/restrições reais. Estado final do executor é IMPLEMENTADO_PENDENTE_VERIFICACAO, sem fechar BUG-004 ou liberar R1 por conta própria. |

Os auxiliares da auditoria em `/tmp/compass-audit-i1-0aiuhfud/` podem orientar uma conferência adicional, mas **não são dependência do teste permanente**. Não sobrescrevê-los: `public-races.py` está ligado ao binário antigo e registra observações, não um critério automático de aprovação da correção. A conferência independente posterior repetirá o cenário pela CLI contra o novo hash.

## Comandos, evidência e retorno

Comandos existentes, na raiz. Durante desenvolvimento, executar primeiro o teste focado (`test -p compass-core bug004 --locked --offline` pelo mesmo wrapper). Após a última edição/formatação de fonte:

```sh
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
python3 verification/reference/acceptance.py --binary target/release/compass
python3 verification/reference/mutations.py --binary target/release/compass
python3 verification/d0-i1/run-v1.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Formatação em escrita somente nos três arquivos Rust autorizados. Não compilar enquanto o binário está em campanha. Mudança posterior de input do build exige renovar identidade, build e verificações afetadas. O perfil lógico/policy_id default não muda por corrigir a prova de estabilidade; o producer_build muda com os fontes/testes, pelo mecanismo existente.

Sockets Unix locais podem exigir permissão técnica fora do sandbox. Solicitar pelo mecanismo da ferramenta ou registrar limitação; EPERM/skip/timeout não é aprovação. Não acessar acervos reais ou rede. Não repetir a auditoria geral nem iniciar benchmark/RSS/R1 nesta correção.

Acrescentar no resultado I1: baseline/hashes antes/depois, deltas e tentativas, ID da regressão e asserção que falhou antes/passou depois, tabela dos seis ACs, valores observados de checks/status/exits/reservas, comandos executados, novos producer_build/SHA-256 binário e hashes dos JSONs regenerados. Preservar o relato original, sem trocar retroativamente seus resultados pelos novos.

Arquivo ou dependência fora da lista: `BLOQUEIO_DE_FRONTEIRA`, com motivo e evidência. Divergência legítima de contrato: encaminhar à coordenação, sem alterar critério. Duas falhas da mesma abordagem do BUG exigem reinvestigação antes de terceiro patch automático. Reverter somente hunks desta ordem contra cópias iniciais, sem reset/clean e preservando trabalho anterior.

Após o retorno, auditoria focada de BUG-004, C01-AC-01–06 e impactos nos AC-I1-03/04/07. O parecer anterior e a OS original permanecem históricos; a nova conferência identifica a versão corrigida. Nenhuma mensagem a outros chats, operação remota ou liberação automática de R1 está autorizada.
