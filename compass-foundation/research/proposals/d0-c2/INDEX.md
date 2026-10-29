# Índice D0-C2 — candidatos inativos

Entrega em 14/10/2026. Parecer **pendente para ativação**; P1–P4 e liberação da coordenação antecedem V1/I1. Nenhum arquivo desta pasta é contrato ativo. Foundation/C1/documento original preservados.

| Artefato | Finalidade |
|---|---|
| [Parecer](../D0_C2_REVISAO.md) | Evidências, gravidades/resoluções, E01–E35, decisões de produto, gate |
| [CONTRATO_CANDIDATO.md](CONTRATO_CANDIDATO.md) | Fonte/universo/matemática/disponibilidade/ledger/reservas/CLI/transporte/semântica/plano fechado |
| [schemas/report.schema.json](schemas/report.schema.json) | Report D0 separado, referencia tipos Foundation ativos sem modificá-los |
| [schemas/source.schema.json](schemas/source.schema.json) | Descriptor de 6 campos e binding |
| [schemas/metric-policy.schema.json](schemas/metric-policy.schema.json) | Regras fixas e configuração efetiva |
| [schemas/limits.schema.json](schemas/limits.schema.json) | Objeto completo de limites reduzíveis pela CLI candidata |
| [schemas/error.schema.json](schemas/error.schema.json) | Envelopes de erro/cancelamento, separados de unavailable |
| [fixtures/cases.json](fixtures/cases.json) | E01–E55, entradas literais e esperados independentes; D0-C2-A1 registra igualdade normalizada exata sem reserva de células |
| [fixtures/byte-vectors.json](fixtures/byte-vectors.json), [hash-vectors.json](fixtures/hash-vectors.json) | Normalização, identidade bruta e prefixo/hash de policy |
| [fixtures/resource-frontiers.json](fixtures/resource-frontiers.json) | Igualdade exata e excesso em bytes/linhas/células/memória/u64, incluindo agendas de saldo |
| [fixtures/metric-policy.json](fixtures/metric-policy.json), [metric-policy.jcs](fixtures/metric-policy.jcs) | Default e bytes JCS fixos; ID abaixo |
| [fixtures/limits-default.json](fixtures/limits-default.json), [limits-reduced.json](fixtures/limits-reduced.json) | Configuração default e perfil com total de células 4 |
| [fixtures/report-inputs.json](fixtures/report-inputs.json) | 10 montagens físicas manifesto/pacote/report/exit, incluindo fonte compartilhada e adulteração intencional |
| fixtures/manifests/, packages/ | 9 manifestos independentes,11 descritores e pequenos objetos; sem captura automática |
| fixtures/reports/, errors/ | 10 reports com `.jcs` fixos e 10 envelopes; complete/partial/unavailable, zeros, criação, unknown, non_text, transição e falha de hash |
| [evidence/conferir.py](evidence/conferir.py), [conferencias.json](evidence/conferencias.json) | 44 LCS literais,23 agregações,961 pares pequenos,5 metaschemas,10 reports/inputs/budgets, igualdade e negativos |
| [evidence/preservacao.json](evidence/preservacao.json), [LEIA-ME.md](evidence/LEIA-ME.md) | 34 arquivos protegidos, métodos/reprodução/limitações/uso da conta |
| [RESULTADO-D0-C2-A1.md](RESULTADO-D0-C2-A1.md) | Ordem executada, baseline, aceites, verificações e pendências |
| [SHA256SUMS](SHA256SUMS) | Hash de cada artefato revisado desta pasta e do parecer, conforme caminhos relativos |

Policy default D0-C2-A1: `metric_policy_id=51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

SHA256SUMS inclui **este índice** e o parecer, e exclui apenas a si mesmo (evita hash autorreferente). Paths são relativos à pasta d0-c2; `../D0_C2_REVISAO.md` é parte do conjunto revisado. Verificação:

```sh
cd research/proposals/d0-c2
sha256sum --check SHA256SUMS
```

O índice de hashes é a referência congelável para a coordenação; não recalcular esperados para acomodar produção. A ativação deve registrar quais hashes foram aprovados. Se mover schemas para outra pasta, ajustar referências relativas à Foundation e revalidar **antes** de consumidores, registrando novos hashes; não modificar Foundation para satisfazer referência. V1 e I1 devem receber o mesmo conjunto autorizado, com suas áreas de escrita delimitadas.
