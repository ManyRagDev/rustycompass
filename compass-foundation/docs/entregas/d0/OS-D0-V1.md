# OS-D0-V1 — verificador independente do sensor D0

```yaml
id: OS-D0-V1
versao: 2
estado: LIBERADA
porte: padrao
origem: plano D0; D0-C2-A1 auditada CONFORME; pedido de próxima entrega em 18/10/2026
baseline: main sem commits; árvore local preservada; conjunto C2-A1 identificado abaixo
objetivo: entregar oráculo e campanha executáveis que detectem desvios do contrato D0 sem importar produção
dependencias:
  - decisão registrada de P1–P4 e ativação pela coordenação
  - conjunto de contrato/schema/esperados com hashes autorizados
autorizacoes_remotas: nenhuma
```

## Entrada e condição para executar

Diretório de trabalho: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.

**Execução liberada em 19/10/2026**, após Emanuel aprovar P1–P4. A autorização e os hashes do conjunto congelado estão em [ATIVACAO.md](ATIVACAO.md). Antes da primeira escrita, o executor deve conferir esse registro e os identificadores abaixo; o bloqueio anterior por ausência de ativação está resolvido. Se o registro estiver ausente ou divergir dos inputs autorizados, reportar o bloqueio específico, sem regenerar hashes ou escolher outra política. A versão 2 altera somente o estado e a referência de liberação; escopo, AC-01–AC-08 e comandos permanecem iguais.

O conjunto técnico de referência é:

| Identificador | Valor |
|---|---|
| SHA-256 de `research/proposals/d0-c2/SHA256SUMS` | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| SHA-256 de `CONTRATO_CANDIDATO.md` | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| `metric_policy_id` default | `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5` |
| SHA-256 do parecer de A1 | `9e89ad9217b50f475fafd4dc74ce7edae37345187ca00225cfa8d8f8b9b51c20` |

Conferir o hash do próprio índice e suas 81 entradas. Se a ativação autorizar outra versão, a coordenação deve atualizar esta OS antes da execução. Não recalcular o índice para apagar uma divergência.

## Leitura delimitada

1. `AGENTS.md`, `COMPASS_STATE.md`, esta OS, seu `PLANO-ENTREGAS.md` e o registro de ativação.
2. `research/proposals/d0-c2/CONTRATO_CANDIDATO.md`, `INDEX.md`, schemas e E01–E55, bytes, políticas, relatórios, pacotes e evidências pertinentes. `research/audits/AUDITORIA-OS-D0-C2-A1.md` comprova o encerramento do ajuste.
3. `research/contracts/FOUNDATION.md` e `transport.schema.json`, para validar/referenciar tipos existentes; `verification/ACCEPTANCE.md`, `verification/reference/acceptance.py` e `mutations.py`, para reaproveitar convenções de CLI e temporários sem alterar esses arquivos.
4. Código Rust apenas para localizar uma interface ou entender uma dependência concreta. Não importar Rust, chamar funções internas da produção ou transportar seu algoritmo/esperados para o oráculo. Não repetir auditoria geral de Foundation nem ler o legado.

## Fronteira de escrita

Todos os arquivos abaixo são **novos** e constituem a lista fechada de escrita:

| Caminho | Responsabilidade |
|---|---|
| `verification/d0/oracle.py` | Normalização, LCS independente, equações e arredondamento; somente laboratório |
| `verification/d0/validate.py` | Parser estrito, schemas, hashes/binding e conferências semânticas do relatório contra os inputs/esperados |
| `verification/d0/cases.py` | Montagem reproduzível dos cenários em temporários e ligação dos testes a E01–E55 |
| `verification/d0/acceptance.py` | Entrada executável: `--self-test` ou `--binary <path>`, evidências e códigos de saída |
| `verification/d0/mutations.py` | Negativos do protocolo, sem alterar produção, contratos ou esperados |
| `verification/d0/fixtures.json` | Esperados adicionais literais e receitas/referências aos casos congelados; escritos antes do uso de Rust D0 |
| `verification/d0/coverage.json` | Mapa E01–E55 → teste/camada/estado e responsável pela prova restante |
| `verification/d0/ACCEPTANCE.md` | Como reproduzir, interpretar resultados e distinguir offline de runtime |
| `verification/d0/RESULTADO-D0-V1.md` | Baseline, alterações, aceites, verificações efetivas e pendências para R1 |
| `verification/d0/.gitignore` | Ignorar somente `results/` nesta pasta |
| `verification/d0/results/self-test.json` | Saída gerada das verificações sem binário |
| `verification/d0/results/acceptance.json` | Saída gerada ao testar um binário D0 estável |
| `verification/d0/results/mutations.json` | Saída gerada dos negativos, identificando a modalidade executada |

Arquivos fora dessa lista são protegidos, inclusive Rust/Cargo, `research/contracts/`, `research/proposals/`, auditorias, scripts/gabaritos Foundation, `COMPASS_STATE.md`, `docs/COORDENACAO.md`, esta OS, ativação e documento principal original. Os resultados de runtime só são gerados quando essa execução ocorrer. Se um caminho já existir com trabalho de outra entrega, inspecionar o diff e conciliar com a coordenação; não sobrescrever.

Nova dependência de projeto, arquivo fora da lista ou necessidade de hook Rust exige `BLOQUEIO_DE_FRONTEIRA` e decisão da coordenação. Não instalar pacotes nem editar manifests nesta OS. Usar biblioteca padrão e o jsonschema já disponível; aceitar `--cached-dependencies /home/many/.cache/uv/archive-v0` como na conferência C2, sem tratar esse caminho como requisito portável.

## Comportamento a entregar

O modo `--self-test` deve funcionar sem sensor D0: conferir integridade dos inputs, oráculo, schemas, relatórios congelados e mutações offline. O modo `--binary` deve invocar a CLI pública com `subprocess`, tempo de guarda do laboratório e árvores/pacotes temporários próprios. Não reconstruir nem substituir o binário; registrar seu SHA-256 antes/depois e exigir estabilidade.

O oráculo deve calcular LCS exata por procedimento próprio e conferir exemplos pequenos por método independente. Para igualdade grande, usar a identidade matemática explícita; não executar diff quadrático para provar E36. Normalização opera nos bytes cujos hashes/tamanhos foram conferidos. Não usar contagem de linhas ou hash declarado como prova de igualdade. Metadados, produtores reais e IDs de snapshots são conferidos conforme seus inputs, sem trocar valores matemáticos esperados por saídas do binário.

Reutilizar o contrato, os cinco schemas e os vetores existentes por leitura. `json.dumps(sort_keys=True)` só pode ser tratado como canônico no subconjunto cuja equivalência a JCS foi demonstrada; documentar o domínio e conferir ordenação/escapes exigidos pelo transporte. Não anunciar implementação geral de JCS sem a verificação correspondente. Relatórios e entradas rejeitam duplicatas, BOM JSON, trailing data, UTF-8 inválido e números não finitos; schema não substitui invariantes semânticas.

Os resultados devem informar modalidade, IDs efetivamente conferidos, falhas, skips/bloqueios justificados, provas de runtime pendentes, hashes do conjunto de fixtures e identidade do binário/build quando houver execução real. Falha retorna código não zero. Binário sem comando D0 é bloqueio/falha explícita, nunca campanha aprovada com zero testes. Um `--self-test` aprovado pode deixar runtime pendente, com essa distinção explícita no JSON e no resultado da OS.

## Aceites

| ID | Dado / quando / então |
|---|---|
| AC-01 | Dado o conjunto autorizado, ao iniciar, verificar o hash do índice, os artefatos e a política. Adulterar uma cópia de fixture/esperado deve ser detectado; os originais permanecem intactos. |
| AC-02 | Dados E01–E55 e vetores, a matriz associa todos os IDs a testes e camadas reais. Conferir LCS/equações/agregados, massa de unchanged, adição/remoção/renomeação, vazios, A→B→A, normalização, não texto, zeros e arredondamento. Nenhum esperado depende de executar Rust D0. |
| AC-03 | Dados E36/E51–E55, igualdade normalizada verificada custa zero células acima do teto e com saldo zero; preserva denominador e saldo das seguintes. Mesma contagem com sequências diferentes mantém `n0*nt`; fonte ausente/adulterada impede a igualdade disponível. |
| AC-04 | Dados relatórios e seus inputs, validar transporte e semântica: disponibilidade/global vs subconjunto, prioridades unknown/exclusões, binding bruto, ordem nativa, u64, IDs/hashes, equações, reservas de bytes/células e memória lógica. Incluir exemplos schema-válidos semanticamente falsos que o verificador rejeita. |
| AC-05 | Dado um binário D0 estável quando disponível, a campanha monta fontes/manifestos identificados e verifica relatório, exit, bytes canônicos, repetibilidade e imutabilidade dos inputs. Cobrir fonte omitida, binding errado, objetos faltantes/adulterados, descritor não canônico e compartilhamento permitido/proibido. Preparar a campanha sem declarar que foi executada contra o Foundation atual. |
| AC-06 | Os cenários de runtime cobrem tipos especiais/links estáticos, UTF-8/caminhos nativos, fronteiras e precedências de recursos, EOF/prefixo, saída limitada e cancelamento. Em `coverage.json`, separar provas pela CLI das corridas/checkpoints que exigem sincronização Rust I1/R1. Timeout do laboratório não comprova deadline do produto; repetição probabilística não comprova ausência de corrida. Não esconder uma prova não executada como skip bem-sucedido. |
| AC-07 | Os negativos incluem: zero falso com fonte ausente; agregado sem unchanged; igualdade deduzida apenas de contagem; débito quadrático para igualdade; hash de política incoerente; subconjunto apresentado como integral. Cada mutação deve atingir uma asserção específica; falha de import, timeout ou erro genérico não conta como rejeição correta. Preservar controles válidos que passam, evitando um validador que rejeita tudo. |
| AC-08 | Documentação, cobertura e resultado distinguem conferências offline, campanha pronta e campanha executada. Declarar `CONCLUIDA` para V1 exige ferramentas funcionais, controles positivos/negativos aprovados e entrega dos cenários de runtime; homologação de Rust/CLI e das provas restantes pertence a R1. |

## Comandos e registro de verificação

Comando **já existente**, para conferir a integridade antes/depois:

```sh
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
```

Os comandos abaixo são interfaces a serem entregues por V1; **não existem nesta baseline**. Executá-los após criá-los e registrar os resultados:

```sh
python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
git diff --check
```

Revisar também os novos arquivos não rastreados, que `git diff --check` pode não incluir. Conferir os 34 arquivos protegidos pelos hashes de `research/proposals/d0-c2/evidence/preservacao.json`.

Quando a coordenação fornecer um release D0 estável, os comandos de runtime serão:

```sh
python3 verification/d0/acceptance.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
python3 verification/d0/mutations.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
```

O release Foundation atual não satisfaz esse pré-requisito. A regressão Foundation em R1 usa os comandos existentes de `verification/ACCEPTANCE.md`, além de testes Rust, fmt, Clippy e build pelo wrapper local; V1 não altera esse fluxo nem recompila durante I1.

## Risco, reversão e retorno

Risco principal: um verificador concordar com um erro da produção ou apresentar preparação como execução real. Controles: esperados anteriores ao sensor, métodos independentes, negativos com falha específica, proveniência e camadas declaradas. Sem nova camada de produto, retenção, banco, rede, GUI, busca ou benchmark nesta OS.

Executar casos apenas em temporários; inclusive alvos de links e pacotes adversariais são criados pelo laboratório. Não usar acervos ou segredos reais. Reverter retirando somente os arquivos novos desta OS depois de conferir alterações locais; não usar reset/clean sobre a árvore existente.

Modelo recomendado pelo plano: **GPT-6.1 Sol / Médio**, uma conversa para V1, sem subagentes por padrão. Se houver dúvida contratual, registrar o caso/AC concreto e devolver à coordenação; não elevar o esforço nem redesenhar o sensor automaticamente.

Para executar depois da liberação, o prompt é:

```text
Use executor-blindado para executar exclusivamente docs/entregas/d0/OS-D0-V1.md,
no diretório /home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation.
Leia AGENTS.md e confira a ativação exigida pela OS antes de escrever.
Preserve o conjunto autorizado e limite a escrita aos caminhos da ordem.
Ao concluir, registre RESULTADO-D0-V1.md com ACs, comandos e resultados reais,
runtime pendente e bloqueios. Não inicie I1/R1 nem envie mensagens a outros chats.
```
