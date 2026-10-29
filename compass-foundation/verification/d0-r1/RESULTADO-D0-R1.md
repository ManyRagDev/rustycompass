# Resultado — OS-D0-R1 v1

**IMPLEMENTADO_PENDENTE_VERIFICACAO** — 19/10/2026, America/Sao_Paulo.

Executadas exclusivamente a OS-D0-R1 e seu CONTRATO-ACEITE-D0-R1, ambos v1. Skill executor-blindado v2.0; execução sequencial, um escritor. Fluxos públicos M/L, quatro negativos e seis amostras passaram. Este retorno encaminha os artefatos para conferência independente; não concede aceite final de R1.

## Baseline e fronteira

Branch `main`, **sem HEAD/commit**: `git rev-parse --verify HEAD` retornou 128, como esperado. O índice está vazio; os 34 arquivos Foundation já apareciam como adições no diff da árvore e os demais diretórios como não rastreados. Esse diff preexistente não é atribuído à R1. `git status --short --branch`, diff binário, diff cached e índice foram capturados antes da primeira escrita. Diff, índice, branch e ausência de HEAD continuaram idênticos ao fim.

Inventário inicial de **165 arquivos**: fontes, contratos, documentos, V1/I1/evidências anteriores, executável e referências externas preservadas. Cópias e registros completos em `/tmp/compass-r1-baseline-kmw6vsx5/`; SHA-256 de `inventory.json`: `34cb0dc32a09db6c684d466a59fcc87631cdf8a7f6bebabd3d6004e9208c2a67`. O mesmo mapa caminho→SHA está embutido nos três inventários idênticos de [provenance.json](provenance.json). Uma primeira tentativa de inventário falhou ao relativizar o caminho externo; foi corrigida com `os.path.relpath` antes de qualquer escrita no projeto ou chamada ao sensor. Não houve patch de produto ou esperado por esse erro de montagem.

| Identidade conferida antes/depois | Valor |
|---|---|
| Binário `target/release/compass` | `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f` |
| Produtor recomposto dos **37 inputs** e emitido pela CLI | `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676` |
| Política default emitida | `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5` |
| Índice C2-A1, 81 artefatos íntegros | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| Auditoria C01 CONFORME herdada | `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3` |

Os 37 caminhos, tamanhos, hashes e enquadramento do hash do build estão em `provenance.json/baseline/build`. O adaptador I1 foi usado somente por leitura, em `verify(ROOT)`: conferiu tar histórico, V1 e conjunto congelado. Não executou `run`, `mount` ou preparação. Os 34 fontes **históricos** foram autenticados dentro do tar, sem exigir igualdade indevida com os fontes I1 atuais.

Escritas: somente os **nove arquivos novos autorizados**, todos em `verification/d0-r1/`: `run.py`, `fixtures.json`, `inputs.json`, `reports.json`, `integration.json`, `resource-profile.json`, `coverage.json`, `provenance.json` e este resultado. Os próprios outputs anteriores à execução foram copiados, com hashes, para o diretório `prior_outputs.path` registrado na proveniência. Produção, executável, contratos, V1, I1 e evidências anteriores permaneceram íntegros. Não houve build, instalação, commit, operação remota, deploy ou mensagem a outro chat.

## AC → evidência

| Aceite | Resultado desta execução e evidência |
|---|---|
| R1-AC-01 | ATENDIDO PARA CONFERÊNCIA. `provenance.json`: 165 hashes preservados, 37 inputs/produtor, binário estável, Git antes/depois, OS/contrato e sete hashes dos outputs anteriores ao retorno. Nove caminhos autorizados. |
| R1-AC-02 | ATENDIDO PARA CONFERÊNCIA. `integration.json/self_test_before_sensor`: receitas/linhas/comprimentos, comparação, LCS, frações/decimais, Wb/W, bytes/células. Self-test executado antes da CLI; `runtime_executed=false`, zero invocações de sensor nesse modo. Cópia temporária com C=661 rejeitada por **`M.aggregate_numerator: '660' != '661'`**. SHA da fixture permaneceu idêntico. |
| R1-AC-03 | ATENDIDO PARA CONFERÊNCIA. `integration.json/loads` e 24 comandos: `init`, snapshots reais S0/S1, empacotamento explícito, `compare`, D0 e repetição sem árvore viva para M/L. Cada relatório validado pelo V1 independente e por literais/equações R1. Inputs externos preservaram bytes/metadados durante cada amostra. |
| R1-AC-04 | ATENDIDO PARA CONFERÊNCIA. `loads/L/negative_controls`, comandos e stdout integral em `reports.json`: quatro rejeições específicas e quatro restaurações ao relatório válido byte a byte. Detalhes abaixo. |
| R1-AC-05 | ATENDIDO PARA CONFERÊNCIA. `resource-profile.json`: três amostras M + três L, GNU time bruto individual e wall, ambiente/cache/limites, ledger S/Wb/W separado de RSS e custo lógico/físico de disco. Nenhum SLO aplicado. |
| R1-AC-06 | ATENDIDO PARA CONFERÊNCIA. `coverage.json`: **E01–E55 HERDADO_VERIFICADO**, com IDs/testes/pointers/hashes e versão C01; RSS de E46 marcado **EXECUTADO_R1**. E35/E43/E46/E49 e BUG-004 reconciliados com testes permanentes auditados. Nenhuma pendência obrigatória mascarada; HIP-I1-01 permanece observação. |
| R1-AC-07 | ATENDIDO PARA CONFERÊNCIA. `inputs.json`: quatro pares manifesto/descritor brutos + variante negativa do binding; `reports.json`: 14 saídas brutas completas. Base64, tamanho, SHA, IDs, exits e vínculos conferidos após limpeza. Modalidades self-test/runtime e rejeição contratada estão separadas de erro do laboratório. |

## Fluxos e negativos executados em R1

M observou 1.000 arquivos e 4.096.000 bytes por snapshot, em 100 diretórios; as cinco adições ficaram em `dir-000`. Compare: unchanged=985, modified=10, removed=5, added=5, demais=0. D0: **complete/exit 0**, 1.005 candidatos/resolvidos; **660/128000 = 0.005156**. Os dez pares modificados tiveram LCS=63; unchanged, LCS=64; adições/remoções, LCS=0. Exclusão `.compass` preservada fora da massa.

L observou um arquivo de 20.000 bytes/2.000 linhas por snapshot. Compare: modified=1, demais=0. D0: **complete/exit 0**, n0=nt=2000, l=1999, a=r=1, c=2, m=4000, **2/4000 = 0.000500**. A prova independente usa linhas únicas e a mesma sequência de 1.999 linhas comuns, além do cálculo DP do V1.

Pacotes baseline/current são raízes físicas distintas, sem hardlinks entre objetos ou com o acervo. Somente bytes observed, conferidos por receita/hash/tamanho, foram copiados; nenhum conteúdo excluded foi copiado. Os manifestos foram copiados integralmente para fora da árvore viva antes de removê-la. D0 repetiu os mesmos bytes sem a árvore, em ambas as cargas. Somente árvores criadas pelo próprio runner foram removidas.

| ID de cenário L | Exit e verificação específica |
|---|---|
| `L-no-baseline-source` | 1; unavailable; baseline not_attempted/source_missing; current verified. |
| `L-object-missing` | 1; unavailable; baseline failed/object_missing; current verified. |
| `L-object-tampered` | 1; unavailable; baseline failed/hash_mismatch; tamanho conservado, EOF comprovado, size_check=true/hash_check=false; current verified. |
| `L-binding-mismatch` | 2; exatamente `{"error":{"code":"incompatible_inputs","message":"incompatible_inputs"}}`, sem campos de relatório. Apenas snapshot_file_sha256 do descritor foi alterado para 64 zeros. |

Nos três relatórios indisponíveis: `calculation` ausente; value/line_mass/zero_basis/known_subset=null; motivos globais `[no_text_units, unresolved_units]`. Cada negativo foi restaurado individualmente, com hashes recuperados e controle válido idêntico ao `L-sample-1`. Nenhum timeout, erro genérico ou falha de setup foi contado como rejeição correta.

SHA-256 dos stdout representativos, iguais nas três amostras, no controle sem árvore e nas restaurações aplicáveis:

- M: `21c19d13aea70a2827590a0e96a909eeae13e0ed3e01027cf4b4212b4e5faf23` — 1.601.015 bytes.
- L: `9db4222efe4342398257ed7575451c95e02820521cf8dcb8743224c8750db940` — 6.950 bytes.

## Seis medições novas

GNU time lança o processo nativo: `-q -f "%U %S %M %e %x" -o <temporário> <binário> d0 ... --json`. RSS é do filho CLI individual. Wall inclui o lançamento do wrapper; não inclui o verificador Python posterior. CPU/elapsed têm resolução impressa de 0,01 s; zeros medidos foram mantidos. Os bytes originais do medidor, hashes e unidades estão no JSON.

| Amostra | CPU user (s) | CPU system (s) | RSS máximo (KiB) | elapsed GNU (s) | wall (ns) | Exit |
|---|---:|---:|---:|---:|---:|---:|
| M-1 | 0.08 | 0.04 | 30272 | 0.12 | 124188174 | 0 |
| M-2 | 0.07 | 0.05 | 30024 | 0.12 | 127706849 | 0 |
| M-3 | 0.08 | 0.04 | 30088 | 0.12 | 124979075 | 0 |
| L-1 | 0.01 | 0.00 | 3748 | 0.01 | 15098516 | 0 |
| L-2 | 0.01 | 0.00 | 3800 | 0.01 | 14243022 | 0 |
| L-3 | 0.01 | 0.00 | 3880 | 0.01 | 16912894 | 0 |

Ambiente: Linux 7.0.11-76070011-generic, x86_64/glibc 2.39; AMD Ryzen 5 8500G, 12 CPUs lógicas; MemTotal=31979304 kB, MemAvailable=14603180 kB no início; Python 3.12.3. `/usr/bin/time --version` retornou `time (GNU Time) UNKNOWN`, exit 0, saída integral preservada. Cache não controlado, sem flush ou alegação de cold-cache; dados recém-gerados, execução sequencial, sem compilação iniciada nesta etapa. Limites default completos no perfil; max_duration_ms=null. Guard de 60 s por subprocesso protege o laboratório e não é deadline/SLO do produto.

| Recurso lógico, em bytes salvo células | M | L |
|---|---:|---:|
| Objetos lidos | 8192000 | 40000 |
| Objetos reservados | 8194000 | 40002 |
| Células reservadas | 40960 | 4000000 |
| S, com bytes brutos e N reais | 12390536 | 118272 |
| Maior Wb / W | 81924 / 86540 | 145540 / 289548 |
| logical_memory_peak = S + maior W | 12477076 | 407820 |

S=8×(manifestos+descritores)+4096×N+65536; N=2204 em M e 6 em L. Unidades unilaterais M também conferidas: Wb=73732, W=75788. Ledger de admissão não é RSS nem garantia contra OOM.

| Disco dos arquivos regulares | M lógico / alocado (bytes) | L lógico / alocado (bytes) |
|---|---:|---:|
| Dois manifestos externos | 411499 / 417792 | 2842 / 8192 |
| Dois descritores | 678 / 8192 | 678 / 8192 |
| Objetos das duas raízes | 8192000 / 8192000 | 40000 / 40960 |
| Pacotes totais (objetos + descritores) | 8192678 / 8200192 | 40678 / 49152 |

Alocação obtida por `st_blocks×512`; exclui metadados de diretório, corpus vivo, `.compass`, compilador e caches de dependências. Relatórios circularam por pipe, sem arquivo próprio alocado; seus tamanhos lógicos estão acima e seus bytes completos permanecem em base64 em `reports.json`. Não se atribui a esse stdout a alocação do contêiner JSON/base64.

## Herança e reconciliação

**Não reexecutados em R1:** 64 testes Rust, 23 testes Foundation CLI, 253 verificações V1, três mutações Foundation e 11 D0, fmt/Clippy/release e intervenção pública sincronizada BUG-004. São provas herdadas da auditoria C01 identificada acima, com binário/37 inputs/hashes conferidos nesta etapa. A V1 histórica permanece intacta, inclusive seus campos preparados/runtime_pending; a matriz R1 explica a resolução sem regravar a matriz antiga.

- E35: herdados `synchronized_sigint_after_first_unit_before_next_and_before_publication`, `deadline_before_publication_and_priority_over_error` e `interrupt_precedes_deadline_and_pending_error`; smoke público de sinal é prova distinta, já auditada.
- E43: herdados testes permanentes de substituição de objeto/nome/componente/pós-EOF e ancestral do pacote. BUG-004 usa os três testes `bug004_*` de `d0/mod.rs` e a prova pública antes/depois da auditoria C01. Não se usa o antigo parecer NAO_CONFORME como aceite do defeito corrigido.
- E46: herdados checkpoints de chunk/LCS, preflight antes do domínio e ledger/reservas; **RSS é evidência nova EXECUTADO_R1**, seis amostras com bytes brutos do medidor.
- E49: herdado `e49_u64_and_checked_equations`, incluindo extremos/empates. Não se afirma construir esses extremos pela CLI sob limites default.

`coverage.json` conserva caminhos, hashes, pointers dos testes V1 e fontes dos testes permanentes, produtor/binário e parecer auditado. Temporários antigos não são a única sustentação. HIP-I1-01 continua observação contratual separada; nenhum patch ou obrigação nova foi criado.

## Comandos e conferência dos artefatos

Executados na raiz de `compass-foundation`:

```sh
python3 verification/d0-r1/run.py --self-test
python3 verification/d0-r1/run.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Todos retornaram 0. O self-test também roda no início do modo binary, antes da primeira chamada CLI. Os **24 argv efetivos**, cwd temporário, exits esperados/reais, stderr, hashes e wall constam de `integration.json/commands`; os dois comandos novos do runner são entregues nesta OS. Nenhum comando de suíte herdada foi executado.

Conferência adicional somente de leitura, após limpeza: decodificou/recalculou hashes/tamanhos dos **14 stdout**, dos **cinco registros de input** e dos **seis medidores**; validou binding manifesto↔descritor↔source_ref, variante negativa, hashes de todas as referências da cobertura, 24 exits, 55 IDs, sete hashes dos outputs e os 165 protegidos. Resultado PASS. Repetida integridade das 81 entradas C2-A1 e `git diff --check`, exit 0. Não houve divergência de produto observada nesta etapa.

| Arquivo R1 | SHA-256 |
|---|---|
| `run.py` | `f97882cb6090c7b363b7358068c760d52bf7cf892a72f3a049b8025c872a6a5c` |
| `fixtures.json` | `53f06546adc49940f9b2a54ab83dc3bf271309bcb403c479bcd89d900f3b2b7f` |
| `inputs.json` | `2fc60964464f44a4bde7c96db8fff60aec585242001e322f50b5e64b001c46e3` |
| `reports.json` | `00f9d98f3cd8bf7faa9434891c36943b97dc0702844e16ba7458cc1a415feba1` |
| `integration.json` | `d24807d03a29247bbc0f2c08e1148f9fdfad447254683b09fc6642b9a32554aa` |
| `resource-profile.json` | `1b92f433a0b669d8574b248de5aeaeea93706313883e6556f1276a413b71a097` |
| `coverage.json` | `3393e8fbf6fcae85b04bfcb8407e0e42d6f51a2ff68e5ee34a2607bbb7c3e52d` |
| `provenance.json` | `beb99704a1ecb0f55525aae327d03222025884a7734bae8f6fac5fdcef0e13fb` |

A proveniência exclui seu próprio hash e este retorno para evitar ciclos; o hash final deste resultado será informado na mensagem de entrega. Receitas/digests dos corpora estão fixados nas fixtures, self-test e perfil. Os objetos sintéticos são regeneráveis; os diretórios runtime foram limpos, e manifestos/descritores/relatórios permanecem integralmente recuperáveis dos JSONs. As cópias iniciais e de outputs em `/tmp` têm retenção temporária e não substituem os artefatos do projeto.

Limites: medição descritiva de uma máquina, seis invocações, sem promessa de escala, latência, superioridade ou cache frio. As verificações estáticas Python não demonstram corridas por si; essas provas são herdadas dos testes permanentes/auditoria na mesma identidade C01. A estabilidade é cooperativa, sem atomicidade global. Não houve revisão independente dos artefatos R1 nesta execução; esse é o próximo ato do rito, fora desta OS. Nenhuma pendência obrigatória conhecida ficou NAO_VERIFICADO; nenhum BUG foi fechado pelo executor.

Uso compartilhado consultado: 18% → 19% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta OS.

**IMPLEMENTADO_PENDENTE_VERIFICACAO**
