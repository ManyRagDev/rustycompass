# OS-D0-V1-C03 — corrigir BUG-003: estímulo e evidência de E52

```yaml
id: OS-D0-V1-C03
versao: 1
estado: PREPARADA
porte: curto
origem: BUG-003 da AUDITORIA-OS-D0-V1; OS-D0-V1 v2, AC-02, AC-03 e AC-08
baseline: árvore resultante de C02, incluindo C01, com testes e hashes registrados
objetivo: montar os bytes congelados de E52 e vincular provas reais na cobertura
dependencias: OS-D0-V1-C02 implementada e verificada pelo executor
autorizacoes_remotas: nenhuma
```

19/10/2026. Terceira ordem da [sequência corretiva](PLANO-ENTREGAS.md#correcoes-da-v1). O próximo passo após ela é auditoria focada dos três BUGs.

## Baseline e leitura

Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`. Inspecionar Git/diff e os retornos C01/C02. Usar seus hashes de saída como baseline acumulada; preservar correções anteriores. Não há commit HEAD.

Ler `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md), [OS-D0-V1 v2](OS-D0-V1.md), [BUG-003](../../../research/audits/AUDITORIA-OS-D0-V1.md), E52 em `research/proposals/d0-c2/fixtures/cases.json` e somente as partes necessárias de `cases.py`, `acceptance.py`, `coverage.json` e documentação V1. O input literal de E52 já está aprovado; não há nova decisão de produto.

## Comportamento

A montagem genérica reconstrói E52 a partir de listas normalizadas e produz `610a` dos dois lados. Corrigir o cenário **E52** para consumir os inputs hexadecimais congelados: baseline `610d0a` e current `610a`. Manifestos, hashes e descritores devem ser preparados a partir desses bytes brutos, pelos construtores existentes, sem alterar os gabaritos C2.

Conferir explicitamente o estímulo físico antes de atribuir cobertura: bytes/hashes diferentes, linhas normalizadas iguais, classificação Foundation `modified`, divergência textual `0/2`, massa dois e zero células. Não usar apenas a igualdade do resultado para concluir que a normalização foi exercitada.

Reconciliar a entrada E52 da matriz com testes e cenários efetivamente executáveis. A função de vetores só pode aparecer como prova de E52 se realmente conferir seu input; caso contrário retirar esse crédito indevido e apontar a prova física correta. O cenário equivalente `bytes-E16a` continua preservado, assim como segundo BOM e demais receitas de normalização.

## Escrita autorizada

Somente arquivos **existentes**, com mudanças pertinentes ao BUG-003:

- `verification/d0/cases.py`;
- `verification/d0/acceptance.py`;
- `verification/d0/coverage.json`;
- `verification/d0/ACCEPTANCE.md`;
- `verification/d0/RESULTADO-D0-V1.md`;
- `verification/d0/results/self-test.json` e `verification/d0/results/mutations.json`, gerados pelos comandos.

Todos os demais arquivos são protegidos, inclusive `validate.py` corrigido por C01/C02, `oracle.py`, `mutations.py`, `fixtures.json`, C2 congelado, auditorias, contratos, ativação e documentos globais. Não acrescentar arquivos/dependências nem mudar schemas ou matemática. Esta ordem corrige a receita/evidência do caso existente.

## Aceites de fechamento

| ID | Dado / quando / então |
|---|---|
| C03-AC-01 | Os objetos físicos preparados de E52 contêm exatamente `610d0a` e `610a`; seus hashes são distintos e correspondem às respectivas revisões/descritores. |
| C03-AC-02 | Após verificação dos bytes, normalização produz sequências iguais; o resultado esperado continua completo, `0/2`, massa dois, decimal `0.000000` e zero células. A comparação Foundation registra mudança de bytes. |
| C03-AC-03 | Um controle que reconstrua ambos os lados apenas a partir das linhas normalizadas é rejeitado pela asserção do estímulo E52, mesmo que a matemática produza zero. Não aceitar erro de schema/import como prova desse controle. |
| C03-AC-04 | A matriz E52 aponta testes efetivos e o cenário físico correto; referências sem exercício real são retiradas/reconciliadas. E16a e os demais casos normalizados, inclusive E48, continuam aprovados. |
| C03-AC-05 | C01/C02 e os comandos comuns passam; documentação, evidências e retorno distinguem correções implementadas de auditoria pendente e sensor não executado. Nada transforma self-test em homologação de Rust/CLI. |

## Verificação, retorno e reversão

Executar os comandos comuns do plano. Registrar os bytes/hash observados de E52 e os IDs das asserções positiva/negativa, além do resultado matemático, como evidência de fechamento. Não basta referenciar `coverage-map` ou repetir a contagem total do self-test.

Acrescentar `OS-D0-V1-C03 / BUG-003` ao resultado V1, com diff/hashes e ACs; manter os resultados anteriores. Reversão limitada aos próprios hunks, preservando C01/C02. Encerrar a execução corretiva e devolver à coordenação para auditoria focada; não declarar os BUGs verificados pelo auditor nem iniciar I1/R1.
