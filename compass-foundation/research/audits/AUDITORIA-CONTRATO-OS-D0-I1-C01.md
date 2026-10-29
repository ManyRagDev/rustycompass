# Auditoria de contrato — OS-D0-I1-C01 v1

**Veredito: CONFORME.** Ordem corretiva executável, sem achado obrigatório. A correção permanece limitada ao BUG-004 e aos impactos nos I1-AC-03/04/07; não altera decisões de produto nem transforma HIP-I1-01 em obrigação.

19/10/2026, America/Sao_Paulo. Intenção: auditoria. Skill `auditor-red-team` v2.0, modo `contrato`. Revisão documental e estática focada; não confirma implementação ou fechamento do BUG.

## Versão concreta

Git sem HEAD, com alterações locais e arquivos não rastreados; não há commit de referência. Os seis hashes de baseline listados na [OS corretiva](../../docs/entregas/d0/OS-D0-I1-C01.md) coincidem com os arquivos presentes.

| Referência | SHA-256 |
|---|---|
| OS-D0-I1-C01 v1, estado `PREPARADA_PARA_AUDITORIA_DE_CONTRATO` | `7998fcf1777da1d0752f1dfa946a33167664f71b2be0063c3e94e11b7b660447` |
| Mesma OS, removendo somente a linha iniciada por `estado: `, preservados os demais bytes | `ed23ec4f8fdd9c7799e22477e1e6adbb3eecc92fd044edf970f581f190b069e6` |
| Contrato C2-A1 ativo | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| Índice C2-A1 | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |

O parecer admite a transição administrativa para `LIBERADA`/`LIBERADA_PARA_EXECUCAO`, alterando somente o campo `estado` da C01. Escopo, aceites, baselines e demais bytes permanecem os auditados; o segundo hash permite conferir essa condição. Esta transição não autoriza editar a OS-I1 original.

## Fundamentação

| Conferência | Evidência e conclusão |
|---|---|
| Causa e obrigação existente | Contrato ativo §2 exige associação nome↔handle e validade de ambas as referências compartilhadas. `Source::shared` (`source.rs:307–308`) verifica raiz/descritor/revisão; `report.rs:116–118` clona a evidência, enquanto `BoundFile::stable` (`source.rs:227–236`) confere a raiz do lado lido. A C01:44–54 corrige essa omissão e exclui expressamente a hipótese sobre metadados de diretórios. |
| Fronteira suficiente | Helpers internos em `source.rs`, integração em `report.rs` e testes em `mod.rs` permitem a correção sem nova dependência ou API pública. A lista fechada autoriza somente esses três Rust, apêndice ao resultado I1 e os dois JSONs gerados; protege V1, baseline, CLI, Cargo e demais arquivos (C01:58–68). |
| Compartilhamento e ledger | C01:48–52 e AC-02/03 preservam uma reserva de 3 bytes, leitura física de 2 e zero células para `a\n`, com controle estável completo `0/2`. A referência instável bloqueia cálculo, conservando apenas fatos EOF comprovados. Permanecem identidade física, binding bruto, objeto esperado e limite por unidade; não se autoriza desabilitar compartilhamento ou modificar o ledger. |
| Regressão antes do patch | C01:72–83 define fixture, pacote real artificial, referência com `alias-component/../package`, intervenção e asserções específicas. `mod.rs:16–43` permite testar operação, relatório e exit; seus testes já acessam `Context::hook` e barreiras. `source.rs:337` oferece `object_opened` antes da leitura. Logo, o teste permanente pode exercitar a omissão existente e falhar antes da mudança de produção, sem depender de auxiliares em `/tmp`. |
| V1 e identidade histórica | `run-v1.py:58` exige o hash da OS-I1 original; seu SHA atual `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4` coincide com `baseline.json::os_sha256`. C01 preserva esses inputs e exige campanha pelo adaptador intacto, binário estável e cópias exatas dos JSONs (AC-05/06), sem regenerar baseline ou esperados. |
| Aceite e reversão | AC-01–06 exigem reprodução específica, controles positivos, regressões, proveniência e inventário. Falha/permissão indisponível não conta como aprovação; reversão preserva trabalho anterior. O executor termina pendente de verificação, sem fechar BUG-004 ou liberar R1. Não há autorização remota. |

Nenhuma suíte de produção, build ou campanha runtime foi executada nesta revisão. O comportamento corrigido e os seis ACs serão verificados após a implementação; o parecer anterior permanece histórico. Somente este arquivo foi escrito pela auditoria.
