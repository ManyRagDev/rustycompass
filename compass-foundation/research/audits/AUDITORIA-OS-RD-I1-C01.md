# Auditoria focada — OS-RD-I1-C01 / BUG-005

**CONFORME. BUG-005 VERIFICADO.** C01-AC-01–06 atendidos; não conformidade de RD-AC-07 resolvida. A coordenação pode aceitar RD-I1 corrigida. Nenhum novo achado obrigatório identificado no escopo da correção.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo entrega. Referências: [OS corretiva](../../docs/entregas/rd/OS-RD-I1-C01.md), [auditoria anterior](AUDITORIA-OS-RD-I1.md), [contrato RD](../../docs/entregas/rd/CONTRATO-RD-V0.1.md) e [retorno C01](../../verification/rd-c01/RESULTADO-RD-I1-C01.md). Critérios, fontes, testes e esperados não foram alterados pelo auditor.

## Versão, deltas e preservação

Branch main, sem HEAD/commit (exit 128); Git/status/diff e inventário fixados antes da verificação, em `/tmp/compass-audit-rd-c01-_ncyjd5b/`. Resultados não atribuídos a commit inexistente.

| Referência | SHA-256 |
|---|---|
| OS-RD-I1-C01 v1 | `4ea245f6137b41e4c33ca279360e8bbb4d40ca60dd334a99f06ffcc37b44bae6` |
| Resultado C01 | `e507f97b915430c9b563921a4450ef6fc2cf00fbb685cb85cd4a625f45610637` |
| Proveniência C01 | `57d83dee09befb2bff33e9af7d73ad569e865f801f7a650991b61fe6322bb247` |
| CLI corrigida | `864c93850e8962508858591cc112cedacfdcb5eba51c2e47140d8802b29f1395` |
| Teste CLI corrigido | `96ea97897ccee2460f40c930a1834af274cfb9582fdff089e5e551da9de7a0b0` |
| Binário anterior | `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d` |
| Binário corrigido entregue/testado | `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8` |

Hashes externos informados conferem. Baseline do executor `/tmp/rd-c01-baseline-zf5_t_8_/inventory.json` autenticada pelo SHA `81c2b6d36a705ace727aeb263244a1ac294936ebb227a787dd7ff5b5510ee69b`, igual ao mapa embutido. Comparadas as cópias `files/`: somente main.rs e tests/search.rs mudaram. Os testes anteriores são prefixo byte a byte do arquivo atual. **187 arquivos preexistentes preservados**, cinco novos autorizados em rd-c01; verification/rd original, núcleo e contratos intactos.

Recompostos os **41 inputs**: `producer_build=sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`. Somente os dois inputs CLI autorizados diferem da versão anterior. Binários antes/depois estáveis; 81 entradas C2-A1 intactas.

## Prova independente antes/depois

O auditor executou o checker entregue em **cópia isolada**, sem sobrescrever os resultados originais. Contra os bytes do release anterior: **47/48**, exatamente uma falha `BUG-005.format` no caso sem flag real e query separada `--query --json`. Exit 2, stderr vazio e valor invalid_arguments corretos; somente os bytes do transporte eram incorretos. A forma com `=` e os controles com flag real passaram.

No release corrigido: **48/48**. Verificados os modos legível/newline e compacto/sem newline, flag real antes do comando/antes da query/após argumento inválido, opção de caminho sem valor seguida de flag real, erros de domínio, help/version e queries hifenizadas válidas em pacote sintético. Cada registro conserva argv, stdout/stderr, esperado e hashes. Nenhum erro genérico ou falha de montagem foi considerado reprodução correta.

Também foram mantidos os **testes permanentes atuais** em outra cópia e restaurado somente main.rs ao fonte anterior. O teste filtrado retornou 101: **um passou e um falhou** na asserção específica `BUG-005.format`, depois de conferir exit/stderr/valor. Com a CLI corrigida, os dois passam no conjunto de **76 testes Rust**. Essa árvore experimental com teste novo não recebeu o hash do antigo release.

A correção acrescenta uma decisão de modo exclusiva do erro de parsing search: consome o próximo token de `--query`, trata `--query=...` como token único, reconhece flags reais posteriores e respeita `--`. Opções de caminho não herdam o consumo hifenizado de query. Parser tipado, matching, fonte, recursos e decisão histórica Foundation/D0 permanecem preservados.

## Resultado por AC

| Aceite | Veredito | Evidência |
|---|---|---|
| C01-AC-01 | CONFORME | Release antigo 47/48 por BUG-005.format; controle equivalente passa. Teste permanente falha com fonte anterior pela mesma asserção. |
| C01-AC-02 | CONFORME | Release corrigido 48/48; formas equivalentes sem flag legíveis com LF, flag real compacta sem LF, código/exit/stderr invariantes. |
| C01-AC-03 | CONFORME | Dois testes permanentes e controles públicos de literals/flags; parsing inválido não é compensado por leitura de conteúdo. Testes antigos preservados. |
| C01-AC-04 | CONFORME | 76 Rust, fmt/Clippy, RD151/cinco mutações/três medições e regressões Foundation/D0 reproduzidos sem skips finais. |
| C01-AC-05 | CONFORME | Dois deltas/cinco novos, 187 preservados, 41 inputs e build recompostos; núcleo/contrato/verification/rd intactos. |
| C01-AC-06 | CONFORME | Mapeamento BUG-005→RD22/RD24/AC-07, registros antes/depois e artefatos completos, retorno permaneceu pendente até este parecer. |

**RD-AC-07: CONFORME após C01.** Os demais aceites da auditoria anterior mantêm seu resultado, apoiados nas regressões da mesma capacidade. BUG-001–004 e BUG-RD-C-001 não foram reabertos.

## Comandos/resultados do auditor

Nas cópias sob `/tmp/compass-audit-rd-c01-_ncyjd5b/`, target Rust separado; campanhas públicas usam o executável entregue. O checker completo monta laboratórios intactos em temporários; originais preservados.

| Comando/verificação | Resultado |
|---|---|
| `python3 verification/rd-c01/check.py --control-only --binary /tmp/rd-c01-baseline-zf5_t_8_/compass-rd-before` | Exit 1 esperado; 47/48, único BUG-005.format |
| `scripts/cargo-local.sh test -p compass-cli --test search bug005 --locked --offline`, com main anterior/testes atuais | Exit 101 esperado; um passou, um falhou por formato |
| `scripts/cargo-local.sh test --workspace --locked --offline`, fontes corrigidos | Exit 0; **76 testes**, zero falhas/ignorados |
| `scripts/cargo-local.sh fmt --all -- --check` e Clippy workspace/all-targets/offline com `-D warnings` | Exit 0 |
| `python3 verification/rd-c01/check.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; 48/48 e campanhas aprovadas |
| Campanhas intactas lançadas pelo checker | **151 comandos RD, cinco mutações, três amostras; 23 Foundation/três mutações; 253 verificações V1/119 cenários e 11 mutações D0**, sem skips |
| Conferência própria dos JSONs originais e regenerados | 2.494 registros brutos autenticados; falhas antigas somente format, esperados/contagens/identidades consistentes |
| C2-A1 `sha256sum --check --quiet SHA256SUMS`; `git diff --check` | Exit 0 |

Socket Unix dos testes/campanhas executado com permissão técnica; nenhuma rede ou acervo real. Não houve build release adicional do auditor: foram autenticados os inputs e usados os bytes entregues; a compilação dos testes em cópia não é anunciada como reprodução binária idêntica do release.

Três amostras novas do auditor: RSS **12600/12604/12564 KiB**; wall **117745728/99911708/88913472 ns**. Em todas, 1000 matches conhecidos/20 entregues, leitura 4096000, reserva 4097000, peak lógico 14690378, stdout idêntico. São medidas descritivas locais, cache não controlado, sem SLO/garantia contra OOM.

## Rastreabilidade e fechamento

| Evidência em `/tmp/compass-audit-rd-c01-_ncyjd5b/` | SHA-256 |
|---|---|
| `controls-before.log` | `913edbb33e1da53f4f8387012b6016ab006ed1b1930550a50a29e5c7fa25a445` |
| `controls-after-campaigns.log` | `917aa3812467e8646e74b17452a462045e13a9d7037a536a53c8f586c71a06ba` |
| `rust-before.log` | `d4334013a39df833651d8c92245b8e85f3906c230eb8a47556a914b6c7064232` |
| `workspace.log` | `74b94655a7491026462a0d59874c8b97f3f4a228ced22e670c05cc58d327dfe4` |
| `artifacts.json` | `b8b7907256268ae9608c8fcadce49b2c77f919463177f0c05f2bd07e97f24c74` |

| ID | Tipo | Estado | Tentativas | Fechamento |
|---|---|---|---|---|
| BUG-005 | implementação/P2 | **verificado** | 1 | Mesma falha reproduzida no antigo, corrigida no novo; controles e regressões aprovados |

Os **195 arquivos locais inventariados** permaneceram idênticos durante a auditoria, incluindo executável e evidências originais. Somente este parecer foi acrescentado antes da etapa separada de coordenação/arquitetura. Temporários estão sujeitos à retenção do sistema; provas e hashes essenciais estão descritos aqui e nos artefatos entregues. Nenhuma correção adicional ou capacidade posterior foi implementada.
