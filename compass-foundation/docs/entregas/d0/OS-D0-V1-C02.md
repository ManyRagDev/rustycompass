# OS-D0-V1-C02 — corrigir BUG-002: componente proibido versus ausência

```yaml
id: OS-D0-V1-C02
versao: 1
estado: PREPARADA
porte: curto
origem: BUG-002 da AUDITORIA-OS-D0-V1; OS-D0-V1 v2, AC-04 e AC-06
baseline: árvore resultante de C01 com testes aprovados e hashes registrados
objetivo: classificar links intermediários pendentes como source_invalid_kind
dependencias: OS-D0-V1-C01 implementada e verificada pelo executor
autorizacoes_remotas: nenhuma
```

19/10/2026. Segunda ordem da [sequência corretiva](PLANO-ENTREGAS.md#correcoes-da-v1). A dependência é de execução sequencial, por compartilhar arquivos; não exige uma nova aprovação de produto.

## Baseline e leitura

Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`. Inspecionar Git/diff e a seção de retorno de C01 antes de escrever. A baseline inicial auditada e os inputs congelados estão no plano; os hashes pós-C01 substituem os hashes iniciais dos arquivos legitimamente alterados. Não reverter C01 para coincidir com a versão defeituosa.

Ler `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md), [OS-D0-V1 v2](OS-D0-V1.md), [BUG-002](../../../research/audits/AUDITORIA-OS-D0-V1.md), `validate.py::regular_path/model/report` e as montagens/testes pertinentes. O contrato D0 §2 define `source_invalid_kind` para link/componente proibido e `object_missing` para ausência de objeto. A auditoria confirma o problema em árvore estática, sem depender de TOCTOU.

## Comportamento

Com manifestos/descritores válidos, substituir `baseline-source/objects` por um link para destino inexistente. O modelo hoje conclui `object_missing` antes de conferir os componentes. Corrigir essa classificação mantendo a distinção entre tipo proibido, ausência regular e falhas de leitura já contratadas. A correção pertence ao modelo estático do laboratório; não criar outro mecanismo de produção/openat2.

Conservar reservas aceitas `s+1`, débito sem devolução e leitura do lado current quando admitido. Um bloqueio na baseline não deve conferir bytes externos, fabricar revisão, liberar cálculo ou ocultar verificações do outro lado. Manter compartilhamento e precedência de recursos existentes.

## Escrita autorizada

Somente arquivos **existentes**, com mudanças pertinentes ao BUG-002:

- `verification/d0/validate.py`;
- `verification/d0/cases.py`;
- `verification/d0/acceptance.py`;
- `verification/d0/coverage.json`;
- `verification/d0/ACCEPTANCE.md`;
- `verification/d0/RESULTADO-D0-V1.md`;
- `verification/d0/results/self-test.json` e `verification/d0/results/mutations.json`, gerados pelos comandos.

Os demais arquivos são protegidos, inclusive oráculo, mutações existentes, fixtures/contratos, Rust/Cargo, ativação, auditorias e documentos da coordenação. Reutilizar construtores de temporários; não acrescentar dependência, API pública ou arquivo de teste fora da lista. Preservar a correção e a regressão de C01.

## Aceites de fechamento

| ID | Dado / quando / então |
|---|---|
| C02-AC-01 | Componente intermediário `objects` como link para alvo existente ou inexistente resulta em `source_invalid_kind`. A expectativa é literal do contrato, não obtida apenas do modelo corrigido. |
| C02-AC-02 | Objeto regular genuinamente ausente continua `object_missing`. Link direto, FIFO, socket e diretório continuam proibidos; objeto regular válido continua verificável. |
| C02-AC-03 | Para o par de bytes `abc` nos dois lados, baseline bloqueada e current regular, reservas e bytes efetivamente conferidos permanecem coerentes; nenhuma revisão é atribuída ao lado bloqueado. |
| C02-AC-04 | O relatório contratualmente correto para o link pendente passa no schema e no verificador. A variante que troca o motivo por `object_missing` passa no schema, mas é rejeitada pela asserção semântica específica. Aceitação/rejeição não pode depender de erro genérico. |
| C02-AC-05 | A regressão e o cenário estático têm correspondência real em `coverage.json` para componentes de fonte. C01, self-test e mutações continuam passando. Relato mantém provas de concorrência/segurança de produção pendentes para I1/R1. |

## Verificação, retorno e reversão

Executar os comandos comuns do plano e a regressão adversarial descrita na auditoria, sem alterar o parecer. Socket Unix pode exigir permissão técnica; falha de montagem não vale como aprovação. Não iniciar I1/R1 nem exercitar corridas probabilísticas para alegar segurança.

Acrescentar `OS-D0-V1-C02 / BUG-002` ao resultado V1, com baseline acumulada, diff/hashes antes/depois, ACs e comandos. Registrar correção implementada/testada, aguardando auditoria. Reversão limitada aos hunks de C02, preservando C01 e alterações preexistentes. Só avançar a C03 se os testes pertinentes passarem.
