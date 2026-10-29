# Auditoria de contrato — OS-D0-I1 v1

**Veredito: CONFORME.** A ordem é executável dentro das fronteiras propostas, preserva o significado ativo e a independência dos esperados. Não foi identificado achado obrigatório que exija correção antes da liberação pela coordenação. Este parecer avalia a ordem; não declara I1 implementada nem inicia I1/R1.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo `contrato`. Escopo: [OS-D0-I1](../../docs/entregas/d0/OS-D0-I1.md), integração com a V1 aceita e contrato C2-A1 ativado; sem reauditoria geral da V1.

## Versão concreta

Branch `main`, sem commits/HEAD, com alterações locais e arquivos não rastreados. Não há commit ao qual atribuir o parecer. A coordenação atualizou plano/estado/coordenação durante esta revisão; esses registros não são inputs do build nem objetos congelados desta auditoria. A OS permaneceu com o hash abaixo após sua fixação.

| Referência | SHA-256 auditado |
|---|---|
| **OS-D0-I1 v1** | **`17e6fea97ea4d8267307bd2e5809c987661e365f8babd6aa5b2558d076feca63`** |
| Ativação | `a8ec86cf2958648b7384e7577362f98b3b0a6e1b67ae707c051d01fbcfa313c9` |
| Índice C2-A1, 81 entradas | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| Contrato D0 ativo | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| Auditoria V1 C01–C03 | `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1` |
| Resultado V1 | `4e04373959ebec7c1237cd888bb9aa67ab1038847fc5a97e5b5d6a01cea2c186` |

Os dez arquivos duráveis atuais de `verification/d0/`, excluindo `results/` e caches, têm digest de inventário `1587531eedf72dc0b5c87047ec71756241e3041f733cc9c885009bf169ee6aa8`: SHA-256 do mapa caminho relativo→SHA-256, serializado por `json.dumps(sort_keys=True,separators=(',',':')).encode()`. Os hashes de `validate.py`, `acceptance.py`, `cases.py` e `coverage.json` coincidem com a auditoria corretiva aceita. O estado atual registra o aceite da V1; os pareceres históricos anteriores não são tratados como bloqueios novos.

## Conferência e evidências

| Aspecto | Resultado e fundamento |
|---|---|
| Autoridade e objetivo | A ativação registra P1–P4 e identifica contrato, índice e política. A OS remete a esse conjunto como autoridade, preserva Foundation e não reabre decisões de produto (OS 18–38, 156–162). |
| Fronteiras de escrita | Quatro arquivos existentes têm integração delimitada; módulos/testes/adaptador/evidências novos têm caminhos fechados. Dependências, algoritmos Foundation, contratos, V1 e esperados ficam protegidos. As APIs públicas de validação/comparação/JCS e as dependências existentes permitem a extensão; a resolução efetiva privada pode ser reproduzida localmente com equivalência, expressamente autorizada (OS 40–69). Não foi encontrada necessidade contratual demonstrada de arquivo proibido. |
| Independência dos esperados | E01–E55, literais e V1 são consumidos por leitura, sem gerar esperado de Rust ou copiar a implementação Python. As provas internas de segurança, preflight, checkpoints e aritmética têm aceites próprios e controles válidos; a campanha CLI complementa essas provas (OS 88–128). |
| Segurança e disponibilidade | Mesmos bytes para parse/hash; abertura segura, tipos, associação nome↔handle e estabilidade; ausência/pacote inválido/binding/objeto local separados. Universo, precedências, normalização, igualdade exata após admissões, LCS, reservas e ledger seguem §§2–6 ativos. AC-02–08 exigem evidência comportamental e corridas com barreiras, sem tratar build ou corrida probabilística como substitutos. |
| Interface e erros | As flags reproduzem §7: baseline/current obrigatórios, sources opcionais, perfil completo dos 11 limites estáticos, deadline apenas por flag em 1..3600000 e JSON JCS sem newline. A remissão ao contrato preserva os dez códigos, `message=code`, exits 0/3/1/2/130 e prioridades. Erros de argumentos D0 antes do dispatch recebem tratamento próprio; Foundation conserva seus envelopes. A OS não cria flag pública de teste (OS 98–108; schemas limits/error). |
| Aceite, permissões e reversão | ACs distinguem campanha, regressões Foundation, identidade real e auditoria/R1 pendentes. Há registro explícito de falhas/skips/permissão indisponível, contenção dos testes em árvores artificiais e reversão somente dos novos arquivos/hunks. Operações remotas e acervos reais não estão autorizados (OS 112–162). |

## Baseline histórica e execução honesta da V1

O conflito é real e corretamente delimitado: `verification/d0/validate.py:132–149` confere os 81 artefatos e os 34 arquivos históricos, incluindo README, `lib.rs`, `build.rs` e CLI. Rodar essa conferência diretamente na futura árvore I1 modificada produzirá `protected_hash`. A OS não manda atualizar esse esperado histórico.

O protocolo de OS 71–84 preserva os 34 arquivos antes das edições, com nomes relativos normalizados sob as duas raízes irmãs, inventário/hash do tar, identidade da V1 e hash da própria OS. Reutilização exige conferência; membros extras, `..`, links e tipos especiais são rejeitados. A montagem temporária repõe também C2, seu parecer externo `research/proposals/D0_C2_REVISAO.md`, schema Foundation e auditoria A1 nas posições exigidas. Assim, a referência externa do índice e o único documento irmão do inventário têm destinos válidos sem membro de tar com `..`.

O laboratório fixa `ROOT` por `__file__` (`validate.py:20–22`), mas resolve e invoca o caminho recebido por `--binary` (`acceptance.py:458–480`, `mutations.py:111–125`). Portanto os hashes históricos pertencem à árvore de laboratório e o sensor executado pode ser o release real absoluto da árvore I1. Não existe dependência do Cargo ou do Rust histórico na campanha. Os dois comandos já geram seus JSONs em `results/`; o wrapper pode copiá-los sem transformação e propagar os códigos existentes. Hash do binário antes/depois, proibição de compilação concorrente e conferência independente de build/fontes impedem atribuir a prova à árvore histórica. Os 30 arquivos preservados e quatro deltas reais continuam sendo conferidos fora do laboratório.

Isso permite reexecução posterior com o arquivo de baseline, a V1 identificada e o binário estável, sem restaurar produção à Foundation. Os `runtime_pending` genéricos da V1 não precisam ser reescritos: AC-09/11 exigem a reconciliação por testes reais e mantêm RSS com R1.

Nesta auditoria foi exercitada **somente a pré-condição de integridade**: uma árvore temporária recebeu cópias exatas dos 34 arquivos e das dependências acima, mais a V1 intacta, sem resultados/cache. `validate.integrity()` passou com **81 artefatos e 34 protegidos**. Acrescentar bytes ao README dessa cópia foi rejeitado especificamente por `protected_hash`; após restaurá-lo, acrescentar whitespace à fixture `cases.json` da cópia foi rejeitado por `artifact_hash`. Não houve monkeypatch de `ROOT` ou de esperado. Esse controle demonstra a viabilidade da montagem e a permanência dos controles; não é teste de um adaptador já entregue, tar já produzido ou campanha contra D0. Esses artefatos e suas negativas permanecem obrigações de I1-AC-01/09.

## Identidade do build

O auditor reproduziu a coleta atual de `build.rs` e seu enquadramento binário: prefixo `compass-build-v1` mais um byte NUL, caminhos ordenados, comprimento u64 little-endian do caminho UTF-8 e dos bytes. Os **21 inputs atuais** reproduzem `sha256:94866a9d405c283d1913c13e53ff0d5413823f3ce1a654e8bbc68619d7ddd4b0`, conforme a OS.

OS 110 define exatamente estes **sete acréscimos normativos**, todos existentes e presentes no índice congelado, sob `research/proposals/d0-c2/`:

- `CONTRATO_CANDIDATO.md`;
- `schemas/report.schema.json`;
- `schemas/source.schema.json`;
- `schemas/metric-policy.schema.json`;
- `schemas/limits.schema.json`;
- `schemas/error.schema.json`;
- `fixtures/metric-policy.json`.

A coleta existente de `crates/` já abrange os novos fontes/testes Rust autorizados. A extensão explícita inclui D0 fora de `research/contracts/`, sem absorver incidentalmente outras propostas/evidências. Manter o domínio não reutiliza o build antigo: os bytes e a lista efetiva mudam, e AC-10 exige recomputar o hash e compará-lo com o produtor real após o último input modificado. Nenhum build D0 futuro foi alegado nesta revisão.

## Encerramento

**Achados obrigatórios: nenhum.** Não há correção de contrato exigida antes da liberação da OS v1 identificada neste parecer. Implementação do adaptador, captura da baseline antes das edições, provas Rust/CLI, regressões, hashes finais e auditoria da entrega continuam nos aceites originais; este parecer não os antecipa nem os dispensa.

Verificações executadas: leitura focada de contratos/OS/harness; conferência dos 81 hashes e dos 34 protegidos; recomputação independente do build Foundation; verificação dos sete caminhos; integridade e dois negativos em cópia temporária. Conferência final confirmou preservação da OS, 81 artefatos, 34 históricos e dez arquivos duráveis V1; `git diff --check` passou. Não foram executados produção, self-test completo V1, campanha `--binary`, testes/build Rust ou R1. Nenhuma dependência foi instalada. Apenas este parecer novo foi escrito no projeto; OS, contratos, testes e esperados permaneceram intactos. Uso compartilhado da conta: 14% → 14%, diferença visível zero sujeita a arredondamento e outros trabalhos; não é contagem de tokens desta auditoria.

## Nota administrativa — versão liberada

19/10/2026. Após o parecer CONFORME acima, a coordenação liberou a OS-D0-I1 v1. Seu SHA-256 final é **`25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4`**. A cópia auditada em `/tmp/compass-plan-i1-_mv7_s3e/OS-D0-I1-auditada.md` conserva o hash `17e6fea97ea4d8267307bd2e5809c987661e365f8babd6aa5b2558d076feca63` registrado neste parecer.

Conferido o diff restrito: somente a troca do estado YAML para `LIBERADA_PARA_EXECUCAO` e uma linha de registro da liberação com link para este parecer. Ao desfazer apenas esses dois deltas em memória, os bytes coincidem integralmente com a versão auditada. **Conteúdo normativo, escopo e ACs permanecem idênticos; o veredito CONFORME aplica-se à OS liberada.** Esta nota preserva o parecer histórico, não constitui reauditoria ou reteste e não executa produção/I1/R1.
