# OS-D0-V1-C01 — corrigir BUG-001: domínio u64 por campo

```yaml
id: OS-D0-V1-C01
versao: 1
estado: PREPARADA
porte: curto
origem: BUG-001 da AUDITORIA-OS-D0-V1; OS-D0-V1 v2, AC-04
baseline: main sem HEAD; versão V1 auditada, identificada abaixo
objetivo: aceitar caminhos nativos válidos sem enfraquecer validações u64
dependencias: P1–P4 aprovadas; ATIVACAO.md existente
autorizacoes_remotas: nenhuma
```

19/10/2026. Primeira ordem da [sequência corretiva](PLANO-ENTREGAS.md#correcoes-da-v1). Não há decisão de produto pendente. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.

## Baseline e leitura

Inspecionar Git/diff antes de escrever. Referência: [auditoria de V1, BUG-001](../../../research/audits/AUDITORIA-OS-D0-V1.md), SHA-256 `5c3134a49ee4a4125d67c5769a564139ad0149c190a49c50c6f80769c8d126e0`.

| Arquivo na versão auditada | SHA-256 |
|---|---|
| `verification/d0/validate.py` | `4337f4f5ce049557873d5b2827cf2ef47b9cafbd95f2bafd7f032db09278a963` |
| `verification/d0/acceptance.py` | `e54b49bf5f63ab73c5c62dd7bd2b92af5e62e3cc251f5f90cf5a15c977c074ae` |
| `verification/d0/cases.py` | `9aadb76d70ac3c509bfa5d33089a0bb18540d34fd0ff754c18916c0f4dade398` |

Ler `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md), [OS-D0-V1 v2](OS-D0-V1.md), BUG-001, e somente os trechos pertinentes de `validate.py`, `acceptance.py`, `cases.py`, `coverage.json` e schemas Foundation/D0. Consultar os campos numéricos nos schemas existentes, não deduzi-los pela aparência de seu valor. Usar o protocolo comum do plano para preservação, evidências e retorno. Divergência de baseline não autoriza sobrescrever trabalho existente.

## Comportamento

O `path_key` `111111111111111111111111` codifica um componente nativo válido, mas `domain()` também o interpreta como inteiro e rejeita por overflow. Corrigir a seleção dos campos sujeitos a u64 conforme seus papéis no contrato. Preservar validação canônica de caminhos, limites u64 reais, tipos, schemas e igualdade semântica de relatórios. Não resolver apenas removendo validação numérica ou proibindo paths numéricos.

Adicionar regressão executável ao self-test e um cenário estático de campanha com o caminho válido, reutilizando as montagens atuais. Os bytes `a\n` nos dois lados devem produzir resultado completo, numerador zero, massa dois e zero células. Associar a prova real aos IDs pertinentes de caminhos/u64 na matriz, identificando BUG-001 no resultado da correção.

## Escrita autorizada

Somente arquivos **existentes**, com mudanças pertinentes ao BUG-001:

- `verification/d0/validate.py`;
- `verification/d0/cases.py`;
- `verification/d0/acceptance.py`;
- `verification/d0/coverage.json`;
- `verification/d0/ACCEPTANCE.md`;
- `verification/d0/RESULTADO-D0-V1.md`;
- `verification/d0/results/self-test.json` e `verification/d0/results/mutations.json`, gerados pelos comandos.

Não é obrigatório alterar todos os caminhos permitidos. Todos os demais são protegidos, inclusive `oracle.py`, `mutations.py`, fixtures/contratos congelados, Rust/Cargo, ativação, auditorias, esta OS e documentos globais. Nenhum arquivo novo ou dependência nova é previsto. Descoberta fora da fronteira deve ser reportada à coordenação.

## Aceites de fechamento

| ID | Dado / quando / então |
|---|---|
| C01-AC-01 | `path_key='1'*24` e um path numérico curto passam na validação nativa, no snapshot e no modelo; o relatório válido de igualdade retorna `0/2`, massa dois e zero células. |
| C01-AC-02 | Strings textuais/identificadores não são limitados como contadores por parecerem números. Contadores e limites continuam sujeitos ao domínio u64; máximo permitido passa e overflow real é rejeitado pela asserção pertinente. |
| C01-AC-03 | A regressão protege os dois sentidos: aceita o caso válido e rejeita overflow de contador. Alterar o modelo para rejeitar tudo não satisfaz o aceite. O teste tem referência real na matriz/evidência, sem esperado extraído de Rust. |
| C01-AC-04 | Self-test, mutações e hashes congelados passam. Estado de runtime continua pendente; o retorno registra BUG-001 como correção implementada/testada pelo executor, aguardando auditoria. |

## Verificação, retorno e reversão

Executar os comandos comuns do plano: `acceptance.py --self-test`, `mutations.py --self-test`, conferência de SHA256SUMS e diff. A nova regressão deve falhar na versão auditada e passar após a correção, por comportamento verificável. Não executar o Foundation como D0.

Acrescentar uma seção `OS-D0-V1-C01 / BUG-001` ao resultado V1, preservando seu histórico: baseline, paths/hashes antes/depois, ACs, testes e pendências. Guardar cópia/diff local antes de editar; reversão restaura somente os hunks desta ordem, sem reset/clean do repositório. Não iniciar C02 em caso de regressão ou bloqueio não resolvido.
