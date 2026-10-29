# Resultado OS-RD-I1 — versão 1

**IMPLEMENTADO_PENDENTE_VERIFICACAO**. 19/10/2026, America/Sao_Paulo.
Execução da skill executor-blindado v2.0, um escritor, três fases sequenciais.
Este resultado é a verificação do executor; o aceite depende da auditoria posterior.
Não há pendência obrigatória conhecida da RD nem skip no conjunto final executado.

## Baseline e identidade

`main`, sem HEAD (`git rev-parse --verify HEAD`: 128). Os arquivos adicionados e
não rastreados eram preexistentes; nenhum commit foi criado. Git/status/diffs
foram lidos antes da escrita. Baseline de 176 arquivos em
`/tmp/compass-rd-baseline`, com cópias e binário anterior; o inventário, os deltas
e os hashes estão incorporados em [provenance.json](provenance.json).

| Identidade | SHA-256 / build |
|---|---|
| Binário C01/R1 preservado | `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f` |
| Produtor anterior, 37 inputs recompostos | `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676` |
| Contrato RD v1, preservado | `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72` |
| OS liberada v1, preservada | `dedbf49a02f8b9c774e4a3b11f1c1be80a28f79901873ae08cd1d2028b02d419` |
| Parecer R1, preservado | `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9` |
| Inventário inicial | `551ef43e6c5b47c54cd9dc05f00dd59348049e8db23938a3a4c7a2586ec8357c` |
| Release RD testado, antes/depois das campanhas | `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d` |
| Produtor final, 41 inputs recompostos | `sha256:6192a943a657f23ebaea7267eea67f63ee482741c85c37d44634817113bfebf9` |

Os 41 inputs efetivos, seus tamanhos e hashes completos constam na proveniência.
O contrato RD foi acrescentado explicitamente ao build, preservando domínio e
framing Foundation. Snapshots de produtor histórico continuam aceitos.

## Escritas e comportamento

Cinco existentes alterados, exatamente na lista da OS:
`src/lib.rs` exporta search; `d0/mod.rs` muda somente três visibilidades para
pub(crate); `build.rs` inclui o contrato RD; CLI adapta search; README documenta
uso real, limites e cobertura. Os caminhos completos e diffs estão na proveniência.

Foram criados somente `crates/compass-core/src/search.rs`, os dois testes
`crates/compass-core/tests/search.rs` e `crates/compass-cli/tests/search.rs`, e os
oito artefatos autorizados em `verification/rd/`. Os **171 arquivos preexistentes
fora desses cinco deltas permaneceram idênticos**, incluindo contratos,
gabaritos, evidências, testes antigos, Cargo/lock/toolchain, estado e coordenação.
O índice C2-A1 e suas 81 entradas conferiram. Não houve edição do leitor D0.

Search valida um manifesto e o descritor explicitamente vinculado, admite S/W e
reserva s+1 por caminho. Só busca bytes após EOF/hash/tamanho/estabilidade.
Entrega uma linha por hit, bytes/offsets originais e ordem nativa. Preflight de
todas as linhas precede retenção de hits. O scan continua após max_matches;
cobertura e seleção são separadas. JSON canônico e legível são preparados no
núcleo, sob os checkpoints finais de cancelamento/deadline, antes de publicar.
A CLI preserva os comandos anteriores e seus transportes.

## Fases e AC → evidência

Fase 1: [fixtures.json](fixtures.json) foi fixado antes da produção, SHA-256
`6152b318bbc47f88d797164d351ba3451a268f7d3e6fe944801c4aeebc82159b`, inalterado
até a entrega. Inclui literais/offsets/hashes, fronteiras, RD01–RD24 normativos e
controle de mutações. O primeiro self-test, sem produção, passou com a ferramenta
SHA `71482df998b7430cbd4eea9625069d6df6cb3a875fc94fb3fe6c45b8f8986457`.
A ferramenta foi depois ampliada para a campanha real; não ajustou o fixture.

Fase 2: núcleo/CLI e testes focados. Fase 3: fmt/Clippy/testes, release,
campanha RD, três medições e regressões completas sobre a identidade final.
Os JSONs fornecem bytes brutos recuperáveis por base64, comprimentos e hashes.

| AC | Resultado do executor e evidência |
|---|---|
| RD-AC-01 | Lista fechada conferida: cinco deltas existentes, 11 novos; 171 preservados. `provenance.json` baseline/deltas/release_inputs e regressão dos testes D0. |
| RD-AC-02 | Self-test pré-produção, fixture estável e esperado adulterado rejeitado por `RD01.literal`; contas S_manifest=128656, S=131224, W=81922, peak=213146 conferidas independentemente. `fixtures.json`, `acceptance.py::selftest`, proveniência phase1. |
| RD-AC-03 | RD01–08: literais, Unicode/BOM/CRLF, CR/U+2028 sem quebra, caixa, sobreposição, espaços/metacaracteres, paths 10/2/0xff, UTF-8/NUL. `integration.json`, `coverage.json` e Rust `rd01_08_frozen_literals_nontext_and_boundaries`; teste público de bytes/preservação. |
| RD-AC-04 | RD09–13/21/23: ausente, adulterado mesmo/menor/maior, symlink/pending/FIFO/socket, binding inclusive vazio e snapshot parcial. Barreiras Rust `rd12_synchronized_object_component_ancestor_and_dotdot` trocam objeto, objects, ancestral e alias/../package antes da reconferência, com controle estável; nenhum hit instável, fatos EOF preservados. |
| RD-AC-05 | Vazio/non_text complete, parcial sem resolvidos unavailable; late failure após max_matches=1 conserva known=3/truncated e partial. `integration.json`, pointers RD03/04/08/09/13/16 de `coverage.json`. |
| RD-AC-06 | Reservas por caminho/sem reembolso, pequeno posterior, W antes de leitura, memória/peak e transporte nas fronteiras, query 256/257 e linha 4096/4097, output exato/cap-1. Rust `rd17_exact_structural_and_working_credit_precedes_open` e `rd19_synchronized_cancel_deadline_and_priority`; checkpoints na leitura, segunda janela UTF-8 sem LF/hits, linhas/busca e preparação/publicação, cancelamento vence deadline/erro pendente. |
| RD-AC-07 | JSON duplicado/extra/u64 inválido/depth/perfil completo, JCS sem newline/legível com newline e queries d0/init/search. Cinco mutações específicas de resposta pública real parcial/truncada, controle válido antes: RD24.text/offsets/revision/coverage/truncation. `integration.json#/mutations`, self-test e testes CLI, inclusive erro sem --json. |
| RD-AC-08 | init→snapshot→pacote→search real, árvore viva removida e stdout idêntico, entradas imutáveis; três invocações nativas da receita textual, RSS separado de ledger. `integration.json`, `resource-profile.json`. |
| RD-AC-09 | 74 Rust, fmt/Clippy/build; 23 Foundation + três mutações; 253 verificações V1/119 cenários + 11 mutações D0, sem skips. `regressions.json` inclui os JSONs V1 completos e logs; proveniência incorpora logs Rust/build e comandos. Tudo contra o release final identificado acima. |
| RD-AC-10 | `coverage.json` liga os 24 IDs aos índices de comandos realmente armazenados e/ou nomes de testes internos; `provenance.json` liga inputs, deltas, ferramentas, binário e artefatos. Resultados do executor aguardam auditoria, sem conversão em aceite. |

RD23 usa dois snapshots do mesmo workspace, `BOM+alvo+CRLF` e `alvo+LF`:
D0 real tem numerador zero, mas cada search entrega seu próprio texto bruto;
pacote trocado falha com incompatible_inputs. D0 zero não permite misturar revisões.

## Comandos finais e resultados

Na raiz do projeto, salvo o comando do índice:

```sh
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
PYTHONDONTWRITEBYTECODE=1 python3 verification/rd/acceptance.py --self-test
PYTHONDONTWRITEBYTECODE=1 python3 verification/rd/acceptance.py --binary target/release/compass
PYTHONDONTWRITEBYTECODE=1 python3 verification/rd/acceptance.py --regressions --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Todos passaram. RD: **151 comandos CLI, 1.482 assertivas auxiliares e cinco
mutações de resposta**, além do self-test; esses números não representam 1.482
cenários independentes. Rust: 74 testes, zero falhas/ignorados. Os comandos
Foundation executados pelo modo regressions são acceptance.py e mutations.py,
ambos com o executável final absoluto. D0 usa exclusivamente verify/mount do
runner I1 autenticado, V1 intacta em /tmp e binário real absoluto; adapter.run e
runner R1 fixado ao release antigo não foram chamados. Os resultados históricos
não foram regravados nem apresentados como execução RD.

## Três medições nativas finais

1.000 arquivos × 4.096 bytes, 100 diretórios; receita ASCII M/S0 com BASE somente
em j=31 e NONE nas outras 63 linhas. Um snapshot/pacote; 1.000 linhas matching,
20 retornadas, truncated=true. Três stdout idênticos. GNU time individual,
sem compilação/leitura paralela iniciada pelo executor durante as medições.

| Amostra | User/System (s) | Max RSS (KiB) | GNU elapsed (s) | Wall com wrapper (ns) |
|---|---:|---:|---:|---:|
| 1 | 0.11 / 0.02 | 12520 | 0.14 | 145939976 |
| 2 | 0.07 / 0.02 | 12684 | 0.09 | 98611384 |
| 3 | 0.08 / 0.03 | 12636 | 0.11 | 115090880 |

Ledger em cada amostra: object_bytes_read=4.096.000,
object_bytes_reserved=4.097.000, logical_memory_peak=14690378.
Ambiente: Linux x86_64, UID 1000, 12 CPUs lógicas;
kernel `7.0.11-76070011-generic`. Pacote: tamanho lógico
4096339 bytes, alocado
4100096 bytes; manifesto
205750 bytes. Máquina/comandos/GNU time bruto,
stdout, hashes, ambiente/cache e ledger estão em `resource-profile.json`.
Cache não controlado, sem flush/root/SLO; memória lógica não é RSS.

## Falhas de desenvolvimento e limites

- O sandbox inicialmente negou socket Unix num teste D0 preexistente. A permissão
  técnica para testes sintéticos foi obtida; suíte e controles finais passaram,
  sem skip. Isso não autorizou operação remota.
- Erro de compilação por movimento parcial no teste novo e lint de inicialização
  foram corrigidos; logs/narrativa estão na proveniência. Não houve ajuste do
  esperado ou mudança de contrato.
- Duas tentativas do laboratório falharam no mesmo socket path longo. Antes da
  terceira, a causa foi reinvestigada e medida: 108 bytes, reduzidos para 100.
  Outra falha do laboratório removia o manifesto junto da árvore viva: o
  manifesto sintético foi copiado para fora dela. A campanha final passou.
- Uma checagem usou cwd d0-c2 para caminhos relativos da raiz e recebeu arquivo
  inexistente; foi repetida na raiz. Não conta como verificação bem-sucedida.
- A revisão final moveu a preparação do JSON legível para o núcleo, dentro da
  janela de deadline/cancelamento. Após isso foram renovados release, testes,
  campanhas, regressões e três medições; artefatos intermediários foram salvos
  em /tmp antes de substituição.
- Cancelamento é cooperativo. I/O bloqueante, hashing/parse/validação/JCS e
  serialização compartilhada não têm preempção interna; há checks antes/depois.
  Os novos laços RD respeitam checkpoints por janela. Não há promessa de
  latência máxima, atomicidade global, segurança contra OOM ou autoria autenticada.
- Anotações runtime_pending da V1 histórica são preservadas literalmente nos
  JSONs aninhados; as provas sincronizadas desta árvore são identificadas em
  coverage/proveniência, sem editar a V1 ou declarar que seu smoke é sincronizado.
- Apenas Linux/openat2 e acervos sintéticos. Nenhuma instalação, rede, commit,
  deploy, migration, acervo real, módulo posterior ou mensagem a outro chat.
- Uso compartilhado da conta: 21%→21% arredondados na janela de 10.080 minutos;
  não é medição de tokens deste projeto. Temporários estão sujeitos à retenção
  de /tmp; inventários/deltas/logs finais relevantes estão incorporados.

## Hashes dos artefatos

| Artefato | SHA-256 |
|---|---|
| `verification/rd/acceptance.py` | `609882519abfa81d02d54e28176ef2bda8c988141b83854635c1c71a271c01ba` |
| `verification/rd/coverage.json` | `214ddbcba093d75b40ac080008e8e11f801c7a6e49b325fa33b8c88c2bb9394b` |
| `verification/rd/fixtures.json` | `6152b318bbc47f88d797164d351ba3451a268f7d3e6fe944801c4aeebc82159b` |
| `verification/rd/integration.json` | `d2a807745b41b9afb59370ff9134cc5edf82e5284b1c97e6a1c2fc382d02dd8d` |
| `verification/rd/regressions.json` | `b17099cc5f202033e54980fb32a539c256358bffaee11e465d54d4abff306da8` |
| `verification/rd/resource-profile.json` | `f65d6f0462ee233303bff41daebe119f182ffcf7187a89ffe9d4885c2222ca4d` |

A proveniência exclui seu próprio hash e este resultado para evitar ciclo;
ambos os hashes finais são informados na mensagem externa. Reversão deve usar
somente os cinco deltas e novos arquivos da lista autorizada, com as cópias
preservadas; não usar reset/clean.

**IMPLEMENTADO_PENDENTE_VERIFICACAO**
