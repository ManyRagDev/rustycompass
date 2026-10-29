# Auditoria focada — OS-D0-I1-C01 v1 / BUG-004

**Veredito: CONFORME. BUG-004: VERIFICADO.** A correção cumpre C01-AC-01–06 e resolve a não conformidade nos I1-AC-03/04/07 relacionada à referência compartilhada. Não foi identificado achado obrigatório remanescente nesse escopo. A I1 corrigida pode seguir para o aceite/integração R1; este parecer não executa R1 nem mede RSS.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo `entrega`, conferência dos mesmos BUG/ACs e regressões afetadas. Referências: [ordem C01](../../docs/entregas/d0/OS-D0-I1-C01.md), [parecer I1 anterior](AUDITORIA-OS-D0-I1.md) e [apêndice do executor](../../verification/d0-i1/RESULTADO-D0-I1.md), a partir da linha 179. O parecer anterior permanece histórico e não foi alterado.

## Versão e preservação

Branch `main`, sem HEAD/commits, com alterações locais e diretórios não rastreados. Nenhum resultado foi atribuído a um commit inexistente.

| Referência corrigida | SHA-256 |
|---|---|
| OS-D0-I1-C01 v1 liberada | `c5a47f6cb4b76ec3b9311f4c114e28bc06b97ef8fcf5d1e9e0c17b30a0f66866` |
| Resultado I1 com apêndice C01 | `1d19a188de9b44170cb6655752dc5554f7787a027692370b1568ae4b3e906ebf` |
| `d0/source.rs` | `4d1c263758ea605376be4938dde947945480aa49eafd78038a3e8281be9e9f42` |
| `d0/report.rs` | `c10a161dc9a4c2f385757cbe79389dc8b9f4f6b3bf3a4931c92315275621fe22` |
| `d0/mod.rs` | `cefc57b3b695edcc643b15c5239b8f454b6503e6e8addb53dd7d0f33386d0a12` |
| Binário corrigido entregue | `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f` |

Confrontada a baseline de **159 arquivos** do executor, `/tmp/compass-c01/before.json`: somente os seis arquivos autorizados mudaram — três Rust, apêndice do resultado e dois JSONs gerados. Os outros 153 permaneceram idênticos; não houve adições de arquivos na correção. O resultado original é prefixo byte a byte do atual. Cópias das evidências antigas e diffs correspondem aos hashes apresentados pelo executor.

O delta de `mod.rs` está somente nos testes; seu prefixo de produção é idêntico. `source.rs` acrescenta o helper interno `reference_stable`, que chama a reconferência existente de Root. `report.rs` copia os campos de evidência compartilhável, preserva os campos próprios de current e, se sua referência estiver instável, acrescenta `source_unstable`, ordena/deduplica motivos e marca failed. Não há nova leitura de objeto, retry, cache, dependência, API pública ou alteração de ledger.

Antes desta auditoria foi fixado inventário de **161 arquivos**, incluindo o executável e o documento principal externo. Digest SHA-256 do mapa caminho→hash serializado por `json.dumps(sort_keys=True,separators=(',',':')).encode()`: `dc4aa6fa43cbd025ac73d7ce6164ec8c969b567bfdc658ffc90bb312b500294c`. Os originais foram preservados; execução em cópias sob `/tmp/compass-audit-c01-gegos3_7/`.

## Conferência independente antes/depois

O auditor não se limitou aos logs do implementador. Em uma cópia isolada, manteve os **testes atuais sem alteração** e restaurou somente `source.rs` e `report.rs` aos dois hashes de produção anteriores registrados na OS. O comando focado retornou **101: dois testes passaram e um falhou**, com a asserção específica:

```text
BUG-004: unstable current was declared verified
left: String("verified")
right: "failed"
```

Isso reproduz o defeito nos fontes anteriores com a regressão permanente. Não foi falha de montagem, compilação ou timeout. Essa árvore inclui o teste novo, portanto não recebeu indevidamente a identidade do antigo binário release.

Na árvore corrigida os três testes passaram, incluídos nos **64 testes Rust** executados pelo auditor. Os controles verificam operação completa, relatório estruturado, estado de cada check, nulls, ausência de cálculo, reserva 3/leitura 2/células 0, abertura única e fatos EOF/source_ref/revisões.

Também foi repetida a **intervenção sincronizada pela CLI**, contra cópias dos executáveis antigo e corrigido identificados por hash. Mesma receita do BUG: baseline usa `T/package`, current usa `T/alias-component/../package`, ambos vinculados ao mesmo manifesto e objeto `a\n`. Dois pipes pausam a primeira leitura de objeto, depois dos handles/descritores/metadados capturados; o pai substitui o componente por outro inode, mantendo a raiz final e conteúdo. A instrumentação não muda bytes ou retornos das leituras; nenhuma flag pública de teste foi acrescentada.

| Controle público | Binário antigo | Binário corrigido |
|---|---|---|
| Mesma referência, estável | complete, ambos verified | mesmo resultado, integral 0/2 |
| Referências distintas para mesma raiz, estáveis | complete, ambos verified | mesmo resultado, integral 0/2 |
| Troca do componente da referência current | **complete indevido, ambos verified** | **exit 1, unavailable; baseline verified, current failed/source_unstable** |
| Troca equivalente na baseline | unavailable/source_unstable | bloqueio preservado |

Nos quatro controles corrigidos: **reserva 3, leitura física 2 e zero células**. Nos bloqueados, unidade unavailable, `calculation` ausente, `measurement.value/line_mass/zero_basis/known_subset=null`, motivos globais `[no_text_units, unresolved_units]`. Os fatos reais do único EOF são preservados: actual_revision igual à expected_revision, hash_check/size_check true. No controle current, baseline continua sem motivo de falha. Os dois controles estáveis produzem inclusive os mesmos bytes JCS, sem newline.

## Resultado por AC

| Critério | Parecer | Evidência do auditor |
|---|---|---|
| C01-AC-01 | CONFORME | Mesma regressão falha com produção anterior e passa com a corrigida; prova pública antes/depois confirma failed/source_unstable, nulls/exits e ausência de cálculo. Nenhum erro genérico foi aceito como fechamento. |
| C01-AC-02 | CONFORME | Controles de mesma referência e referência indireta válida permanecem complete 0/2, ambos verified, uma leitura/reserva e zero células; não se proibiu `..` para eliminar o caso. |
| C01-AC-03 | CONFORME | Operação permanente e CLI confirmam 3/2/0, fatos EOF e campos próprios dos checks; nenhuma segunda leitura ou débito indevido. Motivos únicos e ordenados. |
| C01-AC-04 | CONFORME | Troca na baseline continua bloqueando integral; suíte Rust mantém trocas de objeto/nome/componente/ancestral/pós-EOF. Campanha V1 mantém compartilhamento limitado à unidade, raízes distintas/hardlinks e fontes faltantes/adulteradas. |
| C01-AC-05 | CONFORME | 64 testes Rust, fmt/Clippy/release, 23 testes Foundation, três mutações Foundation, 253 verificações V1 e 11 mutações D0 reproduzidos, sem falhas/skips nas execuções finais. |
| C01-AC-06 | CONFORME | Seis deltas autorizados, histórico e inputs preservados, identidade dos 37 inputs recomposta e produtor/binário conferidos. Retorno mantém pendência de verificação até este parecer. |

**I1-AC-03/04/07: não conformidade do BUG-004 resolvida.** Os demais resultados da auditoria I1 anterior não foram reabertos por preferência. HIP-I1-01 continua observação contratual separada, sem patch ou obrigação nova nesta C01.

## Comandos e resultados executados

Comandos Cargo pelo wrapper local na cópia correspondente; `CARGO_TARGET_DIR` em `/tmp/compass-audit-c01-gegos3_7/rust-target`. As evidências originais do executor não foram sobrescritas.

| Verificação | Resultado |
|---|---|
| `scripts/cargo-local.sh test -p compass-core bug004 --locked --offline -- --nocapture --test-threads=1`, na cópia com os dois fontes históricos | Exit 101 esperado; 2 passaram, 1 falhou pela asserção de current indevidamente verified. |
| `scripts/cargo-local.sh fmt --all -- --check` | Exit 0, sem escrita. |
| `scripts/cargo-local.sh test --workspace --locked --offline` | Exit 0, **64 testes**, zero falhas/ignorados. |
| `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings` | Exit 0. |
| `scripts/cargo-local.sh build --release --workspace --locked --offline` | Exit 0. |
| `python3 verification/reference/acceptance.py --binary /tmp/compass-audit-c01-gegos3_7/submitted-compass` | **23 testes**, OK. |
| `python3 verification/reference/mutations.py --binary /tmp/compass-audit-c01-gegos3_7/submitted-compass` | **3 mutações rejeitadas**, zero skips. |
| `python3 verification/d0-i1/run-v1.py --binary /tmp/compass-audit-c01-gegos3_7/submitted-compass --cached-dependencies /home/many/.cache/uv/archive-v0` | **253 verificações**, incluindo 119 cenários runtime repetidos; **11 mutações D0**, zero falhas/skips; hash estável antes/depois. |
| Instrumentação sincronizada pela CLI, binários antigo/corrigido | Quatro controles por versão; o defeito específico desaparece e os controles válidos permanecem aprovados. Envelopes completos da versão corrigida conferidos. |
| `(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)`; `git diff --check` | Exit 0; 81 artefatos intactos; inventário cobre não rastreados. |

As suítes com socket Unix foram executadas com permissão técnica fora do sandbox; não houve conexão de rede. A auditoria não converteu restrição, skip ou timeout em aprovação.

## Proveniência e evidências

Recomputados independentemente os **37 inputs efetivos**, com domínio `compass-build-v1` mais byte NUL, caminhos ordenados e comprimentos u64 little-endian. Resultado:

`producer_build=sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`.

Coincide com o emitido pelo release entregue e pelo rebuild do auditor. Política default preservada: `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`. As campanhas públicas usaram o executável entregue `7c4e61fd…a5ebd2f`, sem compilação concorrente. O rebuild em diretório temporário tem SHA-256 `6cc4ddb6a66dd1a4f9b7c159c937306203c693feffc13a12ce222652c1bad334`; não se alega reprodução binária idêntica nem se confundem os dois artefatos.

Evidências auxiliares em `/tmp/compass-audit-c01-gegos3_7/`, sujeitas à retenção de temporários:

| Arquivo | SHA-256 |
|---|---|
| `regression-old-production.log` | `d8354b98f1a1567f1b15dcfbc89585d6fb2e6b7c7a60fd91d5f5746810a9be52` |
| `workspace.log` | `6c6db9d6a8b1fe93e7242feb8a3156280678fb316726728ffc8ea56a79f70988` |
| `public-races-before.json` | `5d41461494d7131618be187cdc9a3cf29282d9674f43a177722cb2f6851971b3` |
| `public-races-corrected.json` | `5df8b662fefd3943bc5e166288699b93e8f56f9a6c4a21075f0578a8eaf13c31` |
| `public-full-corrected-full-alias_current.json` | `ef0aad7f732c5264bde7ff2b08ccc55132cfb2be24f900ddc90288d34f537188` |
| `io_gate.c`, mesma instrumentação da prova anterior | `2e903bd68284a4ffb9b253ab1eb34782baee6b130bfa5c24f97a50bc7fdda29a` |
| `v1-runtime.log` | `292fa7694dc8f6f5bcbbdf88d2b31a13ee0e6fba562fd8809f0f11376cd0685f` |

Reexecução dos controles públicos, sem modificar provas anteriores:

```sh
python3 /tmp/compass-audit-c01-gegos3_7/public-races.py /tmp/compass-audit-i1-0aiuhfud/submitted-compass before
python3 /tmp/compass-audit-c01-gegos3_7/public-races.py /tmp/compass-audit-c01-gegos3_7/submitted-compass corrected
```

## Encerramento

| ID | Tipo | Estado | Fechamento | Tentativas declaradas |
|---|---|---|---|---|
| BUG-004 | implementacao | **verificado** | C01-AC-01–06 demonstrados, prova pública antes/depois e regressões aprovadas na versão identificada | 1 |

Os 161 arquivos inventariados permaneceram idênticos durante a auditoria, inclusive código, binário original corrigido, contratos, V1, baseline, resultado do executor e parecer anterior. Apenas este novo parecer foi acrescentado ao projeto. Nenhum achado foi corrigido, nenhum outro chat foi contatado e nenhuma nova entrega I1/R1 foi iniciada.

RSS/benchmark e o aceite integrado de R1 continuam pendentes conforme o plano, sem serem tratados como defeitos desta correção. A verificação de referências é cooperativa; não se anuncia atomicidade global. HIP-I1-01 e BUG-001/002/003 conservam o estado anterior.

Uso compartilhado consultado no início/fim: 17% → 18% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta auditoria.
