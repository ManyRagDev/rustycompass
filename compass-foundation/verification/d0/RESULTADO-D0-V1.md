# Resultado — OS-D0-V1 versão 2

Data: 19/10/2026, America/Sao_Paulo. Intenção: execução.
Skill: executor-blindado v2.0. Estado: **CONCLUIDO para a entrega V1**,
conforme AC-08: ferramentas funcionais, controles aprovados e campanha entregue.
**Homologação do sensor Rust/CLI: NAO_VERIFICADO; runtime pendente de I1/R1.**

## Autorização e baseline

Conferidos `AGENTS.md`, `COMPASS_STATE.md`, OS v2, plano, ativação,
contrato consolidado, índice, schemas/fixtures/evidências pertinentes, auditoria
A1, contrato/schema Foundation e convenções da verificação Foundation.
P1–P4 estão aprovadas e V1 liberada em `docs/entregas/d0/ATIVACAO.md`.
O BLOQUEIO_DE_PRODUTO anterior está resolvido; nenhum contrato foi alterado.

Branch `main`, sem commits/HEAD (`git rev-parse --verify HEAD` falhou como
esperado nessa baseline). Havia 34 arquivos Foundation marcados como adições
locais e `docs/`, `research/audits/`, `research/proposals/` não rastreados.
`verification/d0/` não existia. Não há commit/diff histórico ao qual atribuir
esta entrega. As alterações preexistentes foram preservadas: inventário inicial
em `/tmp/d0-v1-baseline.json`, com 122 arquivos fora de `.git` e `target`,
conferido novamente ao encerrar, sem alteração/remoção de nenhum deles.
O inventário temporário é auxiliar; as identidades duráveis estão abaixo.

| Referência preservada | SHA-256/identificador |
|---|---|
| Índice de 81 artefatos | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| Contrato D0 | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| Auditoria A1 | `9e89ad9217b50f475fafd4dc74ce7edae37345187ca00225cfa8d8f8b9b51c20` |
| Política default | `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5` |

Os 34 arquivos protegidos por `evidence/preservacao.json` coincidem com seus
hashes registrados antes/depois. Os hashes dos gabaritos V1 e da matriz constam
em `results/self-test.json`. Nenhuma recompilação ou nova identidade Foundation
foi produzida; nenhum arquivo Rust/Cargo foi escrito ou importado.

## Arquivos novos e autorização

Todos pertencem à lista fechada da OS, em `verification/d0/`:

| Arquivo | Entrega |
|---|---|
| `oracle.py` | Normalização, LCS por DP, enumeração independente, razões, agregação e células |
| `validate.py` | Transporte estrito, JCS restrito, hashes, schemas, Foundation e recomposição independente dos relatórios de árvores estáticas |
| `cases.py` | 116 montagens temporárias e literais, fontes, manifestos, recursos e erros |
| `acceptance.py` | `--self-test` / `--binary`, guardas, repetibilidade, integridade e evidência |
| `mutations.py` | 11 negativos semânticos específicos e controles positivos, offline ou sobre respostas reais de CLI |
| `fixtures.json` | Literais adicionais e referências/provas de checkpoints anteriores ao uso de Rust D0 |
| `coverage.json` | E01–E55, testes/camadas reais, cenários preparados e responsáveis pela prova restante |
| `ACCEPTANCE.md` | Reprodução, modalidades, interpretação e limites da prova |
| `.gitignore` | Ignora somente `results/` nesta pasta |
| `results/self-test.json` | Evidência gerada offline, 126 verificações aprovadas |
| `results/mutations.json` | Evidência gerada offline, 11 negativos rejeitados corretamente |
| `RESULTADO-D0-V1.md` | Este retorno durável |

`results/acceptance.json` não foi criado: nenhum binário D0 foi executado.
Os esperados não foram extraídos da produção nem do checker C2.

## Aceites

| Aceite | Resultado V1 e evidência | Limite/transferência |
|---|---|---|
| AC-01 | ATENDIDO. Índice autorizado, 81 hashes, política e 34 protegidos conferidos; alteração de cópia de cases detectada como `artifact_hash`. | Originais preservados. |
| AC-02 | ATENDIDO. Matriz completa E01–E55 e verificação de suas referências; 44 cálculos, 23 agregados, 961 pares pequenos por dois métodos; cenários físicos para unchanged, criação/remoção/rename, vazios, extremos A→B→A, bytes e não texto. | Preparação e matemática não equivalem a 55 testes de sensor. |
| AC-03 | ATENDIDO no laboratório. E36/E51–E55 verificam custo zero para igualdade de sequências verificadas, CRLF/LF, saldo e denominador; mesma contagem diferente mantém produto; fontes ausentes/adulteradas impedem cálculo. | Execução desses cenários em Rust/CLI pendente. |
| AC-04 | ATENDIDO no domínio estático do laboratório. Dez relatórios com inputs passam em schema/semântica; comparação, checks, binding bruto, ordem nativa, u64, matemática, disponibilidade, bytes/células e ledger recompostos; negativos schema-válidos rejeitados. | Segurança concorrente/openat2 não é provada pelo modelo Python; barreiras em I1/R1. |
| AC-05 | ATENDIDO como campanha entregue. Modo público `--binary`, temporários, fontes omitidas, binding, objetos, descritor não canônico e compartilhamento, hash antes/depois, exit, JCS, repetição e imutabilidade preparados. | NAO_VERIFICADO contra binário D0. Nenhuma execução Foundation apresentada como D0. |
| AC-06 | ATENDIDO como entrega dos cenários e divisão das provas. Symlinks/componentes, FIFO/socket/diretório, hardlinks, nomes nativos, UTF-8, fronteiras/precedências, EOF/prefixo, saída, deadline e smoke SIGINT preparados. | Corridas, checkpoints e precedência exata SIGINT/deadline exigem sincronização I1/R1; timeout do harness não prova prazo. |
| AC-07 | ATENDIDO offline. Os seis negativos obrigatórios e cinco adicionais passam no schema e atingem asserções semânticas específicas; 11 controles válidos passam antes das mutações. | Negativos sobre respostas reais D0 aguardam release estável. |
| AC-08 | ATENDIDO para V1. Documentação, matriz e evidências distinguem self-test, campanha preparada e execução real ausente. Ferramentas e controles funcionam. | Homologação Rust/CLI, provas internas e recursos pertencem a R1, não iniciada. |

## Comandos efetivamente executados

Ambiente: Linux `7.0.11-76070011-generic`, Python 3.12.3; jsonschema e
suas dependências já disponíveis no cache local, sem instalação.

| Comando/conferência | Resultado final |
|---|---|
| `git status --short`, `git branch --show-current`, `git rev-parse --verify HEAD`, `git diff --stat` | Baseline identificada; `main` sem HEAD; alterações existentes preservadas. |
| `sha256sum` do índice, contrato e auditoria | Todos coincidiram com OS/ativação. |
| `(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)` | Exit 0, antes e depois da implementação; 81 entradas. |
| `python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; `SELF_TEST_PASSED_RUNTIME_PENDING`; 126 verificações, zero falhas; 116 cenários preparados e matriz de 55 IDs. |
| `python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; 11 mutações rejeitadas pela asserção esperada, zero falhas, controles válidos aprovados. |
| `git diff --check` e `git diff --cached --check` | Exit 0. |
| AST Python e whitespace dos arquivos novos não rastreados | Cinco módulos parseados sem erro; JSONs lidos; sem trailing whitespace. Isso complementa, sem substituir, testes comportamentais. |
| Inventário SHA-256 inicial/final de 122 arquivos e hashes dos 34 protegidos | Preservados. |

Os self-tests finais que criam o socket Unix temporário foram autorizados pela
revisão técnica e executados fora do sandbox: dentro dele, `socket.bind()`
retornou `PermissionError: EPERM`. Não houve operação remota.

Durante o desenvolvimento houve falhas do próprio laboratório, sem atribuição
a Rust: a mutação de falso zero inicialmente falhou no schema, antes da asserção
pretendida. Após duas tentativas, foi relida a condição completa do schema de
checks verified e corrigida a montagem da mentira (incluindo source_ref),
sem mudar o validador para aceitar o erro. O socket excedia o limite de caminho
Unix, resolvido vinculando nome curto e renomeando dentro do temporário.
A restrição EPERM do sandbox foi tratada com execução técnica autorizada.
As receitas E11 (vazio criado) e E48 (segundo BOM) foram corrigidas para preservar
os esperados congelados. Os comandos finais acima passaram após esses ajustes.
Não há falha restante mascarada como skip ou aprovação.

## Pendências de integração

- Coordenação deve disponibilizar release D0 estável identificado por SHA-256;
  executar os dois comandos `--binary` documentados, guardar relatório/exit,
  proveniência e estabilidade. Foundation atual não satisfaz o pré-requisito.
- I1/R1: provas sincronizadas de troca de objeto/nome/componente após abertura,
  estabilidade com EOF, checkpoints de chunks/LCS, prioridade SIGINT/deadline,
  cancelamento após p/antes de q e prazo antes da publicação.
- I1/R1: vetores extremos de arredondamento em teste direto do formatador Rust;
  as contas offline de E49 não afirmam viabilidade pela CLI sob limites default.
- R1: regressão Foundation, testes Rust, fmt/Clippy/build e medição de RSS;
  o ledger lógico não é garantia de RSS/OOM nem observação atômica.

Nenhuma OS I1/R1 foi iniciada, nenhum outro chat foi contatado e nenhum
subagente foi usado. Sem deploy, migration, operação remota, instalação ou
mudança dos contratos/esperados originais. A coordenação recebe este retorno
para revisão e integração; não há encerramento de bug de produção nesta V1.

Uso da conta consultado no início/fim: 10% → 11% na janela compartilhada de
10.080 minutos (aproximadamente +1 ponto percentual). Medida arredondada e
compartilhada com outros trabalhos, não contagem de tokens desta entrega.

## OS-D0-V1-C01 / BUG-001

Estado: **IMPLEMENTADO_PENDENTE_VERIFICACAO** — correção implementada e
testada pelo executor, aguardando auditoria focada. A conclusão histórica acima
não representa aceite final; permanece válido o parecer NAO_CONFORME até revisão.

19/10/2026. Baseline: `main` sem HEAD; versão auditada com hashes conferidos na OS C01.

Tentativa de correção #1, com ajuste após regressão local: BUG-001 reproduzido
antes do patch (path curto passa, path de 24 dígitos falha em `u64`). O teste
novo `regression:BUG-001` também falhou sobre o validador auditado, sem usar Rust.
A seleção inicial só por nome revelou a colisão entre o texto
`metric_policy.unknown` e o contador `counts.unknown`; relidos ambos os schemas,
foi corrigido o papel do campo. A regressão e os 11 negativos voltaram a passar.
Um comando auxiliar de documentação foi inicialmente executado no diretório
errado e falhou sem escrever; foi repetido na raiz correta.

C01-AC-01: paths `1111` e `'1'*24` passam em path/snapshot/modelo/report, com
`0/2`, massa dois, decimal `0.000000` e zero células. C01-AC-02: identificadores
numéricos preservados; 49 campos numéricos dos schemas conferidos com u64::MAX
e overflow `u64`, inclusive `counts.unknown`. C01-AC-03: controle positivo e
negativo executáveis, cenários `BUG-001-path-4/24` e referências E24/E50 na
matriz. C01-AC-04: comandos comuns aprovados, runtime ainda pendente.

Verificações efetivas: `acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, **129 verificações, 118 cenários, zero falhas**; `mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, 11 negativos específicos e controles positivos aprovados. Self-test executado com permissão técnica para socket Unix local fora do sandbox.
`sha256sum --check --quiet SHA256SUMS`: exit 0, 81 artefatos;
`git diff --check`: exit 0. Whitespace dos não rastreados e 34 protegidos
conferidos diretamente. Nenhuma operação remota, I1/R1 ou mensagem a outro chat.

Hashes SHA-256 dos arquivos alterados (o próprio retorno é excluído desta
tabela para evitar autorreferência):

| Caminho | Antes | Depois |
|---|---|---|
| `verification/d0/acceptance.py` | `e54b49bf5f63ab73c5c62dd7bd2b92af5e62e3cc251f5f90cf5a15c977c074ae` | `cca88718b4bc0faba932ed654b81eaa63c0a9f5d5a85645fd7f001f36ddb547f` |
| `verification/d0/validate.py` | `4337f4f5ce049557873d5b2827cf2ef47b9cafbd95f2bafd7f032db09278a963` | `80ade03cebd4e39edcba39e529322d361ef62887baee83e79032b6bb24701c88` |
| `verification/d0/coverage.json` | `b2d971f6c46da7672564954d9c043c4f0b6536c2e1d02529bd97bb5a83500fda` | `44cb26981061c7647d8769510722f94a70b0e6380e56fd25a34c5a24698eb08f` |
| `verification/d0/cases.py` | `9aadb76d70ac3c509bfa5d33089a0bb18540d34fd0ff754c18916c0f4dade398` | `d4994bde07e7338c6dc16e1f372d43ba6e3d8b90376118a76d27055838206dbe` |
| `verification/d0/ACCEPTANCE.md` | `96b4fdb693cd1db3cce347e05b16ff9a09fcc74207420a63737b3712c1b076ac` | `c3f48f178656d7b9753521fea39417e3faf5adc0375cef3970e5b584e133b547` |
| `verification/d0/results/self-test.json` | `eda978b2de13ef40ac2762003b6e113abfe5f1901c71897971955f77feb7aa8d` | `f73d54096b5b4b4a42c85fe48bfde4521fe1e90650c5323c32873182ae630c0e` |

Cópias/diff/hashes auxiliares em `/tmp/compass-v1-corrections-vgt05mao`: `C01-before`,
`C01.diff`, `C01-after-hashes.json`. Reversão limitada aos hunks desta
ordem, sem reset/clean, preservando mudanças anteriores. As evidências geradas
continuam offline; nenhum binário D0 foi executado.

## OS-D0-V1-C02 / BUG-002

Estado: **IMPLEMENTADO_PENDENTE_VERIFICACAO** — correção implementada e
testada pelo executor, aguardando auditoria focada. A conclusão histórica acima
não representa aceite final; permanece válido o parecer NAO_CONFORME até revisão.

19/10/2026. Baseline: `main` sem HEAD; árvore acumulada pós-C01, com testes aprovados e hashes abaixo preservados.

Tentativa de correção #1. O teste `regression:BUG-002` foi executado antes
do patch e reproduziu o mesmo achado: `objects` pendente virou `object_missing`.
A correção verifica componentes estáticos proibidos antes de concluir ausência,
sem implementar segurança de produção em Python.

C02-AC-01: links intermediários existentes/pendentes resultam literalmente em
`source_invalid_kind`. C02-AC-02: ausência regular continua `object_missing`;
link direto, FIFO, socket e diretório continuam proibidos na campanha; controle
regular válido passa. C02-AC-03: `abc` bilateral mantém reserva 8, leitura 3,
nenhuma revisão/hash/tamanho inventado na baseline e current verified.
C02-AC-04: relatório com motivo correto passa no schema e em report(); mentira
schema-válida `object_missing` é rejeitada especificamente por `report_units`.
C02-AC-05: cenário e prova E43 na matriz, C01 e comandos comuns aprovados;
TOCTOU/openat2/corridas continuam com I1/R1.

Verificações efetivas: `acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, **131 verificações, 119 cenários, zero falhas**; `mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, 11 negativos específicos e controles positivos aprovados. Self-test executado com permissão técnica para socket Unix local fora do sandbox.
`sha256sum --check --quiet SHA256SUMS`: exit 0, 81 artefatos;
`git diff --check`: exit 0. Whitespace dos não rastreados e 34 protegidos
conferidos diretamente. Nenhuma operação remota, I1/R1 ou mensagem a outro chat.

Hashes SHA-256 dos arquivos alterados (o próprio retorno é excluído desta
tabela para evitar autorreferência):

| Caminho | Antes | Depois |
|---|---|---|
| `verification/d0/acceptance.py` | `cca88718b4bc0faba932ed654b81eaa63c0a9f5d5a85645fd7f001f36ddb547f` | `8f0592d3c37f22b0f27bbcf199c8dbe8eee0ec75ab8e52b007798132cb5869d0` |
| `verification/d0/validate.py` | `80ade03cebd4e39edcba39e529322d361ef62887baee83e79032b6bb24701c88` | `39e8d47f4dd1baa743fdad98589771757dd8f510bcea08fe2239cd157bb64461` |
| `verification/d0/coverage.json` | `44cb26981061c7647d8769510722f94a70b0e6380e56fd25a34c5a24698eb08f` | `f78ee734bc20c29700828f5a6af15b9b222a6b260c705cb8e959c76d364b1c99` |
| `verification/d0/cases.py` | `d4994bde07e7338c6dc16e1f372d43ba6e3d8b90376118a76d27055838206dbe` | `f18c481ad43c6cdfcf85f011507e8667b8f7bcdc22d9ee83b222b99d75324e26` |
| `verification/d0/ACCEPTANCE.md` | `c3f48f178656d7b9753521fea39417e3faf5adc0375cef3970e5b584e133b547` | `bf546b84e0fbc1821e0b396114cef5eb3b196cfdcecfe77fe56aa5e3c1262346` |
| `verification/d0/results/self-test.json` | `f73d54096b5b4b4a42c85fe48bfde4521fe1e90650c5323c32873182ae630c0e` | `d18415be639ed6c389840821e2a8d8d24915c72998f634dc1f017db88fe6c6f6` |

Cópias/diff/hashes auxiliares em `/tmp/compass-v1-corrections-vgt05mao`: `C02-before`,
`C02.diff`, `C02-after-hashes.json`. Reversão limitada aos hunks desta
ordem, sem reset/clean, preservando mudanças anteriores. As evidências geradas
continuam offline; nenhum binário D0 foi executado.

## OS-D0-V1-C03 / BUG-003

Estado: **IMPLEMENTADO_PENDENTE_VERIFICACAO** — correção implementada e
testada pelo executor, aguardando auditoria focada. A conclusão histórica acima
não representa aceite final; permanece válido o parecer NAO_CONFORME até revisão.

19/10/2026. Baseline: `main` sem HEAD; árvore acumulada pós-C02, com testes aprovados e hashes abaixo preservados.

Tentativa de correção #1. O controle `regression:BUG-003` falhou antes do
patch em `bug003_stimulus_bytes`, reproduzindo E52 com bytes iguais. A receita
agora usa diretamente os hexadecimais congelados, antes de preparar os manifestos
e descritores. O teste também roda durante prepare/runtime do cenário E52.

C03-AC-01: objetos E52 com `610d0a` e `610a`; revisões de tamanhos 3 e 2,
hashes distintos, descritores vinculados aos bytes dos manifestos.
C03-AC-02: checks verified, linhas `['a']` nos dois lados, Foundation modified,
medida completa `0/2`, massa dois, decimal `0.000000`, zero células.
C03-AC-03: controle reconstruído de listas normalizadas tem modelo/schema
válidos e D0 zero, mas é rejeitado por `bug003_stimulus_bytes`, sem contar
schema/import como detecção. C03-AC-04: matriz E52 aponta regressão e montagem
física; retirado crédito indevido de byte-hash-schema-vectors, preservados
bytes-E16a/E48 e demais casos. C03-AC-05: C01/C02 e comandos comuns aprovados;
validate.py pós-C02 preservado byte a byte, sem runtime D0.

Estímulo observado por `regression:BUG-003` / `prepare:E52`:

| Lado | Hex | SHA-256 do objeto | SHA-256 bruto do manifesto |
|---|---|---|---|
| baseline | `610d0a` | `8e4621379786ef42a4fec155cd525c291dd7db3c1fde3478522f4f61c03fd1bd` | `7c0abcd8b43cca61f473257cf750c7db363cc71516dbb4a15a8fbb163200ff92` |
| current | `610a` | `87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7` | `193a36b48a52acc069a044a2247963b8b3324d1e47e84b234bf49637482e3ffd` |

Asserções positivas: `bug003_stimulus_bytes`, `bug003_revision_hash`,
`bug003_revision_size`, `bug003_descriptor_binding`,
`bug003_distinct_raw_hashes`, `bug003_equal_normalized_lines`,
`bug003_foundation_modified`, `bug003_verified_bytes` e literais 0/2/zero células.
Controle negativo schema-válido: `bug003_stimulus_bytes`.

Uso da conta nesta sequência: 12% → 12% na janela compartilhada de
10.080 minutos. Diferença visível zero, sujeita a arredondamento e outros
trabalhos; não significa consumo nulo nem contagem exata de tokens.

Verificações efetivas: `acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, **132 verificações, 119 cenários, zero falhas**; `mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0`: exit 0, 11 negativos específicos e controles positivos aprovados. Self-test executado com permissão técnica para socket Unix local fora do sandbox.
`sha256sum --check --quiet SHA256SUMS`: exit 0, 81 artefatos;
`git diff --check`: exit 0. Whitespace dos não rastreados e 34 protegidos
conferidos diretamente. Nenhuma operação remota, I1/R1 ou mensagem a outro chat.

Hashes SHA-256 dos arquivos alterados (o próprio retorno é excluído desta
tabela para evitar autorreferência):

| Caminho | Antes | Depois |
|---|---|---|
| `verification/d0/acceptance.py` | `8f0592d3c37f22b0f27bbcf199c8dbe8eee0ec75ab8e52b007798132cb5869d0` | `d98ce5927217281bf0172fa989c786d55f329ef49b49ee02d221f836370737a2` |
| `verification/d0/coverage.json` | `f78ee734bc20c29700828f5a6af15b9b222a6b260c705cb8e959c76d364b1c99` | `0ed2fc8894852b95c1e55c2cc2609f09f3714db51e64cc0dbc992c9e48afdba3` |
| `verification/d0/cases.py` | `f18c481ad43c6cdfcf85f011507e8667b8f7bcdc22d9ee83b222b99d75324e26` | `319516964ac3bcf0f89093cf09f898cfe5e9d3b188a9cc281379c466969b8c78` |
| `verification/d0/ACCEPTANCE.md` | `bf546b84e0fbc1821e0b396114cef5eb3b196cfdcecfe77fe56aa5e3c1262346` | `3a29a5c07b94b639b3f5fa765e3700141ee1efdaf29c8411e51dc8fbacb66b15` |
| `verification/d0/results/self-test.json` | `d18415be639ed6c389840821e2a8d8d24915c72998f634dc1f017db88fe6c6f6` | `600b47ab4a8b19f5d5981db2ae1f03bffbc87a6f3d998a90cbcc4851d47bcbca` |

Cópias/diff/hashes auxiliares em `/tmp/compass-v1-corrections-vgt05mao`: `C03-before`,
`C03.diff`, `C03-after-hashes.json`. Reversão limitada aos hunks desta
ordem, sem reset/clean, preservando mudanças anteriores. As evidências geradas
continuam offline; nenhum binário D0 foi executado.
