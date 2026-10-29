# Auditoria — OS-D0-V1 versão 2

**Veredito: NAO_CONFORME.** Foram confirmados dois defeitos do validador estático (BUG-001/002) e uma divergência de rastreabilidade do cenário E52 (BUG-003). Os self-tests e negativos entregues passam, mas não detectam esses desvios. Não há alteração de contrato ou decisão de produto exigida pelos achados.

Data: 19/10/2026, America/Sao_Paulo. Skill: `auditor-red-team` v2.0, modo `entrega`. Escopo exclusivo: implementação de OS-D0-V1 v2 e AC-01–AC-08. Ambiente: Linux `7.0.11-76070011-generic`, Python 3.12.3, jsonschema do cache existente; nenhuma instalação.

## Baseline e preservação

Branch `main`, sem commits/HEAD, confirmada por Git. Não se atribui a entrega a um commit nem se presume que o diff de Git, sozinho, represente V1: havia adições Foundation e diretórios não rastreados antes da implementação.

Antes dos testes foi fixado inventário SHA-256 de **135 arquivos**, incluindo os dois resultados ignorados pelo Git e o documento principal protegido. Digest do inventário: `62d5329b846e0f548e0486ff02b31a721b6f961f4ca13e6d17224acf57584687`. Inventário e evidências auxiliares desta sessão: `/tmp/compass-audit-v1-u07qngkh/`.

| Referência | SHA-256 |
|---|---|
| `docs/entregas/d0/OS-D0-V1.md`, v2 | `9ab2f299498ac110825fa69adff5d081b6c02719ea93cca1854cac72f449d1e5` |
| `docs/entregas/d0/ATIVACAO.md` | `a8ec86cf2958648b7384e7577362f98b3b0a6e1b67ae707c051d01fbcfa313c9` |
| Índice congelado C2-A1 | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| `verification/d0/validate.py` | `4337f4f5ce049557873d5b2827cf2ef47b9cafbd95f2bafd7f032db09278a963` |
| `verification/d0/cases.py` | `9aadb76d70ac3c509bfa5d33089a0bb18540d34fd0ff754c18916c0f4dade398` |
| `verification/d0/coverage.json` | `b2d971f6c46da7672564954d9c043c4f0b6536c2e1d02529bd97bb5a83500fda` |
| `verification/d0/RESULTADO-D0-V1.md` | `62fd302fb6530fba611d40ea694c5e9f2217f269bba93b38ca340d9c78e621b5` |

O conjunto dos **12 arquivos entregues** em V1, representado pelo mapa de caminhos/hashes em `v1-hashes.json` no diretório temporário da auditoria, tem digest `e85246135da8de529890f6beb3893c4e2d2eb21de1c35a35dbb3f242d0e1ecaa`.

Conferido independentemente o inventário inicial do executor, `/tmp/d0-v1-baseline.json`: seus **122 arquivos preexistentes permanecem idênticos**. Os dez novos arquivos de código/documentação e os dois resultados gerados estão na lista fechada da OS. `results/acceptance.json` não existe, coerente com ausência de execução contra D0. Também passaram as 81 entradas do índice congelado e os hashes dos 34 arquivos protegidos. Não foi encontrada escrita indevida nos arquivos conferidos.

Os comandos que geram resultados foram executados em uma cópia idêntica, sob `sandbox/compass-foundation` dentro do diretório temporário da auditoria. Os arquivos originais, inclusive as evidências entregues, permaneceram intactos. A única adição ao projeto nesta auditoria é este parecer.

## Resultado por aceite

| AC | Resultado da auditoria | Evidência e limite |
|---|---|---|
| AC-01 | CONFORME | Índice autorizado, 81 artefatos, política, 34 protegidos e inventário preexistente conferidos. A cópia adulterada é rejeitada. |
| AC-02 | PARCIAL | Matemática e independência conferidas; matriz tem 55 IDs e referências executáveis, mas a correspondência específica de E52 está incorreta (BUG-003). |
| AC-03 | CONFORME_COM_RESSALVAS | E36/E51/E53/E54/E55 e os controles de igualdade, saldo e fontes passaram. O comportamento de CRLF/LF está coberto pelo cenário equivalente `bytes-E16a`, mas o cenário/mapeamento E52 não preserva seus inputs (BUG-003). Nenhuma execução Rust D0 foi comprovada ou exigida nesta etapa. |
| AC-04 | NAO_CONFORME | BUG-001 rejeita um caminho nativo válido; BUG-002 aceita uma classificação incorreta e rejeita a correta para um componente proibido. Os demais controles entregues de matemática, binding, disponibilidade, reservas e ledger passaram. |
| AC-05 | CONFORME quanto à preparação; runtime NAO_VERIFICADO | Modo `--binary`, montagem, comparação, códigos, repetição e inventário estão implementados. Não havia binário D0 disponibilizado; não foi usado Foundation como substituto. O verificador deve ser corrigido antes de servir como referência da integração. |
| AC-06 | PARCIAL | Campanha estática e socket montados; BUG-002 afeta a interpretação de componente proibido. Corridas e checkpoints internos estão explicitamente atribuídos a I1/R1, sem falsa aprovação. |
| AC-07 | CONFORME offline | Onze controles positivos passaram; onze mutações schema-válidas foram rejeitadas pelos códigos de asserção esperados, sem contar falhas genéricas. Mutação em respostas reais permanece pendente. |
| AC-08 | PARCIAL | A distinção entre self-test, preparação e runtime está correta. O aceite final de V1 depende do fechamento dos desvios confirmados; a declaração anterior de conclusão não substitui esta conferência. |

## Comandos e verificações executados pelo auditor

O relato do executor foi confrontado com execução própria. Na cópia temporária, com scripts e inputs idênticos:

| Comando | Resultado observado |
|---|---|
| `python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` — sandbox restrito | Exit 1: `socket.bind()` retornou EPERM na construção; a conferência da matriz também falhou por consequência da campanha não montada. Não foi contabilizado como aprovação ou defeito de código. |
| Mesmo comando com permissão técnica fora do sandbox | Exit 0: `SELF_TEST_PASSED_RUNTIME_PENDING`, **126 verificações, zero falhas**, 116 cenários preparados. Permissão concedida pela ferramenta para o socket Unix local; sem operação de rede. |
| `python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0` | Exit 0: **11 mutações rejeitadas**, controles positivos e schema aprovados, zero falhas. |
| `(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)` | Exit 0; índice e 81 artefatos preservados. |
| `git diff --check` e `git diff --cached --check` | Exit 0. Arquivos novos não rastreados também tiveram whitespace conferido diretamente. |
| `git rev-parse --verify HEAD` / `git branch --show-current` | HEAD ausente; branch `main`. Condição de baseline, não falha da entrega. |

Os JSONs de self-test e mutações reproduzidos são **idênticos byte a byte aos entregues**. A primeira falha por sandbox foi preservada separadamente em `self-test-sandbox-restricted.json`, sem substituir evidência original.

Conferências adicionais, sem importar Rust ou o checker C2:

- Leitura dos cinco módulos, fixtures, matriz, resultado e contrato ativado. Os esperados usam literais congelados e um modelo Python próprio; não há captura de esperado da produção. O oráculo usa DP e o self-test o confronta com enumeração de subsequências.
- **200 pares adicionais** de sequências ternárias, comprimentos de zero a sete, com semente `20261019`: LCS comparada com recorrência recursiva memoizada escrita para esta auditoria; todos passaram.
- **200 razões adicionais** no domínio u64, com a mesma semente: arredondamento comparado com Decimal, precisão 80 e `ROUND_HALF_EVEN`; todos passaram.
- Controles independentes de path numérico, componente de fonte com link válido/pendente, aceitação de relatório falso e rejeição de relatório corrigido; reproduções abaixo.
- Montagem física da campanha e leitura dos objetos de E52 e do cenário equivalente E16a. A construção exigiu a mesma permissão de socket local; não houve execução de sensor.

Os scripts auxiliares são `probes.py` e `independent.py`, com saídas `probes.json` e `independent.json`, no diretório temporário indicado na baseline. Eles não alteraram implementação ou testes entregues.

## Achados confirmados

Os IDs pertencem à OS-D0-V1. Tentativas indicam correções posteriores a esta auditoria, não o desenvolvimento anterior do laboratório.

| ID | Tipo | Prioridade | Estado | AC | Tentativas | Responsável |
|---|---|---|---|---|---|---|
| BUG-001 | implementacao | P2 | confirmado | AC-04; caminhos nativos | 0 | executor V1 |
| BUG-002 | implementacao | P2 | confirmado | AC-04/AC-06; componentes de fonte | 0 | executor V1 |
| BUG-003 | implementacao | P3 | confirmado | AC-02/AC-03/AC-08; rastreabilidade E52 | 0 | executor V1 |

### BUG-001 — path base64url válido tratado como contador u64

**Requisito:** caminhos são bytes nativos codificados em base64url; o domínio u64 se aplica aos campos de contagem. O contrato Foundation e AC-04 não restringem um path por sua aparência numérica.

**Evidência:** [validate.py:177](../../verification/d0/validate.py) valida o path e, em seguida, aplica `uint()` a qualquer string que pareça um inteiro. O `path_key` `111111111111111111111111` é base64url canônico de `d75d75d75d75d75d75d75d75d75d75d75d75`: um componente nativo válido, sem NUL, barra ou componentes especiais.

Reprodução mínima, na raiz do projeto, com `verification/d0` no `sys.path` e `validate` importado como `v`:

```python
v.path_bytes('1' * 24)              # aceito como caminho nativo válido
v.domain({'path_key': '1' * 24})    # Rejected: u64
```

Também foi montado um par de manifestos/fontes com esse caminho e `a\n` nos dois lados. O manifesto passa no schema Foundation e na conferência específica do path; `snapshot()` falha com `u64` antes do modelo. O resultado textual esperado é completo, `0/2`.

**Impacto:** uma entrada válida é recusada pelo verificador; uma implementação correta de D0 pode ser acusada de falha. O problema pode atingir outras strings semânticas de aparência numérica.

**Correção mínima esperada:** aplicar u64 aos campos numéricos definidos pelo contrato, preservando o domínio dos identificadores e caminhos. Não mudar schema ou limite de paths para acomodar o bug.

**Fechamento:** o par nativo acima passa pelo modelo/relatório com `0/2`; um path numérico pequeno também continua válido; overflow real em um contador continua rejeitado pela asserção u64; self-test e mutações existentes permanecem aprovados.

### BUG-002 — link intermediário pendente perde a classificação de tipo proibido

**Requisito:** contrato D0 §2, linha 23: link/componente proibido resulta em `source_invalid_kind`; objeto ausente regular resulta em `object_missing`. Este caso é estático e não depende das provas de corrida reservadas a I1/R1.

**Evidência:** [validate.py:408](../../verification/d0/validate.py) testa primeiro `obj.exists()` e `obj.is_symlink()` no caminho final. Se `objects` é um link intermediário para destino inexistente, ambos são falsos e o caminho recebe `object_missing`, antes de a verificação dos componentes ser chamada.

Reprodução: criar o caso estático `{p: b'abc'}` nos dois lados, mantendo manifestos e descritores válidos; renomear `baseline-source/objects` para `kept`; criar `objects` como link para `kept` ou para um nome inexistente.

| Variante | Esperado pelo contrato | Modelo observado |
|---|---|---|
| Link intermediário para `kept`, existente | `source_invalid_kind` | `source_invalid_kind` |
| Mesmo componente como link pendente | `source_invalid_kind` | **`object_missing`** |

O lado current continua regular e verificado. Um relatório sintético com a classificação incorreta do modelo foi aceito por `report()`. Alterar somente os motivos da unidade e do check baseline para a classificação correta preserva validade do schema, mas provoca rejeição `report_units` pelo verificador.

**Impacto:** o oráculo aceita um desvio de classificação da produção e rejeita a resposta contratualmente correta. Não foi demonstrado acesso indevido pelo Rust; a falha está no modelo estático do laboratório.

**Correção mínima esperada:** distinguir componente proibido de ausência regular no modelo estático antes de concluir `object_missing`; não exigir que Python vire a implementação segura de produção.

**Fechamento:** links intermediários com alvo existente ou ausente recebem `source_invalid_kind`; objeto regular ausente continua `object_missing`; o relatório correto é aceito e o relatório com motivo falso é rejeitado; os cenários de links/FIFO/socket e as reservas permanecem coerentes.

### BUG-003 — cenário E52 usa bytes iguais, embora declare testar CRLF/LF distintos

**Requisito:** E52 congelado e AC-03 exigem bytes/hashes distintos que resultam em linhas normalizadas iguais, com zero células. AC-02 exige que a matriz corresponda aos cenários realmente preparados.

**Evidência:** a receita genérica em [cases.py:104](../../verification/d0/cases.py) reconstrói ambos os lados a partir das listas de linhas já normalizadas, ignorando o par hexadecimal declarado para E52.

| E52 | Baseline hex | Current hex |
|---|---|---|
| Inputs congelados | `610d0a` | `610a` |
| Objetos efetivamente preparados pelo cenário `E52` | **`610a`** | **`610a`** |

`coverage.json` associa o runtime de E52 somente ao cenário `E52`. A prova offline `byte-hash-schema-vectors` é associada a E52 na matriz, mas seus vetores não contêm E52. O cenário `bytes-E16a` realmente monta bytes diferentes com linhas normalizadas iguais, porém não está vinculado à entrada E52 da matriz.

**Impacto:** a campanha possui cobertura equivalente de normalização em E16a, mas a evidência atribuída a E52 não comprova seus inputs declarados. O próprio teste `coverage-map` confere nomes/IDs e aceita esse vínculo sem conferir o estímulo.

**Correção mínima esperada:** fazer E52 consumir os bytes literais congelados, ou registrar explicitamente a cobertura pelo cenário equivalente correto, com as asserções correspondentes. Não alterar os esperados congelados.

**Fechamento:** uma prova efetivamente vinculada a E52 verifica bytes e hashes distintos, igualdade somente após normalização, zero células, massa dois e D0 zero; matriz/resultado apontam essa prova. A diferença de bytes permanece visível na comparação Foundation.

## Pendências legítimas e decisão de integração

A ausência de execução contra D0 **não é um novo defeito**: nenhum release D0 estável foi disponibilizado e a própria OS permite concluir a preparação da campanha antes de I1. Permanecem atribuídos a I1/R1: execução real da CLI e mutações, corridas sincronizadas após abertura, estabilidade/EOF, checkpoints, prioridade SIGINT/deadline, extremos aritméticos diretos em Rust e medição de RSS. O EPERM do primeiro teste foi uma restrição técnica reproduzida e superada com autorização da ferramenta, sem convertê-la em aprovação por suposição.

Não restou hipótese usada como bloqueio neste parecer. BUG-001/002 impedem aceitar a V1 como referência confiável da integração até correção e revisão focada; BUG-003 deve reconciliar a rastreabilidade sem redesenhar a campanha. O próximo responsável é o executor V1, na mesma OS e nos mesmos ACs. Esta auditoria não inicia correções, I1/R1 ou outro chat.

## Alterações e uso

Nenhum código, contrato, esperado, teste entregue, estado ou coordenação foi modificado. Somente este relatório foi acrescentado ao projeto; saídas de execução e controles próprios ficaram em temporários. Os hashes da versão auditada devem ser reavaliados após qualquer correção.

Uso da conta consultado no início/fim da conferência: 11% → 11%, janela compartilhada de 10.080 minutos. A diferença visível foi zero pontos percentuais; isso é arredondamento compartilhado com outros trabalhos e não indica consumo nulo ou contagem exata de tokens.
