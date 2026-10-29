# RESULTADO — D0-C2-A1

Estado: `CONCLUIDO` para o ajuste documental; contrato D0 continua candidato inativo. Data: 18/10/2026.

## Ordem, baseline e fronteira

- Ordem: `docs/COORDENACAO.md`, seção “Prompt pronto — D0-C2-A1”.
- Baseline: branch `main`; `HEAD` indisponível porque o repositório ainda não tem commit. Havia 34 arquivos Foundation já adicionados ao índice e `docs/`/`research/proposals/` não rastreados antes desta execução. Nada desse baseline foi sobrescrito.
- Escrita autorizada: candidato D0-C2, schemas, fixtures, evidência, índice/hash, `COMPASS_STATE.md` e `docs/COORDENACAO.md`. Não houve operação remota.

## AC-01 — igualdade normalizada obrigatória

Após bytes verificados, classificação textual, normalização, limite de linhas e admissão de memória, sequências normalizadas iguais usam `l=n0=nt`, `a=r=c=0` e `cells_reserved=0`. A política identificada é `normalized_lines_exact_equal_zero_else_n0_times_nt_v1`, com `metric_policy_id=51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Hashes, tamanho e número de linhas isolados não dispensam células. Sequências diferentes mantêm a reserva `n0*nt`, os limites e o débito existentes.

## AC-02 — casos reconciliados

- E36: igualdade verificada de 4.000×4.000 linhas, `0/8000`, completa, zero células.
- E51: igualdade segue disponível com saldo de células já zero.
- E52: CRLF/LF com bytes distintos, mas linhas normalizadas iguais, reserva zero.
- E53: fonte ausente/adulterada bloqueia antes da igualdade.
- E54: mesma contagem com sequências diferentes permanece `diff_work_limit`.
- E55: unidade igual não consome o saldo da unidade diferente seguinte.

## Verificações

```sh
python3 research/proposals/d0-c2/evidence/conferir.py --cached-dependencies /home/many/.cache/uv/archive-v0
```

Passou com 55 casos catalogados, 44 cálculos LCS literais, 23 agregações, 961 pares pequenos, 5 metaschemas e 10 relatórios/inputs. A evidência inclui os seis cenários da ordem. `SHA256SUMS` e a preservação dos 34 arquivos protegidos foram conferidos após os ajustes.

Não executado: suite Rust/CLI, testes de filesystem/corrida/prazo, benchmark/RSS e V1/I1. O checker é documental e não é o laboratório independente futuro.

## Pendência

P1–P4, o congelamento dos hashes candidatos e a liberação da coordenação continuam necessários antes de iniciar V1 ou I1.
