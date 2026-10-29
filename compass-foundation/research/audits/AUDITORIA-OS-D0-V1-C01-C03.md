# Auditoria focada — OS-D0-V1-C01, C02 e C03

**Veredito: CONFORME para as três correções e suas regressões afetadas. BUG-001, BUG-002 e BUG-003: VERIFICADOS.** Não foi identificado achado obrigatório remanescente nesse escopo. A entrega do laboratório V1 pode ser aceita após estas correções; execução e homologação do sensor D0 real permanecem com I1/R1.

Data: 19/10/2026, America/Sao_Paulo. Skill: `auditor-red-team` v2.0, modo `entrega`, revisão focada nos mesmos BUGs/ACs. Referências: [parecer anterior](AUDITORIA-OS-D0-V1.md), [C01](../../docs/entregas/d0/OS-D0-V1-C01.md), [C02](../../docs/entregas/d0/OS-D0-V1-C02.md), [C03](../../docs/entregas/d0/OS-D0-V1-C03.md) e [retorno do executor](../../verification/d0/RESULTADO-D0-V1.md). O parecer anterior permanece histórico e não foi alterado.

## Versão concreta e preservação

Baseline: branch `main`, sem HEAD/commits, com alterações locais e V1 não rastreada. O estado é identificado por hashes, não por um commit inexistente. Antes dos testes foi fixado um inventário de **139 arquivos**, incluindo resultados ignorados pelo Git e o documento principal protegido, com digest SHA-256 `2fed273d88dc0f73ad63c4e8f9229b8c8e80f9abe9df1e3a640488ea012a36de`.

| Referência auditada | SHA-256 |
|---|---|
| OS-D0-V1-C01 v1 | `361164078e93c330e1d7881e7ae0f9ebffbcf782e9b3d69d331020a9cad48291` |
| OS-D0-V1-C02 v1 | `dc9dcb4cb06ed5a85dc66d74ae861cdf729d95cd33281c4973b9d09c0962468c` |
| OS-D0-V1-C03 v1 | `37398e8020e126f2845a90083e92cab4b402f7b9a83560dad9a25672dff09a17` |
| `verification/d0/validate.py` | `39e8d47f4dd1baa743fdad98589771757dd8f510bcea08fe2239cd157bb64461` |
| `verification/d0/acceptance.py` | `d98ce5927217281bf0172fa989c786d55f329ef49b49ee02d221f836370737a2` |
| `verification/d0/cases.py` | `319516964ac3bcf0f89093cf09f898cfe5e9d3b188a9cc281379c466969b8c78` |
| `verification/d0/coverage.json` | `0ed2fc8894852b95c1e55c2cc2609f09f3714db51e64cc0dbc992c9e48afdba3` |
| `verification/d0/RESULTADO-D0-V1.md` | `4e04373959ebec7c1237cd888bb9aa67ab1038847fc5a97e5b5d6a01cea2c186` |
| Parecer anterior | `5c3134a49ee4a4125d67c5769a564139ad0149c190a49c50c6f80769c8d126e0` |

Conferidos os 138 hashes do inventário inicial do executor e as transições `C01-after`, `C02-after`, `C03-after`, disponíveis em `/tmp/compass-v1-corrections-vgt05mao/`. As mudanças observadas em cada etapa pertencem à sua lista fechada; não foram acrescentados arquivos ao projeto pela implementação corretiva. Os hashes finais do recibo C03 coincidem com os bytes atuais. C03 preservou `validate.py` pós-C02. Oráculo, mutações existentes, fixtures, produção, contratos, ativação, OSs e auditorias permaneceram preservados.

Também foram conferidos os **81 artefatos congelados** e os **34 arquivos protegidos**. O hash do índice permanece `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`; a política permanece `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

## Resultado por critério corretivo

| Critério | Resultado | Evidência observada pelo auditor |
|---|---|---|
| C01-AC-01 | CONFORME | Paths `1111` e `111111111111111111111111` passaram por path, schema, snapshot, modelo e relatório: `0/2`, massa dois, zero células. |
| C01-AC-02 | CONFORME | Seleção numérica por campos contratados; identificadores textuais preservados. Máximo u64 passa e overflow continua rejeitado. O papel textual de `metric_policy.unknown` é distinguido de `counts.unknown`. |
| C01-AC-03 | CONFORME | Controle próprio reproduziu a rejeição `u64` do path longo na versão anterior e sua aceitação na corrigida, sem eliminar rejeição de overflow real. A regressão e os cenários têm referências E24/E50 na matriz. |
| C01-AC-04 | CONFORME | Comandos comuns aprovados e retorno preservado por seção; runtime continua explicitamente pendente. |
| C02-AC-01 | CONFORME | Links intermediários para alvos existentes ou ausentes resultam em `source_invalid_kind`, por expectativa literal do contrato. |
| C02-AC-02 | CONFORME | Ausência regular mantém `object_missing`; regular válido passa; link direto, diretório, FIFO e socket continuam proibidos. |
| C02-AC-03 | CONFORME | Para `abc` bilateral com baseline bloqueada: reserva oito bytes, leitura três, baseline sem revisão/flags fabricados e current verificado. Controle regular lê seis bytes. |
| C02-AC-04 | CONFORME | Relatório correto para o link pendente é aceito; mentira schema-válida com `object_missing` é rejeitada por `report_units`. Na versão anterior, os resultados eram invertidos. |
| C02-AC-05 | CONFORME | Regressão e cenário E43 correspondem a testes reais; C01 permanece aprovado. Nenhuma prova estática é apresentada como segurança concorrente. |
| C03-AC-01 | CONFORME | Objetos físicos de E52 contêm `610d0a` e `610a`; hashes, tamanhos e binding dos descritores conferidos contra os buffers reais. |
| C03-AC-02 | CONFORME | Linhas normalizadas `['a']` em ambos os lados, Foundation `modified`, D0 completo `0/2`, decimal `0.000000`, massa dois e zero células. |
| C03-AC-03 | CONFORME | Receita defeituosa com os dois lados `610a` conserva modelo/schema válidos e D0 zero, mas a asserção do estímulo rejeita especificamente com `bug003_stimulus_bytes`. |
| C03-AC-04 | CONFORME | E52 aponta `literal-math-two-methods`, `prepare:E52` e `regression:BUG-003`; o crédito indevido de vetores foi retirado. E16a, E48 e demais cenários permanecem aprovados. |
| C03-AC-05 | CONFORME | C01/C02, self-test e mutações passam na versão acumulada; documentação e retorno distinguem testes offline, preparação e runtime ainda ausente. |

## Verificações executadas nesta auditoria

Os relatos do executor foram confrontados com leitura das alterações e execução própria. Para preservar os resultados entregues, os comandos foram executados em cópia temporária idêntica sob `/tmp/compass-audit-v1-fixes-e1vgn1lg/sandbox/compass-foundation/`.

| Comando/conferência | Resultado |
|---|---|
| `python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; `SELF_TEST_PASSED_RUNTIME_PENDING`, **132 verificações, 119 cenários, zero falhas**. |
| `python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0; **11 negativos rejeitados** pelas asserções previstas, controles válidos aprovados. |
| Hash do índice e conferência das 81 entradas | Todos coincidem com o conjunto autorizado. |
| Hashes dos 34 arquivos protegidos | Todos coincidem com os valores registrados. |
| `git diff --check` e `git diff --cached --check` | Exit 0. |
| Comparação dos resultados reproduzidos com os entregues | `self-test.json` e `mutations.json` idênticos byte a byte. |

O self-test e a montagem adicional da campanha foram executados com permissão técnica da ferramenta para criar socket Unix local, cuja restrição do sandbox já estava demonstrada no parecer anterior. Não houve operação de rede. Nenhum binário Foundation foi apresentado como D0, nem foi executado `--binary` contra um sensor.

O ambiente é Python com jsonschema já disponível no cache local; não houve instalação. Nenhum build/teste Rust foi necessário para a revisão destas correções Python.

## Controles independentes de fechamento

Além dos testes entregues, foi escrito um controle próprio, `closure.py`, no diretório temporário desta auditoria. Ele chama as interfaces do laboratório com estímulos e expectativas literais, sem usar as funções `regression_bug001/002/003` do executor para provar seus respectivos fechamentos. A asserção de estímulo E52 é exercitada diretamente como objeto sob teste.

Foi reconstruída uma cópia da versão anterior a partir de `C01-before`. Os hashes dos módulos anteriores foram comparados com os registrados no primeiro parecer antes dos controles. Contratos e entradas permanecem os mesmos nas duas cópias.

| Mesmo estímulo | Versão anterior | Versão corrigida |
|---|---|---|
| Path longo de aparência numérica | `u64` | Aceito com relatório completo `0/2` |
| Overflow real em contador | Rejeitado `u64` | Continua rejeitado `u64` |
| Objeto regular ausente | `object_missing` | Continua `object_missing` |
| Link intermediário pendente | `object_missing`, incorreto | `source_invalid_kind`, correto |
| Relatório correto do link pendente | Rejeitado `report_units` | Aceito |
| Relatório falso com motivo `object_missing` | Aceito | Rejeitado `report_units` |
| Bytes físicos do cenário E52 | `610a` / `610a` | `610d0a` / `610a` |
| Classificação Foundation de E52 | `unchanged` | `modified` |

Controles adicionais confirmaram tipos estáticos, reserva/leitura, validade de caminhos curtos, u64 máximo e a distinção entre regra textual e contador `unknown`. Em E52, o auditor recalculou hashes dos objetos e dos manifestos com `hashlib`, conferiu os descritores e verificou que a receita normalizada indevida continua rejeitada mesmo produzindo resultado matemático zero válido.

Resultados auxiliares: `current-closure.json`, `previous-closure.json` e `baseline.json` em `/tmp/compass-audit-v1-fixes-e1vgn1lg/`. Esses arquivos são evidência temporária da sessão; as versões e resultados essenciais estão registrados neste parecer.

## Registro de fechamento

| ID preservado | Tipo | Prioridade original | Estado anterior | Estado auditado | Correção auditada | Responsável pelo fechamento |
|---|---|---|---|---|---|---|
| BUG-001 | implementacao | P2 | confirmado | **verificado** | C01, primeira entrega corretiva | auditor |
| BUG-002 | implementacao | P2 | confirmado | **verificado** | C02, primeira entrega corretiva | auditor |
| BUG-003 | implementacao | P3 | confirmado | **verificado** | C03, primeira entrega corretiva | auditor |

O resultado `NAO_CONFORME` anterior identifica corretamente a versão anterior. Para a versão corrigida identificada aqui, os três motivos de não conformidade foram resolvidos e verificados. Não há necessidade sustentada de nova rodada de patch desses achados ou de reabrir P1–P4.

## Limites e alterações da auditoria

Este parecer encerra a revisão corretiva do laboratório. CLI D0 real, mutações sobre suas respostas, corridas sincronizadas/openat2, checkpoints, prioridade SIGINT/deadline, aritmética extrema direta em Rust e RSS permanecem provas futuras de I1/R1. A aprovação destas correções não afirma que essas provas já ocorreram.

Os 139 arquivos fixados no início permaneceram idênticos durante a auditoria. Somente este novo parecer foi acrescentado ao projeto; implementação, testes, resultados originais, contratos, critérios, estado e coordenação não foram modificados.

Uso da conta no início/fim: 12% → 12% na janela compartilhada de 10.080 minutos. Diferença visível zero, sujeita a arredondamento e outros trabalhos; não representa consumo nulo ou contagem de tokens desta revisão.
