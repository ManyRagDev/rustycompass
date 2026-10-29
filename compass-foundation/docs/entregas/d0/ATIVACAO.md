# Ativação do contrato D0 e liberação de V1

Data: **19/10/2026**, America/Sao_Paulo.

**Contrato D0: ATIVO PARA DESENVOLVIMENTO E VERIFICAÇÃO. OS-D0-V1, versão 2: LIBERADA PARA EXECUÇÃO.**

## Aprovação de produto

Nesta conversa de coordenação, Emanuel respondeu **“Sim, aprovo”** à pergunta que enumerou as quatro recomendações para a primeira entrega técnica: pacotes externos preparados pelo operador; peso linear pela massa de linhas; normalização textual e zeros explícitos; verificação rigorosa dos bytes, com resultado integral ausente quando faltar evidência.

Essa resposta é a autorização de produto registrada abaixo. A coordenação conferiu a integridade do conjunto auditado antes de liberar a execução.

| ID | Decisão aprovada | Efeito nesta entrega |
|---|---|---|
| P1 | Pacotes externos explícitos, vinculados aos manifestos e com objetos verificados por hash/tamanho | O operador conserva e prepara os bytes necessários; captura/retenção automática fica fora de I1 |
| P2 | Massa bilateral linear `n0+nt`, sensor `d0_line_mass_v1` | Agregado conforme o contrato; não se adota o peso logarítmico histórico nem se presume superioridade preditiva |
| P3 | UTF-8 estrito sem NUL; normalização de um BOM inicial, CRLF e fragmento final de LF; zeros explícitos | Vazio verificado pode ter massa zero; mudanças de bytes podem ter D0 zero; binários/transições ficam fora do universo textual conforme o contrato |
| P4 | Verificação dos bytes, inclusive unchanged/vazios; integral null sob bloqueio e subconjunto conhecido separado | Ausência/adulteração não vira zero. A igualdade exata normalizada de A1 reserva zero células somente após fontes verificadas e admissões anteriores |

P1–P4 estão **APROVADAS** para a primeira entrega técnica. Não há decisão de produto remanescente bloqueando OS-D0-V1.

## Conjunto autorizado e congelado

Referências: [contrato consolidado](../../../research/proposals/d0-c2/CONTRATO_CANDIDATO.md), [índice de 81 artefatos](../../../research/proposals/d0-c2/SHA256SUMS) e [auditoria CONFORME de A1](../../../research/audits/AUDITORIA-OS-D0-C2-A1.md).

| Identificador | Versão/valor autorizado |
|---|---|
| Contrato D0 / schema / métrica | `0.1.0` / `1` / `1` |
| Sensor | `d0_line_mass_v1` |
| SHA-256 de `research/proposals/d0-c2/SHA256SUMS` | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| SHA-256 de `CONTRATO_CANDIDATO.md` | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| `metric_policy_id` default | `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5` |
| Regra de células | `normalized_lines_exact_equal_zero_else_n0_times_nt_v1` |
| SHA-256 da auditoria de A1 | `9e89ad9217b50f475fafd4dc74ce7edae37345187ca00225cfa8d8f8b9b51c20` |

Os 81 artefatos e a auditoria permanecem em seus caminhos originais, sem regravação. Os rótulos de candidato inativo nesses documentos registram o estado anterior à aprovação; **este registro estabelece a ativação atual** do conjunto identificado pelos hashes acima. Não é necessário mover schemas ou alterar a Foundation para consumi-lo.

## Liberação e fronteiras

A coordenação libera a execução de [OS-D0-V1](OS-D0-V1.md), versão 2, nos arquivos, aceites e limites já definidos. O implementador pode retomar a mesma conversa que reportou `BLOQUEIO_DE_PRODUTO`. O bloqueio causado pela ausência deste registro está resolvido.

V1 entrega o verificador independente e a campanha. I1 receberá sua própria OS para implementar Rust/CLI sob o mesmo conjunto aprovado; sua execução não é iniciada por este registro. O paralelismo e a integração seguem o [plano de entregas](PLANO-ENTREGAS.md).

Esta ativação não significa que o sensor já foi implementado ou testado em runtime. O build Foundation anterior mantém sua identidade; D0 será homologado após implementação e integração. Nenhuma operação remota é autorizada.

Mudanças posteriores de significado, schema ou esperado exigem reconciliação explícita e novos hashes antes de seus consumidores. O executor não deve regenerar o índice para contornar divergências.

## Conferência da liberação

Em 19/10/2026, conferidos o hash do índice, suas 81 entradas e os 34 arquivos protegidos por `evidence/preservacao.json`: todos íntegros. Esta liberação documental não exigiu nova suíte Rust/CLI ou repetição das contas já auditadas.
