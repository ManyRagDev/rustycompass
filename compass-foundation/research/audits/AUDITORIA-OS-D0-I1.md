# Auditoria de entrega — OS-D0-I1 v1

**Veredito: NAO_CONFORME. BUG-004 confirmado: o compartilhamento de uma leitura pode ignorar a instabilidade da referência do segundo pacote e publicar um resultado integral.** A campanha existente passou, mas não exercita essa combinação. A correção e sua conferência são necessárias antes do aceite da I1. R1 não foi iniciada.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo `entrega`. Escopo: [OS-D0-I1](../../docs/entregas/d0/OS-D0-I1.md), seus 11 ACs e [resultado do executor](../../verification/d0-i1/RESULTADO-D0-I1.md), contra o contrato C2-A1 [ativado](../../docs/entregas/d0/ATIVACAO.md). Nenhuma alteração de código, contrato, esperado ou critério de aceite foi feita nesta auditoria.

## Versão concreta e preservação

Branch `main`, **sem HEAD/commits**, com 34 adições Foundation locais e diretórios não rastreados. Os resultados pertencem aos hashes abaixo, não a um commit inexistente.

| Referência | SHA-256 |
|---|---|
| OS-D0-I1 v1 liberada | `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4` |
| Resultado do executor I1 | `8c12b18b927cc03c0a7f861ea33facaece13aea302fe91573a58ab1d2e942f42` |
| Índice C2-A1, 81 entradas | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| Contrato D0 ativo | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| `d0/source.rs` | `eaa6a88427b7b894b0f21d4a61572caa74ae0f0d1b106721827533c96b0e44a1` |
| `d0/report.rs` | `b118fa99fd95a67a75ed37378f549b7afcd77422803c7f3ffb60f72e2263e13b` |
| `verification/d0-i1/run-v1.py` | `94d9ada79741fe38121e2884ee964394ba2c7073f1a2805234f5d51bb2c13298` |
| Executável entregue `target/release/compass` | `f3a0153b1ecdc026743c96e0e3595749ac072d0586ae13fae25213f78d47662f` |

Antes dos testes foi fixado inventário de **158 arquivos**, incluindo resultados ignorados, executável e documento principal externo. Digest do inventário: `a09d33f58a289c41e3367952432eaae1d4f456ef7586d66ae4f05b82a94d91c1` — SHA-256 de `json.dumps(mapa_caminho_sha256,sort_keys=True,separators=(',',':')).encode()`.

Os 141 arquivos da baseline do executor, em `/tmp/compass-d0-i1-gpwhq9td/before.json`, foram confrontados com a árvore atual. Somente os quatro existentes autorizados mudaram: README, CLI, `lib.rs` e `build.rs`; seus deltas são pertinentes. Os novos arquivos pertencem à lista fechada. Os outros 30 históricos, a V1 inteira, contratos, schemas, gabaritos, documentos centrais e auditorias anteriores foram preservados. O tar contém os 34 originais e passa na verificação de membros/hashes. Conferidas também as 81 entradas congeladas.

Testes e provas auxiliares ficaram em `/tmp/compass-audit-i1-0aiuhfud/`. A cópia do executável entregue chama-se `submitted-compass`; foi ela que recebeu a campanha V1, os 90 casos próprios e as intervenções sincronizadas pela CLI. Nenhuma compilação ocorreu enquanto esse artefato era testado.

## Verificação executada pelo auditor

Os comandos foram executados na cópia temporária de `compass-foundation`, com o wrapper/toolchain e dependências locais. As saídas originais do executor não foram sobrescritas.

| Comando ou prova | Resultado observado |
|---|---|
| `scripts/cargo-local.sh test --workspace --locked --offline` | Primeira execução no sandbox falhou em `UnixListener::bind`, EPERM, `source.rs:479`; não foi aprovação nem defeito atribuído ao produto. Repetição com permissão técnica local: **61 testes**, zero falhas/ignorados. |
| `scripts/cargo-local.sh fmt --all -- --check` | Exit 0, sem escrita. |
| `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings` | Exit 0. |
| `scripts/cargo-local.sh build --release --workspace --locked --offline` | Exit 0; nova compilação em diretório temporário. |
| `python3 verification/reference/acceptance.py --binary /tmp/compass-audit-i1-0aiuhfud/submitted-compass` | **23 testes**, OK. |
| `python3 verification/reference/mutations.py --binary /tmp/compass-audit-i1-0aiuhfud/submitted-compass` | **3 mutações rejeitadas**, zero skips. |
| `python3 verification/d0-i1/run-v1.py --binary /tmp/compass-audit-i1-0aiuhfud/submitted-compass --cached-dependencies /home/many/.cache/uv/archive-v0` | **253 verificações**, incluindo os **119 cenários runtime** repetidos; **11 mutações** rejeitadas por suas asserções; zero falhas/skips. Hash do binário inalterado. |
| `independent-math.json` | **90 casos adicionais pela CLI**, seed 47813: LCS por enumeração de subsequências, normalização própria, BOM/CRLF/CR/Unicode/vazios e agregado com massa seis de unchanged. Todos passaram; nenhuma equação esperada veio de Rust. |
| `adapter-probes.py` | Baseline válida passa; corrupção do tar, de membro, caminho `../escape`, fonte V1 e fixture são rejeitados pelas verificações específicas. `/usr/bin/false`, como controle sem D0, provoca retorno não zero do wrapper. |
| `public-races.py` | Seis controles sincronizados no executável entregue; confirmam BUG-004 e a observação separada HIP-I1-01 abaixo. |
| `sha256sum --check --quiet SHA256SUMS`, na pasta C2; `git diff --check` | Exit 0. Arquivos não rastreados conferidos pelo inventário. |

O laboratório em árvore histórica montada pelo adaptador efetivamente executa o **binário I1 real** por caminho absoluto. Não houve mock de sensor, captura de esperado da produção, monkeypatch de V1 ou alteração dos seus JSONs para apagar pendências. As provas internas de checkpoints foram lidas e executadas nos testes Rust; o smoke público não foi usado como prova de um ponto interno exato.

## Proveniência

Recomputados independentemente os **37 inputs**: coleta Foundation existente, nove novos fontes/testes Rust e sete inputs normativos D0 explícitos. Domínio, ordem, comprimentos u64 little-endian e bytes reproduzem:

`producer_build=sha256:f532e4195ebce357e910b1e53a6b4a4aff334f8575c0a768569e06fc03aadb69`.

Esse produtor foi emitido pelo executável entregue e pela compilação do auditor. A política default permanece `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`. O novo executável compilado na cópia tem SHA-256 `032c046b084e7dd5d621894dbf396ab0e4ad3df646b81a26d535b44ba0913bf2`; não foi alegada reprodução binária idêntica, nem atribuídos ao executável original os bytes desse rebuild. A campanha pública citada acima usou o hash entregue `f3a0153b…47662f`.

## Resultado por AC

| AC | Parecer | Fundamento |
|---|---|---|
| I1-AC-01 | CONFORME | Baseline, lista fechada, 30 intactos/quatro deltas, V1, 81 artefatos e tar conferidos; negativos próprios do adaptador mantêm proteção e propagação de falhas. |
| I1-AC-02 | CONFORME no escopo conferido | Testes de transporte/perfis/preflight e campanha real passaram. Hook demonstra S_manifest antes do domínio, com controle positivo; binding bruto não canônico e fonte presente inválida no vazio conferidos. |
| I1-AC-03 | **NAO_CONFORME** | Trocas comuns de objeto/nome/componente e ancestrais são detectadas nos testes existentes, mas a referência `current` deixa de ter sua estabilidade conferida quando recebe a leitura compartilhada: BUG-004. |
| I1-AC-04 | **NAO_CONFORME** | EOF/prefixo/vazios/reservas e isolamento entre raízes distintas passaram; compartilhamento admite uma referência instável no segundo lado. Não é autorização para cobrar duas leituras em todo caso compartilhado. |
| I1-AC-05 | CONFORME no escopo conferido | 44 cálculos literais, dez relatórios físicos, campanha E01–E55 e 90 casos próprios conferem matemática/normalização/unchanged. E36/E51–E55 e regressões BUG-001/002/003 permanecem aprovadas. |
| I1-AC-06 | CONFORME no escopo conferido | Recursos, precedências, continuação, igualdade zero células e vetores diretos E49/u64/excesso passaram; código usa linha DP compatível com W. |
| I1-AC-07 | **NAO_CONFORME** | Transporte, ordem e matemática passaram, mas BUG-004 produz `complete`/valor integral com check `current=verified` cuja associação de referência não foi reconferida. Schema e modelo estático não provam essa estabilidade histórica. |
| I1-AC-08 | CONFORME no escopo conferido | CLI, JSON sem newline, perfis, modos e testes de barreira/flag/checkpoints passaram; prioridade cancelamento/deadline/erro verificada. Sem promessa de prazo rígido. |
| I1-AC-09 | CONFORME quanto à execução da campanha | Sensor real testado, 253 verificações e 11 mutações, hashes estáveis e V1 intacta. A campanha aprovada não cobre BUG-004; o mapeamento das provas de concorrência não permite considerá-las universais. |
| I1-AC-10 | CONFORME | Testes, regressões Foundation, fmt/Clippy/release e identidade de fontes/produtor conferidos, sem inventar commit ou identidade binária entre diretórios. |
| I1-AC-11 | CONFORME quanto aos limites declarados | README/retorno distinguem implementação, auditoria/R1, ledger e RSS não medido. A alegação do executor de ACs atendidos é revista pelos achados deste parecer. |

## BUG-004 — compartilhamento ignora a estabilidade do segundo caminho

**Tipo:** `implementacao`. **Estado:** `confirmado`. **Impacto:** bloqueia o aceite I1 pelos AC-03/04/07. **Tentativas de correção:** 0. **Responsável:** executor, após ordem corretiva delimitada. **Versão:** hashes deste parecer.

Requisito: contrato C2-A1 §2 exige associação nome↔handle e que substituição/instabilidade prevaleça sobre hash coincidente; compartilhamento exige prova válida para ambos os lados. A OS exige troca sincronizada de componentes e instabilidade bloqueando cálculo.

Evidência de código: `source.rs:307–308` decide compartilhamento por identidade da raiz, descritor bruto e revisão. Em `report.rs:116–118`, o segundo lado recebe cópia do check/dados da baseline. Não é feita a reconferência da sua própria cadeia de referência. A leitura da baseline verifica apenas `self.root` em `BoundFile::stable` (`source.rs:227–236`).

Reprodução no **binário entregue**, em arquivos artificiais:

1. Mesmo manifesto bruto nos dois lados, revisão `a\n` e pacote válido `T/package`; descritor vinculado ao manifesto exato. Criar também diretório `T/alias-component`.
2. Baseline usa `T/package`; current usa `T/alias-component/../package`. As duas referências abrem a mesma raiz física, mesmo descritor e objeto. `..` está na referência CLI da raiz, não em `path_key` ou nome de objeto; a referência é aceita e não atravessa symlink.
3. Depois de capturados os descritores/metadados e imediatamente antes da primeira leitura do objeto, pausar o processo. Renomear `alias-component`, criar outro diretório no mesmo nome e liberar a leitura. O componente tem outro inode; a raiz final e os bytes do objeto continuam idênticos.
4. Observado: **exit 0, `complete`, integral `0/2`, os dois checks `verified`, sem motivos**. Reserva três bytes, leitura dois, zero células. A referência current não recebe a detecção de instabilidade implementada no próprio adaptador.

O controle estável passa. Com a referência que sofreu a mesma intervenção na **baseline**, o mesmo binário detecta `source_unstable`, retorna exit 1/`unavailable` e não publica integral. Portanto o problema é a omissão da conferência na segunda referência compartilhada, não ausência de suporte à entrada, hash alterado, erro genérico ou falha do instrumento.

A sincronização usou dois pipes e uma biblioteca `LD_PRELOAD` de instrumentação: registra o fd retornado pelo `openat2` de `objects/<hash>` e pausa somente sua primeira chamada `read`. Não altera bytes, retornos de leitura, metadados ou relatório; o processo pai faz a intervenção no filesystem. É uma barreira determinística pela CLI, sem flags públicas de teste. A mesma lacuna foi observada também compilando os módulos originais intactos em harness auxiliar com seu hook privado; a prova decisiva acima usa o executável entregue.

**Fechamento mínimo:** regressão com a intervenção na referência current deve deixar de declarar essa referência verificada e impedir integral; preservar o controle estável, a detecção na baseline, os fatos EOF realmente verificados, a reserva/leitura compartilhada legítima e as demais regressões. Conferir todas as referências participantes da prova compartilhada, sem desabilitar indiscriminadamente o compartilhamento ou enfraquecer contratos/esperados. O fechamento será verificado no mesmo BUG/AC e nos impactos da correção.

## HIP-I1-01 — abrangência dos metadados de diretórios

**Comportamento confirmado; conflito contratual tratado como hipótese, sem nova obrigação automática de implementação.** Responsável por eventual esclarecimento: arquiteto, antes de qualquer correção desse ponto.

Na mesma barreira, alternar apenas o bit de escrita de grupo do arquivo-objeto mantém os bytes/hash e é detectado como `source_unstable`. Alternar o mesmo bit em `objects/` ou na raiz do pacote altera modo/ctime, conserva acessibilidade e identidade, mas produz `complete`, checks `verified` e sem motivo. `DirectoryBinding::stable` (`source.rs:85–91`) compara identidade; `Root::stable` (`source.rs:176–187`) também não conserva/compara o conjunto completo de metadados. `unchanged`, com modo/timestamps/tamanho, é aplicado ao arquivo lido.

O §2 lista os metadados sem distinguir explicitamente arquivo lido de diretórios de resolução. A intenção de impor o mesmo conjunto a todos os diretórios, inclusive quando nenhuma associação de nome foi alterada, precisa ser esclarecida para transformar essa observação em um achado de implementação. Este parecer **não cria essa exigência por preferência** e não usa HIP-I1-01 como fundamento adicional do veredito. BUG-004 independe dessa interpretação: há substituição comprovada de componente e conferência omitida no segundo lado.

## Evidências e reexecução

Auxiliares em `/tmp/compass-audit-i1-0aiuhfud/`, sujeitos à retenção de temporários:

| Arquivo | SHA-256 |
|---|---|
| `io_gate.c` | `2e903bd68284a4ffb9b253ab1eb34782baee6b130bfa5c24f97a50bc7fdda29a` |
| `public-races.py` | `35db1acade5ec45e5860472f25e0f6eb60c5d0c6338bb0ab5696b07a73272b65` |
| `public-races.json` | `f257b672db142ee5cff673e898ae4a32cbbadb8d8b2050a3bf1548d28152afa1` |
| `independent-math.json` | `83f257a0dccf4d9e0b03a520ef827fb40974ead5c0ae2f9f39f1a976a5a40933` |
| `adapter-probes.json` | `66d79494e6579457f0c6befa0aeb0edb8bf1e564e73b434a5d67feb1a350b54c` |
| `rust-test-unrestricted.log` | `42ffbc29e9a8f1326046f337799c41f1a0d4e27b1cd8c75f7b61d23404c759c1` |
| `v1-runtime.log` | `0b4b820e939952c53156fa89c26a3a05c00506f2fdff307f0b565a463a463c2a` |

Comandos das provas próprias, enquanto os auxiliares estiverem disponíveis:

```sh
cc -shared -fPIC -O0 -o /tmp/compass-audit-i1-0aiuhfud/io_gate.so /tmp/compass-audit-i1-0aiuhfud/io_gate.c -ldl
python3 /tmp/compass-audit-i1-0aiuhfud/public-races.py
python3 /tmp/compass-audit-i1-0aiuhfud/adapter-probes.py
```

## Limites e encaminhamento

Sem prova de escape/leitura de conteúdo externo ou erro de LCS/aritmética nos casos conferidos. O defeito confirmado é de validade da evidência e disponibilidade integral sob concorrência; não se atribui impacto não observado.

RSS/benchmark e aceite integrado R1 permanecem legitimamente pendentes. Não são defeitos desta I1 nem foram substituídos pelo ledger lógico. A diferença de checksum do rebuild não foi tratada como defeito ou prova de reprodução binária idêntica. A restrição inicial de socket foi superada com permissão técnica para testes locais, sem acesso remoto.

Os **158 arquivos inventariados permaneceram idênticos durante a auditoria**, incluindo código, binário original, V1, resultados do executor, contratos e documentos protegidos. Apenas este parecer foi acrescentado ao projeto; auxiliares ficaram em `/tmp`. Nenhum achado foi corrigido, nenhum contrato corretivo foi criado, nenhum outro chat foi contatado e I1/R1 não foram executadas como novas entregas.

Próximo trabalho: ordem corretiva focada em **BUG-004**, seguida de auditoria do mesmo achado/ACs e regressões afetadas. BUG-001/002/003 não são reabertos. A observação HIP-I1-01 não autoriza patch por conta própria.

Uso compartilhado consultado no início/fim: 15% → 16% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta auditoria.
