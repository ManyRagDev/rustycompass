# Auditoria de entrega — OS-RD-I1 v1

**Veredito: NAO_CONFORME.** BUG-005 confirmado no transporte de erros de argumentos; RD-AC-07 não atendido integralmente. Os demais aceites foram conferidos sem outro achado obrigatório. RD-I1 permanece pendente de correção e conferência focada; nenhuma capacidade posterior está liberada.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo entrega. Referências preservadas: [OS-RD-I1](../../docs/entregas/rd/OS-RD-I1.md), [contrato RD v0.1](../../docs/entregas/rd/CONTRATO-RD-V0.1.md), [parecer de contrato](AUDITORIA-CONTRATO-OS-RD-I1.md) e [resultado do executor](../../verification/rd/RESULTADO-RD-I1.md). Não houve patch de código, contrato, esperado ou teste durante o parecer.

## Versão e fronteira

`main`, sem HEAD/commit (exit 128). Git/status/diff e inventário fixados antes da auditoria em `/tmp/compass-audit-rd-k2kvztvm/`. Adições Foundation e diretórios não rastreados são preexistentes; os resultados não são atribuídos a commit inexistente.

| Referência auditada | SHA-256 |
|---|---|
| Contrato RD v1 | `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72` |
| OS liberada v1 | `dedbf49a02f8b9c774e4a3b11f1c1be80a28f79901873ae08cd1d2028b02d419` |
| Resultado entregue | `1a0613b0f705e23e1c260011a486a7f0a63189189963b6d94adc9d6b908d1f50` |
| Proveniência entregue | `54ebd07bae46ff450c1fc6eba07ec4038d740523797b55f6bdc7762c5624ba52` |
| Binário entregue/testado | `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d` |
| CLI main.rs | `9338279a2f36151e2806071bdc78848bbf976b9a0caea08abfa774a87cca39c8` |
| Núcleo search.rs | `04a8e696d7fd3739419872aa5b7ed7760f7c9d5e1cf654fcb7167c968f5f65aa` |

Hashes informados pelo executor coincidem. Os **41 inputs** foram autenticados e o produtor recomposto pelo framing existente: `sha256:6192a943a657f23ebaea7267eea67f63ee482741c85c37d44634817113bfebf9`.

A baseline do executor (`/tmp/compass-rd-baseline/inventory.json`, SHA `551ef43e6c5b47c54cd9dc05f00dd59348049e8db23938a3a4c7a2586ec8357c`) corresponde ao mapa incorporado. As cópias em `files/` confirmam exatamente cinco deltas autorizados; o delta de d0/mod.rs é somente três visibilidades. **171 arquivos preexistentes protegidos intactos**, onze novos autorizados, sem mudança do leitor D0, contratos ou gabaritos. Binário C01 salvo e 81 entradas C2-A1 conferidos.

## BUG-005 — valor de query confundido com flag JSON

**Tipo: implementação. Prioridade: P2. Estado: confirmado. Responsável: executor RD.** Causa localizada em [main.rs:270](../../crates/compass-cli/src/main.rs:270): `args_os().any(|a| a == "--json")` inspeciona tokens sem distinguir valor de opção. Quando Clap rejeita outro argumento, o ramo search usa esse booleano nas linhas 290–294.

Reprodução independente no binário entregue; não depende de arquivos existentes porque o erro é de parsing, anterior à leitura:

```sh
target/release/compass search --snapshot x --source x --query --json --max-matches bad
target/release/compass search --snapshot x --source x --query=--json --max-matches bad
```

Ambos têm a mesma query literal `--json` e **nenhuma flag de saída JSON**. Clap permite valores hifenizados em query. Ambos retornam exit 2, stderr vazio e o mesmo valor `{error:{code:invalid_arguments,message:invalid_arguments}}`, porém:

| Forma da query | Saída observada | Saída exigida sem flag |
|---|---|---|
| `--query --json` | Compacta, sem newline | JSON legível, com newline |
| `--query=--json` | JSON legível, com newline | Mesmo formato: controle aprovado |

Com uma flag `--json` real adicional, as duas formas produzem corretamente o compacto. Sem o argumento inválido, o caminho tipado trata `--query --json` como valor e usa o modo legível; o defeito está na detecção prévia usada após falha do parser.

**Requisito:** contrato §2 (query literal), §4 (sem flag: mesmo valor em JSON legível com newline); RD-AC-07/RD22. Não é decisão ambígua de produto nem defeito do matching, leitor ou matemática. O código/exit do erro permanecem corretos; a não conformidade é a escolha do transporte.

**Fechamento mínimo:** regressão permanente e controle público antes/depois exigindo JSON legível byte a byte nas duas sintaxes sem flag; com flag real nas posições globais/subcomando, exigir compacto sem LF. Manter os dois códigos invalid_arguments/2 e stderr vazio, leitura anterior ao parsing proibida, queries hifenizadas válidas e transportes Foundation/D0 preservados. Não proibir a query ou exigir a forma `=` para esconder o caso.

## Conferências executadas

O auditor executou em cópia isolada, com o **binário entregue**, o self-test e a campanha RD completa: 151 comandos CLI, 1.482 assertivas auxiliares, cinco mutações específicas e três medições. Esses números não são 1.482 cenários independentes. A primeira tentativa parou por EPERM ao criar socket Unix no sandbox; após permissão técnica, a campanha passou integralmente, sem skip. Não houve conexão de rede.

Uma revisão independente de fontes conferiu as barreiras RD12 (objeto, objects, ancestral e alias/../package) e RD19 (cancelamento/deadline e prioridade). Também executou **30 controles adicionais** com oráculo próprio: Unicode cruzando janelas de 65536 bytes, UTF-8 inválido/incompleto, NUL, linha longa tardia, literalidade e offsets. Passaram; BUG-005 foi descoberto fora da campanha entregue e reproduzido separadamente pela coordenação.

Ferramenta própria `check-artifacts.py`, somente biblioteca padrão, sem importar verificador/Rust, passou nos artefatos originais e nos regenerados: 641 registros brutos autenticados, 151 comandos, 50 relatórios search, 12 casos literais, 24 IDs/pointers e três amostras. Conferiu bytes/canonicalidade, bindings representativos, ordenação, partição dos arquivos, cobertura, truncamento, revisões dos hits, offsets/texto, memória lógica da carga e medidor nativo. O primeiro passe dessa ferramenta foi ajustado para respeitar o newline Foundation; a restrição foi da ferramenta do auditor, não um defeito da entrega.

| Verificação na cópia | Resultado |
|---|---|
| `scripts/cargo-local.sh fmt --all -- --check` | Exit 0 |
| `scripts/cargo-local.sh test --workspace --locked --offline`, target separado em /tmp | **74 testes Rust**, zero falhas/ignorados; socket local autorizado |
| `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings` | Exit 0 |
| `python3 verification/rd/acceptance.py --self-test` | Exit 0; nenhuma chamada de produção |
| `python3 verification/rd/acceptance.py --binary target/release/compass` | Exit 0 na execução autorizada; campanha/mutações/três medidas reproduzidas |
| `python3 verification/rd/acceptance.py --regressions --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; **23 Foundation + 3 mutações; 253 verificações V1/119 cenários + 11 mutações D0**, sem skips |
| Integridade C2-A1 e `git diff --check` | Exit 0; 81 artefatos intactos |

Não foi necessário rebuild release nesta auditoria: a campanha pública utilizou os bytes entregues, estáveis antes/depois, e os testes Rust compilaram os fontes idênticos em target separado. Não se atribui a esses testes um hash de release recompilado. Suites históricas foram **reexecutadas** contra RD pela montagem autenticada, sem sobrescrever os resultados originais; o runner R1 preso ao release C01 não foi usado.

As três novas amostras reproduziram leitura 4.096.000, reserva 4.097.000, peak lógico 14.690.378, 1000 linhas matching/20 entregues e truncated=true. RSS do auditor: **12696 / 12520 / 12608 KiB**; wall: **83434350 / 97761547 / 86174693 ns**. Stdout igual nas três amostras. Créditos lógicos, RSS, CPU e disco são campos separados; medidas locais descritivas, cache não controlado, sem SLO ou conclusão de escala.

## Resultado por aceite

| AC | Veredito | Evidência/limite |
|---|---|---|
| RD-AC-01 | CONFORME | Cinco deltas, onze adições autorizadas; 171 protegidos, inputs e contratos autenticados. |
| RD-AC-02 | CONFORME | Esperados literais independentes, fixture pré-produção preservada; self-test e mutações específicas. |
| RD-AC-03 | CONFORME | Literais/Unicode/bytes/offsets/ordem e controles adicionais aprovados. |
| RD-AC-04 | CONFORME | Fonte/binding/tipos/EOF e intervenções sincronizadas reais, com controles estáveis. |
| RD-AC-05 | CONFORME | Snapshot parcial, indisponibilidade, non_text e falha tardia/truncamento corretamente separados. |
| RD-AC-06 | CONFORME | S/W/reservas/caps e checks internos executados; não se promete preempção dos primitivos protegidos. |
| RD-AC-07 | **NAO_CONFORME** | BUG-005 na escolha do transporte após erro de parsing. Demais vetores e cinco mutações passaram. |
| RD-AC-08 | CONFORME | Fluxo público sem árvore viva, entradas intactas e três medidas nativas reproduzidos. |
| RD-AC-09 | CONFORME | 74 Rust e regressões Foundation/D0 do release RD passaram, sem skips finais. |
| RD-AC-10 | CONFORME | RD01–RD24 têm vínculos reais; proveniência/artefatos corretos e retorno pendente de verificação. A campanha não cobria o novo caso BUG-005. |

## Evidência e encerramento

Auxiliares em `/tmp/compass-audit-rd-k2kvztvm/`, sujeitos à retenção de temporários:

| Evidência | SHA-256 |
|---|---|
| `arg-query-reproduction.json` | `2d51e537a2f1d6051498cbaeddd529a26a7253556ea97344bb3325f77077cd8e` |
| `workspace.log` | `cbb1fac7533b38091035b69c20427f14bb3339ebab961a40767738eb22438924` |
| `campaign-permitted.log` | `b81032640cbc2029fe224ca1282c94ee3ec26b1086cd8e3306e3dc5528aeca50` |
| `artifacts-original.json` e `artifacts-replica.json` | `e1bcaf19f3710109800f3b23eac00f11bc8df9329672cefda7973f95cf41e9aa` |
| `/tmp/rd-source-audit/evidence.json`, controles independentes | `90c042fed06375fc8e9e9c013eb3c2898cd419c57538dfaec572dd96c5d8f1fa` |

| ID | Tipo | Estado | Fechamento | Tentativas corretivas |
|---|---|---|---|---|
| BUG-005 | implementação/P2 | **confirmado** | Corrigir somente detecção da flag real após erro; pares sem/com flag e regressões conforme acima | 0 |

Os **188 arquivos locais inventariados, incluindo o release**, permaneceram idênticos durante a auditoria. Apenas este parecer foi acrescentado antes da etapa separada de arquitetura da correção. Nenhum achado foi corrigido, contrato/gabarito modificado, outro chat contatado ou próxima capacidade iniciada. BUG-001–004 e BUG-RD-C-001 mantêm seus fechamentos anteriores; não foram reabertos por preferência.

Próximo ato: OS corretiva focada no BUG-005 e conferência posterior do mesmo cenário/AC. O restante da RD não deve ser reimplementado nem a aceitação de novas capacidades antecipada.
