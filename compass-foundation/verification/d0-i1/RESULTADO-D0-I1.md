# Resultado — OS-D0-I1 v1

Data: 19/10/2026, America/Sao_Paulo. Intenção: execução.
Skill: executor-blindado v2.0. Estado: **IMPLEMENTADA_PENDENTE_AUDITORIA**.
I1-a → I1-b → I1-c implementadas sequencialmente e verificadas. A campanha
V1 executou o sensor real; isso não substitui a auditoria independente nem o
aceite R1. R1 não foi iniciada; nenhuma mensagem a outro chat/subagente.

## Baseline e autorização

Branch `main`, sem commits/HEAD, confirmado por `git rev-parse --verify HEAD`.
Havia 34 adições Foundation locais e diretórios não rastreados, incluindo V1,
docs, propostas e auditorias. Git/diff foram inspecionados sem reset/clean.
Inventário/cópias de 141 arquivos antes de I1 em `/tmp/compass-d0-i1-gpwhq9td/before/` e
`before.json`. Os quatro arquivos existentes liberados só foram editados após
`run-v1.py --prepare-baseline` conferir os 34 hashes históricos e os 81 artefatos.

Conferidos antes das escritas: ativação P1–P4, OS liberada após auditoria de
contrato CONFORME, auditoria V1 C01–C03 e referências. Identificadores:

| Referência | SHA-256 |
|---|---|
| OS-D0-I1 liberada | `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4` |
| Índice C2-A1 | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| Contrato D0 | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| Auditoria V1 aceita | `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1` |
| Resultado V1 preservado | `4e04373959ebec7c1237cd888bb9aa67ab1038847fc5a97e5b5d6a01cea2c186` |
| baseline-foundation.tar, 34 membros regulares relativos | `bbc9dd44c871585fa41e2599bc497d1598898918d472a561683340fc55c86c62` |

`baseline.json` registra membros/hashes, V1 durável e resultados existentes,
OS e hash do tar. A preparação repetida confere/reutiliza, sem substituir.
A conferência final demonstrou: contratos, 81 artefatos, V1 inteira e 30
históricos não liberados idênticos; somente os quatro existentes autorizados
mudaram. O inventário histórico original não foi regravado. Sem alteração de
Cargo.toml/Cargo.lock/toolchain ou instalação de dependências.

## Arquivos entregues e hashes

Quatro deltas existentes, comparados às cópias pré-I1:

| Caminho | Antes | Depois |
|---|---|---|
| `README.md` | `a4f6be5111fd6747549e4eec7484d12b9d10e5888d6631dfd92c54d367fcd373` | `47ba3249a5326269a2629a399a00e7366ae263b0f5ea49fb3adb721c717995f8` |
| `crates/compass-core/build.rs` | `b28f43833a943214fb58421c78e730511e00c621f29999e14ae31305219ae013` | `177ed18c772b4f8178df6c231ce995a82dc853e91c4c4efd345406e029815d28` |
| `crates/compass-core/src/lib.rs` | `8715757ba37b97e858c5f961fb0e2fc1ac98ede586312fa882c15530520b939b` | `3bf12624f8c48b63aa448d697afb337bd5a6ee0d1cfd8c0e6f8b457b636caf21` |
| `crates/compass-cli/src/main.rs` | `6fdf80de84d8e34934dbdf9fbd6bb775d3fca9e61d809666f12c15b631747e8b` | `18fc31bb6b4c1c02690d5cd7fe96eceb5ed152387ebae5aba1ebbc5581f1d8a7` |

Novos, todos na lista fechada da OS (este próprio relatório é excluído do
hash autorreferente):

| Caminho | SHA-256 |
|---|---|
| `crates/compass-cli/tests/d0.rs` | `3b984b28ff6d99e8ab18aa297df464fd739788d4ff0e0cbc7363c708af19c987` |
| `crates/compass-core/src/d0/input.rs` | `f54bf0f8eb2cb6b603b181893f9299e69319c90147f3414723337887b3384f6a` |
| `crates/compass-core/src/d0/metric.rs` | `d9b17deea84ee0551ea8a6ed8b530c3b44b297f747fde098aa6075348bdaef86` |
| `crates/compass-core/src/d0/mod.rs` | `63517af44a11bdee2dfe1df460a56f35126566da15d3dec5697e58641f7259ec` |
| `crates/compass-core/src/d0/report.rs` | `b118fa99fd95a67a75ed37378f549b7afcd77422803c7f3ffb60f72e2263e13b` |
| `crates/compass-core/src/d0/resources.rs` | `494688961b1d43be4c8300e65d9498feb01b52011c300970dc36590023bfcc3d` |
| `crates/compass-core/src/d0/source.rs` | `eaa6a88427b7b894b0f21d4a61572caa74ae0f0d1b106721827533c96b0e44a1` |
| `crates/compass-core/src/d0/types.rs` | `c0fae25814d42e95f918b8608aa5f62a3bcc6b2ce705dab01cf5508226b42964` |
| `crates/compass-core/tests/d0.rs` | `d9ad853cadd2b1af12b871217daf718176f65802f550b15b07e83d5ea3bd8c76` |
| `verification/d0-i1/acceptance.json` | `1a93db661534912f9bd9dbe7dc5124005bb0fab82b79ab37601b0be1d34109f6` |
| `verification/d0-i1/baseline-foundation.tar` | `bbc9dd44c871585fa41e2599bc497d1598898918d472a561683340fc55c86c62` |
| `verification/d0-i1/baseline.json` | `19f808a4943da16bea0ea00bb721dcc7d65ba0f5e3554d8a47fc6e1d270453e9` |
| `verification/d0-i1/mutations.json` | `ab33513f4e2ef1e49c587323ef4cb39cba54cbb8086a569762e49cf4c479d7a8` |
| `verification/d0-i1/run-v1.py` | `94d9ada79741fe38121e2884ee964394ba2c7073f1a2805234f5d51bb2c13298` |

Diffs pertinentes dos quatro arquivos existentes, logs e inventários auxiliares
estão em `/tmp/compass-d0-i1-gpwhq9td`. Código novo fica nos sete módulos `d0/`, testes core/CLI e
adaptador. `lib.rs` só exporta D0; `build.rs` acrescenta os sete inputs explícitos;
CLI preserva dispatch/transporte Foundation; README documenta operação/limites.
Nenhum arquivo fora da lista foi alterado. Artefatos Cargo ficam em `target/`.

## Implementação e aceites

| AC | Resultado efetivamente observado / teste ou evidência |
|---|---|
| I1-AC-01 | ATENDIDO. Inventário inicial/final: quatro deltas existentes autorizados e novos permitidos; 30 históricos e V1 intactos. Adaptador montou 34 históricos/81 artefatos e rejeitou membro adulterado, membro `../escape`, fixture adulterada e fonte V1 alterada em cópias. Executável real `/usr/bin/false`, somente como controle sem D0, provocou falha de ambos os comandos e retorno não zero do wrapper; não foi apresentado como sensor aprovado. `adapter-tests.json` no temporário registra esses controles. |
| I1-AC-02 | ATENDIDO. `strict_transport_and_depth`, `structural_preflight_precedes_domain_validation`, `descriptor_binding_and_present_invalid_on_empty_universe`, CLI `strict_manifest_profiles_limits_and_empty_source_envelopes` e `noncanonical_manifest_correct_raw_binding_and_human_mode`. Transporte não canônico com binding bruto correto passa; duplicatas/BOM/trailing/UTF-8 inválido, limites de profundidade/registros/bytes e perfil inválido recebem códigos/exits exatos. Hook demonstra S_manifest recusado antes do evento de domínio, com controle que o alcança. |
| I1-AC-03 | ATENDIDO nos testes Linux executados. `prohibited_kinds_missing_and_no_external_bytes` confere links de raiz/intermediário/pendente/direto, FIFO/socket/diretório, ausência e controle regular. `synchronized_object_name_component_and_post_eof_replacement` e `synchronized_package_ancestor_replacement_with_same_pinned_root` usam threads/barreiras para trocas após abertura, inclusive conteúdo/hash coincidentes. Instabilidade bloqueia e preserva fatos EOF; raiz/ancestrais/componentes mantêm handles e têm associação de nome conferida. Não há fallback stat+open nem leitura do alvo de links proibidos (contador físico zero). |
| I1-AC-04 | ATENDIDO. `object_eof_prefix_empty_and_short`, `reservation_never_refunded_and_chunk_checkpoint`, `same_root_shares_only_within_unit` e cenários V1 E28/E30/E31/E43/E44/E45/E47. Hash/tamanho do mesmo buffer; prefixo s+1 não recebe hash/actual_revision; vazio exige objeto/probe. Reserva não é devolvida em missing/recusa. Mesmo pacote compartilha somente na unidade; raízes distintas/hardlinks não emprestam prova e caminho posterior relê. |
| I1-AC-05 | ATENDIDO. `frozen_lcs_counts_aggregates_and_byte_vectors` confere 44 cálculos e agregados literais; `e36_e51_e52_e54_e55_equality_and_balances`, relatório congelado do núcleo e campanha real de 119 cenários cobrem matemática e disponibilidade. Fontes conferidas antes da igualdade, CRLF/LF brutos distintos, BOM/UTF-8/NUL, unchanged e unknown/exclusões observados. Esperados C2/V1 permaneceram intactos e não foram obtidos de Rust. |
| I1-AC-06 | ATENDIDO. `exact_memory_and_nonrefundable_reservations`, `e49_u64_and_checked_equations`, testes de saldo/igualdade e casos V1 de limites/memória/precedências/continuação. S/Wb/W e peak, bytes reais/reservados separados e células sem débito na recusa; candidato menor posterior passa. u64::MAX, excesso e quatro empates/extremos E49 testados diretamente, sem arquivos enormes. |
| I1-AC-07 | ATENDIDO. `frozen_physical_reports_exact_except_real_producer` reproduz dez relatórios completos. Na V1 real, cinco schemas, semântica/exit/ordem nativa/zeros/subconjunto/binding e regressões BUG-001/002/003 passam. `report.rs` valida cálculos e checks disponíveis no núcleo; Foundation valida os inputs; Limits/descritor têm domínios e campos fechados. Relatório não inclui conteúdo, hunks, raiz ou timestamp novo. |
| I1-AC-08 | ATENDIDO. Três testes CLI exercem help, argumentos/envelopes, perfil, modo humano/JSON sem newline e binding não canônico. `synchronized_sigint_after_first_unit_before_next_and_before_publication` aciona a mesma flag usada pelo handler, por thread/barreira no checkpoint exato; `deadline_before_publication_and_priority_over_error`, `interrupt_precedes_deadline_and_pending_error`, `lcs_checkpoints_bound_work_and_cancel` e chunk checkpoint verificam aborto e prioridade controlados, sem depender de repetição probabilística. O smoke público V1 recebeu SIGINT real e exit 130, complemento distinto das provas internas. |
| I1-AC-09 | ATENDIDO. V1 intacta na árvore histórica temporária invocou o release I1 absoluto: 253 verificações, 119 cenários runtime (duas invocações por cenário), zero falhas/skips; 11 controles positivos e 11 mutações semânticas específicas aprovados. JSONs foram copiados byte a byte, sem filtros/transformação. Binário estável antes/depois; produtor real confere com fontes I1. Pendências genéricas V1 reconciliadas abaixo sem reescrever os JSONs. |
| I1-AC-10 | ATENDIDO. Árvore final: 61 testes Rust (40 Foundation + 21 D0), 23 testes CLI Foundation e três mutações Foundation, nenhum skip; fmt, Clippy com -D warnings e release passaram. Hash independente de 37 inputs reproduz producer_build emitido e registrado na campanha, distinto de Foundation. |
| I1-AC-11 | ATENDIDO para entrega I1. README e este retorno separam implementação testada, auditoria/R1 pendentes, memória lógica e RSS não medido; sem promessa de eficácia contextual, atomicidade do acervo ou deadline rígido. |

## Proveniência final

- `producer_build`: **`sha256:f532e4195ebce357e910b1e53a6b4a4aff334f8575c0a768569e06fc03aadb69`**.
- SHA-256 de `target/release/compass`: **`f3a0153b1ecdc026743c96e0e3595749ac072d0586ae13fae25213f78d47662f`**.
- Política default: `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.
- 37 inputs: 21 originais com os deltas autorizados, nove novos fontes/testes
  Rust e sete referências D0 explícitas, sob `research/proposals/d0-c2/`:
  contrato, cinco schemas e `fixtures/metric-policy.json`.

Recomputação independente em Python percorreu a lista efetiva (Cargo.toml,
Cargo.lock, rust-toolchain.toml, arquivos de crates/, research/contracts/ e os
sete acréscimos), ordenou caminhos UTF-8 relativos e aplicou SHA-256 sobre
`compass-build-v1` + byte NUL, comprimento u64 little-endian/caminho e
comprimento u64 little-endian/bytes. `final-provenance.json` no temporário
registra a lista completa e hashes. Uma invocação real em manifesto vazio
artificial confirmou o produtor; todos os relatórios runtime da campanha
apresentam essa mesma identidade. SHA-256 do binário antes/depois nos dois
JSONs coincide com o executável final. Não houve compilação durante a campanha.

## Comandos e evidências efetivas

Ambiente: Linux 7.0.11-76070011-generic, Rust/Cargo 1.90.0 pelo wrapper local,
Python 3.12.3 e dependências jsonschema já disponíveis em cache. Sem instalação.
Os sockets e testes de filesystem precisaram da permissão técnica fora do
sandbox; não houve conexão externa. Cada comando abaixo terminou em exit 0:

| Comando | Resultado | Log auxiliar SHA-256 |
|---|---|---|
| `scripts/cargo-local.sh fmt --all -- --check` | fmt sem escrita | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `scripts/cargo-local.sh test --workspace --locked --offline` | 61 testes, zero falhas/ignorados | `6004b320fdab25b0efa59e6ef0a992bfdb5836574da31ad1983ab3acea0b51b8` |
| `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings` | sem avisos | `8c233a54790c4800d3dd585436af679aa838da259bfdb129412e2d51cdb75cdc` |
| `scripts/cargo-local.sh build --release --workspace --locked --offline` | release compilado | `c0133482decf48ea11e248ee509e7fdbd63dee40eabeea4726a34c5f85b2180e` |
| `python3 verification/reference/acceptance.py --binary target/release/compass` | 23 testes, OK | `0247bee6866cff58b9bcbf62dcce356bcbbf53cb26f52b0f9f6f58418c5d691a` |
| `python3 verification/reference/mutations.py --binary target/release/compass` | 3 mutações rejeitadas, zero skips | `30681a41b51849d4e210ca200a94003416e5ad341151b8120e38cde65cec6dd2` |
| `python3 verification/d0-i1/run-v1.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0` | 253 verificações/119 cenários; 11 mutações, zero falhas | `07d86546109f83edfb835aec13388293bfe497c49c467e5bd5542d3436d0a028` |

Também executados: `run-v1.py --prepare-baseline` antes das edições e novamente
para conferir reutilização; testes focados durante I1-a/I1-b/I1-c; negativos do
adaptador em cópias; `(cd research/proposals/d0-c2 && sha256sum --check --quiet
SHA256SUMS)`, `git diff --check` e `git diff --cached --check`, todos aprovados.
Os não rastreados tiveram sua fronteira/inventário conferidos diretamente.

## Falhas encontradas durante a execução

Falhas de desenvolvimento foram tratadas como falhas, sem contá-las como aceite:

1. Compilação inicial encontrou empréstimo de AtomicBool temporário em input.rs;
   corrigido com binding de duração adequada.
2. Primeiro teste de filesystem revelou EINVAL em openat2: O_PATH não aceita
   NONBLOCK/NOCTTY. As flags foram separadas para handles sem leitura e handles
   de leitura; os testes de tipos/corridas então passaram. Sem fallback inseguro.
3. Clippy recusou módulo inicial por contador %4096 e tipo complexo do hook;
   usados is_multiple_of e alias privado, sem suprimir lint.
4. Após a primeira campanha aprovada, controle sincronizado adicional demonstrou
   uma lacuna real: trocar ancestral do pacote mantendo o mesmo inode da raiz
   não era detectado. O teste falhou com reasons vazio; foram fixados também
   handles dos ancestrais e dos componentes, com conferência de identidade/nome.
   O mesmo teste passou após o ajuste; build, testes e campanhas foram renovados.

Os builds intermediários e suas campanhas não são atribuídos à identidade final.
`prior-final/` conserva logs da rodada anterior; os logs `final-*.log` correspondem
à árvore final. Não restou falha/skip convertido em aprovação.

## Pendências V1 reconciliadas e retorno para auditoria/R1

| Prova transferida | Evidência I1 / limite |
|---|---|
| CLI D0 real/artefato estável | acceptance.json e mutations.json: runtime_executed=true, hashes estáveis, produtor final conferido. |
| TOCTOU/substituições após abertura | Testes sincronizados de objeto, nome, componente e ancestral do pacote, incluindo hash coincidente e troca após EOF. Apenas árvores artificiais, sem alegar observação atômica. |
| SIGINT/deadline em pontos exatos | Hooks privados/barreiras do núcleo controlam a flag do handler e expiração; testes de prioridade/aborto antes da publicação. Smoke público recebeu sinal real, distinto da prova interna. |
| Chunks e operações LCS | Leituras limitadas a 65536 bytes, checkpoints entre chunks; LCS verifica a cada no máximo 4096 células/operações, com cancelamento testado. |
| S_manifest antes do domínio | structural_preflight_precedes_domain_validation inclui recusa sem alcançar domínio e controle que o alcança. |
| E49 extremo/arredondamento | e49_u64_and_checked_equations com vetores diretos e aritmética u128 checada/saída u64. |
| RSS | NAO_MEDIDO, continua para R1; logical_memory_peak é ledger de admissão. |

Os `runtime_pending` e o rótulo synchronized_checkpoint nos JSONs V1 são os
originais emitidos pelo laboratório; não foram editados para fingir cobertura.
As provas acima reconciliam essas pendências pela identificação dos testes
executados. Auditoria independente da implementação, revisão dos 11 ACs,
aceite/integracão R1 e medição de recursos reais continuam pendentes.

Reversão: somente novos arquivos e hunks I1 contra as cópias prévias, preservando
baseline, evidências úteis e trabalho anterior; sem reset/clean. Nenhum deploy,
migration, alteração remota, coleta de acervos reais ou envio a outro chat.
Uso da conta no início/fim: 15% → 15% na janela compartilhada de 10.080 minutos;
diferença visível zero, com arredondamento e outros trabalhos, sem inferência de
consumo nulo ou contagem exata de tokens desta OS.


## OS-D0-I1-C01 / BUG-004

19/10/2026, America/Sao_Paulo. Intenção: **execução**, skill executor-blindado v2.0.
Estado desta correção: **IMPLEMENTADO_PENDENTE_VERIFICACAO**. Tentativa de
correção de produção #1. Os registros I1 acima permanecem históricos; esta
seção identifica a versão corrigida e não fecha BUG-004 nem concede aceite I1/R1.

### Baseline e fronteira

OS-D0-I1-C01 v1 liberada, com auditoria prévia CONFORME em
`research/audits/AUDITORIA-CONTRATO-OS-D0-I1-C01.md`. `main`, sem HEAD;
`git rev-parse --verify HEAD` retornou 128, `fatal: Needed a single revision`.
As 34 adições Foundation e os diretórios não rastreados já existiam.
Inspecionados status, diff de trabalho, índice e arquivos não rastreados antes
de escrever; nenhum reset/clean/commit. Inventário inicial de **159 arquivos**
do projeto (incluindo ignorados, excluídos `.git`, `target` e `__pycache__`),
cópias dos seis arquivos autorizados e diffs iniciais em `/tmp/compass-c01/`.
SHA-256 de `before.json`: `61d8af6077a3ebf44d8e0748eeb0c8cf93c9a2e0b52a5a14e813251d0b44540f`.
Os seis hashes de baseline enumerados na OS coincidiram antes das escritas.

Somente seis arquivos existentes alterados: três Rust, este apêndice e os
dois JSONs exatos do adaptador. Nenhum arquivo de projeto criado.
`source.rs`: +5/-0, helper interno que chama a conferência existente de Root;
`report.rs`: +22/-1, reutiliza somente os campos de evidência do check e
reconfere a referência própria de current, acrescentando `source_unstable`
com motivos ordenados/únicos. `mod.rs`: +145/-0, somente dentro de
`#[cfg(test)]`; fluxo público de produção preservado. Nenhuma segunda leitura,
retry, cache novo, nova API pública ou alteração de ledger/Source::shared.

V1 inteira, baseline JSON/tar, adaptador, OS-I1 original, contratos, 81
artefatos C2-A1, gabaritos, CLI, Cargo e demais módulos preservados. O inventário
final em `/tmp/compass-c01/after.json` e `preservation.json` confere os 153
arquivos restantes idênticos, sem adições/remoções, além da igualdade dos
diffs Git iniciais/finais. Os deltas de arquivos não rastreados estão nos
`source.rs.diff`, `report.rs.diff`, `mod.rs.diff` e no inventário, pois o diff
Git sozinho não os representa. O resultado original é prefixo byte a byte
deste arquivo (SHA histórico `8c12b18b927cc03c0a7f861ea33facaece13aea302fe91573a58ab1d2e942f42`).

### Prova específica antes/depois

Regressão permanente em `d0::tests`, usando a operação D0 completa, o manifesto
bruto congelado `fixtures/manifests/a.json`, descritor com binding desse arquivo
e pacote artificial `T/package`, objeto `a\n`. Os inputs conservam
`T/alias-component/../package`, sem canonicalização. Thread/barreira pausa
em `object_opened` depois da reserva e dos handles fixados, antes de ler;
outro thread renomeia o componente e cria outro diretório no mesmo nome.
Não depende dos auxiliares da auditoria nem usa flag pública.

Antes de editar produção, foram acrescentados os três testes e executado
`test -p compass-core bug004 --locked --offline -- --nocapture` (wrapper),
exit 101: **2 passaram, 1 falhou**, sem falha de setup/compilação/timeout.
A repetição com `--test-threads=1` somente tornou o log legível e reproduziu
a mesma falha. `before-test-serial.log` registra:

```text
bug004_current_component_replaced_during_shared_read
assertion left == right failed: BUG-004: unstable current was declared verified
left: String("verified")
right: "failed"
```

Naquela árvore, source/report ainda tinham os hashes auditados; somente o
novo teste alterava os inputs. `mod.rs` pré-patch:
`709ffa2de9f508328cf4707740e1a0a375c53ec4214ce36b2362a2000fd80087`.
O produtor recomputado dessa árvore com teste novo é
`sha256:18d5830b886535083ee1db5f0df797129e67fdd37db5e7fa89d169743cdd8d07`;
não é o binário auditado original nem um produtor capturado do emissor release.
O produtor da baseline reconstruída dos inputs originais coincide com o
histórico `sha256:f532e4195ebce357e910b1e53a6b4a4aff334f8575c0a768569e06fc03aadb69`.
O hash binário original `f3a0153b1ecdc026743c96e0e3595749ac072d0586ae13fae25213f78d47662f`
é referência histórica da OS/auditoria; não foi recapturado antes do rebuild.

Depois da correção, o mesmo comando focado com `--test-threads=1` passou:
**3 testes, zero falhas/ignorados** (`after-test.log`). A suíte completa
após a formatação também passou com os três testes finais.

| Cenário | Antes | Depois | Reserva / leitura / células |
|---|---|---|---|
| `bug004_stable_same_and_distinct_references_share_one_read`: mesma referência e referência indireta estável | exit 0, complete 0/2, ambos verified | mesmos resultados; unidade available, cálculo c=0/m=2 | 3 / 2 / 0 em ambos os controles |
| `bug004_current_component_replaced_during_shared_read` | **exit 0, complete 0/2, ambos verified**, cálculo indevido | **exit 1, unavailable**, baseline verified sem motivos, current failed/source_unstable, unidade unavailable sem calculation | 3 / 2 / 0 antes/depois |
| `bug004_baseline_component_replacement_remains_detected` | exit 1, unavailable, baseline failed/source_unstable | mesmo bloqueio; current herda evidência já failed | 3 / 2 / 0 antes/depois |

Nos casos bloqueados, `measurement.value/line_mass/zero_basis/known_subset=null`,
reasons globais exatamente `["no_text_units","unresolved_units"]`, unidade com
`["source_unstable"]`, resolved_text_units=0, unavailable_units=1. O teste
exige Output estruturado e asserções específicas, rejeitando erro genérico.
Uma abertura de objeto por unidade é contada no hook; bytes reais e reservas
são conferidos no relatório. Não houve devolução de reserva nem débito de células.

EOF foi alcançado na leitura compartilhada: ambos os lados conservam
`actual_revision=expected_revision`, hash_check=true e size_check=true, mesmo
com a falha de referência. Revisão: SHA-256
`87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7`, tamanho 2.
Cada check mantém observation/source_ref/expected_revision de seu input;
source_ref do descritor artificial:
`6fb84d32f684f333c294c8fc0d9efcb5b932c58839a2cdca3cf17f095830aa6c`.
Nenhum fato novo de hash/EOF foi inferido pela reconferência de diretórios.

### Aceites C01 → evidência

| AC | Resultado observado e evidência |
|---|---|
| C01-AC-01 | Demonstrado pelo teste current acima, falha específica antes e sucesso depois; `before-test-serial.log`, `after-test.log`, suíte final. Exit/status/checks/nulls/unidade sem cálculo explicitamente asseridos. |
| C01-AC-02 | Dois controles do teste estável preservam ambas as referências válidas, complete 0/2, ambos verified, reserva 3/leitura 2/zero células. |
| C01-AC-03 | Helper comum dos três testes exige uma abertura, reserva 3/leitura 2/zero células, correspondência dos campos e fatos EOF. Current instável tem somente source_unstable; agregação de motivos permanece ordenada/única. |
| C01-AC-04 | Controle baseline da regressão e testes existentes `synchronized_object_name_component_and_post_eof_replacement`, `synchronized_package_ancestor_replacement_with_same_pinned_root`, `prohibited_kinds_missing_and_no_external_bytes`, `same_root_shares_only_within_unit` passam. V1 runtime E44-shared/E44-distinct/E44-hardlink e fontes ausentes/adulteradas preservados; dez relatórios físicos congelados passam. |
| C01-AC-05 | 64 testes Rust (40 Foundation + 24 D0), 23 testes Foundation CLI, 3 mutações Foundation e V1 real com 253 verificações/119 cenários repetidos/11 mutações D0; zero falhas/skips na execução final. Fmt, Clippy, release, 81 hashes e whitespace passaram. Negativos alcançaram as asserções específicas; logs/JSONs intactos, sem enfraquecimento de esperados. |
| C01-AC-06 | Inventários inicial/final, cópias e deltas em /tmp; seis escritas autorizadas, demais 153 arquivos idênticos. `provenance.py/json` recomputam os 37 inputs, produtor emitido igual e binário estável na campanha. Histórico preservado; executor pendente de verificação independente. |

### Comandos e resultados

Todos os comandos na raiz de compass-foundation, salvo indicação; logs em
`/tmp/compass-c01/`. Após o teste antes/depois, rustfmt escreveu **somente** os
três Rust autorizados, com `--edition 2024 --config skip_children=true` pela
ferramenta local em `../.tooling/cargo/bin/rustfmt` e RUSTUP_HOME do projeto.
Nenhuma fonte foi alterada depois da formatação. A ordem de validação final foi:

| Comando | Resultado / log |
|---|---|
| `scripts/cargo-local.sh fmt --all -- --check` | exit 0; `fmt.log` |
| `scripts/cargo-local.sh test --workspace --locked --offline` | No sandbox: exit 101, 39 unitários passaram e o teste de socket falhou em UnixListener::bind com EPERM (`source.rs:484`); não contou como aprovação. Reexecução autorizada fora do sandbox: **64 passaram, zero falhas/ignorados**, exit 0; `workspace-test-unrestricted.log`. |
| `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings` | exit 0; `clippy.log` |
| `scripts/cargo-local.sh build --release --workspace --locked --offline` | exit 0; `release.log` |
| `python3 verification/reference/acceptance.py --binary target/release/compass` | exit 0, **23 testes OK**, zero skips; `foundation-acceptance.log` |
| `python3 verification/reference/mutations.py --binary target/release/compass` | exit 0, **3 mutações rejeitadas**, zero skips; `foundation-mutations.log` |
| `python3 verification/d0-i1/run-v1.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0` | No sandbox: exit 1, construção de campanha EPERM; 11 checks passaram/2 falharam, sem aprovação; `v1.log`, JSONs preservados como `sandbox-acceptance.json`/`sandbox-mutations.json`. Reexecução autorizada fora do sandbox: exit 0, **253 verificações/119 cenários runtime/11 mutações**, zero falhas/skips; `v1-unrestricted.log`. |
| `(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)` | exit 0, 81 entradas; índice congelado intacto |
| `git diff --check` | exit 0; repetido após o retorno |
| `python3 /tmp/compass-c01/provenance.py` | exit 0; recomputação independente dos inputs e comparação ao produtor realmente emitido por D0; `provenance.json`, `emitted-report.json` |

Na repetição final, o adaptador intacto montou o laboratório histórico em
`/tmp/compass-i1-v1-kd4hzmm7/compass-foundation` e chamou o release absoluto.
JSONs foram copiados pelo próprio adaptador, byte a byte; não foram editados.
Não ocorreu compilação durante nenhuma campanha. SHA do binário igual antes
e depois. A primeira restrição técnica de socket foi superada pelo mecanismo
da ferramenta nas duas suítes; não houve rejeição de aprovação automática.
Nenhuma operação remota, rede, acervo real, instalação ou dependência nova.

### Hashes, proveniência e transição das evidências

SHA-256 dos arquivos (antes → depois; este resultado é excluído da autorreferência):

| Arquivo | Antes | Depois |
|---|---|---|
| `crates/compass-core/src/d0/source.rs` | `eaa6a88427b7b894b0f21d4a61572caa74ae0f0d1b106721827533c96b0e44a1` | `4d1c263758ea605376be4938dde947945480aa49eafd78038a3e8281be9e9f42` |
| `crates/compass-core/src/d0/report.rs` | `b118fa99fd95a67a75ed37378f549b7afcd77422803c7f3ffb60f72e2263e13b` | `c10a161dc9a4c2f385757cbe79389dc8b9f4f6b3bf3a4931c92315275621fe22` |
| `crates/compass-core/src/d0/mod.rs` | `63517af44a11bdee2dfe1df460a56f35126566da15d3dec5697e58641f7259ec` | `cefc57b3b695edcc643b15c5239b8f454b6503e6e8addb53dd7d0f33386d0a12` |
| `verification/d0-i1/acceptance.json` | `1a93db661534912f9bd9dbe7dc5124005bb0fab82b79ab37601b0be1d34109f6` | `f3ccbaf7818b86ef2d05fd427405a99c0bd5798586b4d1b2fd07ab840d71b9cb` |
| `verification/d0-i1/mutations.json` | `ab33513f4e2ef1e49c587323ef4cb39cba54cbb8086a569762e49cf4c479d7a8` | `7dc51a4b6204d65fed8136082901c18530feb9102eebd16e1d214cda7e84741d` |

As saídas auditadas dos dois JSONs estão preservadas em
`/tmp/compass-c01/before/verification/d0-i1/`, com os hashes anteriores da tabela.
Não houve regeneração da baseline. Baseline JSON permaneceu
`19f808a4943da16bea0ea00bb721dcc7d65ba0f5e3554d8a47fc6e1d270453e9`;
tar `bbc9dd44c871585fa41e2599bc497d1598898918d472a561683340fc55c86c62`.
OS-I1 original continua `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4`;
auditoria I1 `4a9000d6e75c9d3e30634113c918ea32db7e9e6a795c77630c170e932f975910`;
índice C2-A1 `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`.

- **producer_build final:** `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`.
- **SHA-256 release final:** `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`.
- **metric_policy_id default preservado:** `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Recomputação Python independente do domínio `compass-build-v1\0`, ordem dos
caminhos e comprimentos u64 little-endian de caminho/bytes reproduziu os 37
inputs efetivos, após a última edição de fonte; coincide com o produtor do
relatório realmente emitido pelo release. O manifesto original não foi
reescrito. Não se alega identidade binária com rebuild em outro diretório.

Logs decisivos (SHA-256; todos em `/tmp/compass-c01/`):

| Evidência | SHA-256 |
|---|---|
| `before-test-serial.log` | `b063485569e7bfd41b8e9b219497ffa64ea5ecfd16ceefa4654d89127187ca95` |
| `after-test.log` | `9df3c946d8a2fcbf82e92662eb6dd3bc5f8976054f90a86399c967d5a9198586` |
| `workspace-test-unrestricted.log` | `871135d9df8817922a26b7358d6188c54661747a3f0c048d5ab659219a6ecc02` |
| `foundation-acceptance.log` | `82d1bb473591c3afedb8d4c4909448daa9f37a435ab41b88a600b85382222540` |
| `foundation-mutations.log` | `30681a41b51849d4e210ca200a94003416e5ad341151b8120e38cde65cec6dd2` |
| `v1-unrestricted.log` | `b52e1ad835d39437ba5939111ac5c1dc239b0bbd81bdb1e381f78017dfdeb513` |
| `provenance.json` | `54302aa0efccb3f16ca6889c5b4927c29939e7181cd436dac43bfbcf3467fb8e` |

### Limitações e encaminhamento

A conferência auxiliar de whitespace inicialmente rejeitou o prefixo `+ ` do
próprio ndiff em linhas vazias; o script temporário foi corrigido para conferir
o conteúdo após o prefixo. Não houve ajuste de fonte ou esperado por isso.

A barreira permanente testa operação D0 no núcleo; a CLI foi exercitada pela
campanha real, mas a intervenção sincronizada independente pela CLI do novo
hash fica para a auditoria focada posterior. Os campos genéricos runtime_pending
da V1 foram preservados nos JSONs; as provas internas de concorrência e
checkpoints são os testes Rust citados, não uma alegação de que a V1 os cobriu.
Conferência cooperativa de referências, sem promessa de atomicidade global.
HIP-I1-01 não foi implementada nem usada como critério desta correção;
BUG-001/002/003 permanecem no estado anterior. R1, benchmark e RSS não iniciados.
Não houve subagentes, mensagem a outros chats ou mudança dos critérios.

Uso compartilhado da conta: 17% → 17% na janela de 10.080 minutos; diferença
visível de 0 ponto percentual, sujeita a arredondamento e outros trabalhos,
sem atribuir contagem de tokens a esta OS.

**IMPLEMENTADO_PENDENTE_VERIFICACAO** — aguarda auditoria independente de
BUG-004/C01-AC-01–06 e impactos I1-AC-03/04/07, sem fechamento automático do BUG.
