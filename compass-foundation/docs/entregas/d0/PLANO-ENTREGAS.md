> **Histórico encerrado como instrumento de execução em 21/10/2026.** O [plano único](../../PLANO-IMPLEMENTACAO.md) substitui as OSs, as liberações administrativas e os prompts abaixo. Conteúdo preservado para consulta histórica; não representa a fila atual.

# D0 — próximas entregas

Criado em 18/10/2026; atualizado em 19/10/2026 após aceite de R1. V1, I1 corrigida e R1 foram auditadas e aceitas em seus escopos; BUG-004 está verificado. A entrega técnica D0 está encerrada, conforme [parecer R1 CONFORME](../../../research/audits/AUDITORIA-OS-D0-R1.md). R1 teve porte **padrão**: laboratório/evidências com produção protegida, seguido de conferência independente.

## Encerramento e linha de base

**D0-C2-A1: CONCLUÍDA**, com auditoria `CONFORME` em [AUDITORIA-OS-D0-C2-A1.md](../../../research/audits/AUDITORIA-OS-D0-C2-A1.md). T09/E36 está encerrada documentalmente. C1 e C2 também concluíram suas entregas documentais; a ativação do sensor é uma decisão separada.

Foundation preserva `init`, `snapshot` e `compare`; I1 acrescentou D0 à CLI e sua C01 foi verificada CONFORME. Branch `main`, sem commits. A baseline histórica Foundation continua registrada; a versão corrigida apta à R1 é `producer_build=sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`, binário `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`.

Conjunto técnico auditado, sem alteração nesta arquitetura:

- Índice: `research/proposals/d0-c2/SHA256SUMS`, SHA-256 `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`, contendo 81 entradas.
- Contrato: `research/proposals/d0-c2/CONTRATO_CANDIDATO.md`, SHA-256 `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e`.
- Política: `metric_policy_id=51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

## Ordem e avanço observável

| Etapa | Entrega e dependência | Execução recomendada | Situação |
|---|---|---|---|
| Ativação pela coordenação | [P1–P4 aprovadas e conjunto congelado](ATIVACAO.md), sem alteração dos hashes auditados | Nesta conversa; sem novo agente | Concluída em 19/10/2026 |
| D0-V1 | [OS-D0-V1.md](OS-D0-V1.md), versão 2, com C01–C03: oráculo/campanha e esperados independentes | GPT-6.1 Sol / Médio | CONCLUÍDA/ACEITA; auditoria corretiva CONFORME |
| D0-I1 | [OS-D0-I1.md](OS-D0-I1.md), incluindo correção C01 | GPT-6.1 Sol / Alto | CONCLUÍDA/ACEITA para integrar após C01 CONFORME |
| D0-I1-C01 | [OS-D0-I1-C01.md](OS-D0-I1-C01.md): referências compartilhadas | GPT-6.1 Sol / Alto | CONCLUÍDA/VERIFICADA; BUG-004 encerrado nesse escopo |
| D0-R1 | [OS-D0-R1.md](OS-D0-R1.md) + [contrato de aceite](CONTRATO-ACEITE-D0-R1.md): fluxo real, RSS e consolidação | GPT-6.1 Sol / Alto; nova conversa, um escritor | CONCLUÍDA/ACEITA; [auditoria CONFORME](../../../research/audits/AUDITORIA-OS-D0-R1.md) nos sete ACs |

V1 entregou ferramentas utilizáveis sem Rust D0 e a campanha para executá-lo quando disponível. I1 deve executar a campanha real contra seu release estável; R1 confere a implementação e as evidências para o aceite independente. Self-test não substitui execução do sensor. As provas sincronizadas são implementadas/testadas em I1 e conferidas em R1; RSS permanece medição de R1.

## Decisões aprovadas e liberação

Em 19/10/2026, Emanuel respondeu “Sim, aprovo” à pergunta que enumerou P1–P4. A decisão e a liberação estão registradas em [ATIVACAO.md](ATIVACAO.md). A preparação de 18/10 não atribuía essa aprovação ao pedido de arquitetura; a nova resposta fecha a pendência.

| ID | Decisão aprovada | Consequência |
|---|---|---|
| P1 | Pacotes externos explícitos nesta entrega técnica | Operador conserva/prepara revisões; captura automática não integra I1 |
| P2 | Massa bilateral linear, sensor `d0_line_mass_v1` | Arquivos grandes pesam mais; valores diferem do agregado logarítmico histórico |
| P3 | UTF-8 sem NUL, normalização e zeros do candidato | Algumas mudanças de bytes terão D0 zero; binários/transições ficam fora da métrica |
| P4 | Verificação de unchanged/vazios e resultado integral ausente sob bloqueio | Conteúdo faltante impede resultado integral; subconjunto conhecido fica separado |

O registro de ativação contém a evidência da aprovação, P1–P4, versões, hash do índice autorizado e `metric_policy_id`. Os 81 artefatos permanecem em seus caminhos e bytes auditados; o registro estabelece sua ativação atual sem alterar os rótulos históricos dos candidatos.

Uma revisão futura das decisões deve alterar explicitamente o contrato e os esperados afetados antes de seus consumidores; a OS V1 deverá ser reconciliada com os novos hashes se necessário. Não fazer o executor decidir produto nem criar esperados a partir de Rust.

## Correcoes da V1

A [auditoria OS-D0-V1 v2](../../../research/audits/AUDITORIA-OS-D0-V1.md), SHA-256 `5c3134a49ee4a4125d67c5769a564139ad0149c190a49c50c6f80769c8d126e0`, confirmou BUG-001/002/003. **As três correções foram implementadas e verificadas CONFORME** na [auditoria focada](../../../research/audits/AUDITORIA-OS-D0-V1-C01-C03.md), SHA-256 `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1`. V1 está aceita; 132 verificações, 119 cenários preparados e 11 mutações aprovadas na revisão. O protocolo abaixo preserva o histórico da sequência já encerrada, não constitui a próxima execução.

| Ordem concluída | Achado e avanço observável | Dependência satisfeita |
|---|---|---|
| [OS-D0-V1-C01](OS-D0-V1-C01.md) | BUG-001: paths válidos passam; campos de contagem mantêm limites u64 | Baseline auditada |
| [OS-D0-V1-C02](OS-D0-V1-C02.md) | BUG-002: links intermediários pendentes recebem motivo correto; ausência regular mantém distinção | C01 implementada e testes aprovados |
| [OS-D0-V1-C03](OS-D0-V1-C03.md) | BUG-003: E52 monta CRLF/LF distintos e sua matriz comprova a normalização | C02 implementada e testes aprovados |

**A execução ocorreu sequencialmente, na mesma conversa do implementador**, com arquivos de testes/cobertura/retorno compartilhados. As dependências técnicas entre elas não criaram uma rodada de aprovação de produto ou auditoria geral a cada arquivo. As instruções históricas seguintes foram satisfeitas e ficam registradas para rastreabilidade.

### Protocolo comum de execução e retorno

- Antes da primeira edição, inspecionar Git e salvar hashes/diff/cópias dos arquivos que serão tocados. Não há HEAD. A baseline inicial é a versão identificada na auditoria. C02/C03 consomem a árvore corrigida e os hashes registrados no retorno anterior, sem inventar hashes futuros ou desfazer mudanças autorizadas.
- Respeitar a lista fechada de cada OS; preservar as partes não afetadas. `fixtures.json`, `oracle.py`, `mutations.py`, originais C2, ativação, OS original v2, auditorias e produção permanecem protegidos. Os novos testes ficam nos módulos existentes autorizados; nenhuma dependência ou ferramenta de produção é adicionada.
- As listas fechadas se aplicam aos arquivos do projeto. Testes, cópias de baseline e logs auxiliares podem usar temporários em `/tmp`, sem sobrescrever inputs originais ou criar arquivos adicionais no projeto.
- Reproduzir o BUG e acrescentar controle comportamental que falhe antes da correção e passe depois, preservando os controles positivos. Asserções podem ser estendidas para corrigir o verificador, mas não enfraquecidas para acomodar a resposta errada. Os esperados vêm do contrato, não de Rust nem somente do modelo que está sendo corrigido.
- Acrescentar seções por OS/BUG em `verification/d0/RESULTADO-D0-V1.md`, sem apagar o resultado original. Registrar baseline, diff/hashes antes/depois, ACs e comandos. O estado apropriado do executor é correção implementada/testada, aguardando auditoria; não alterar o parecer anterior ou declarar fechamento pelo auditor.
- A baseline de 126 verificações e 116 cenários pode crescer com regressões novas. Atualizar documentação/evidências pertinentes conforme os testes reais; esses números não são uma meta que justifique retirar casos. Resultados de runtime continuam pendentes de D0 estável.
- Parar em regressão não resolvida ou necessidade de escrita fora da OS, identificando o BUG/AC e encaminhando à coordenação. Duas tentativas falhas do mesmo achado exigem reinvestigação antes de outra tentativa de patch. Não reiniciar uma auditoria irrestrita do aplicativo.

Comandos **já existentes**, a executar após cada mudança funcional das ordens:

```sh
python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Conferir também os arquivos não rastreados e os 34 protegidos por `evidence/preservacao.json`. Os self-tests incluem socket Unix local: se o ambiente retornar EPERM, solicitar a permissão técnica adequada ou registrar limitação; não transformar falha/skip em aprovação. Não há operação remota autorizada e não se usa Foundation como D0. Rust/fmt/Clippy/build não são necessários para estas correções Python.

### Auditoria e integração após as correções

Após C03, uma auditoria focada deve reavaliar **os mesmos BUGs/ACs**, reproduzir seus controles e conferir regressões afetadas e preservação dos inputs. O parecer original continua histórico; a nova revisão registra a versão corrigida e seus hashes. Não abrir outro planejamento se as correções cumprirem estas ordens.

A conferência foi concluída e V1 recebeu aceite final. Provas de CLI D0 real, corridas sincronizadas, checkpoints e RSS continuam com I1/R1 conforme o plano, sem serem tratadas como defeitos destas correções. A OS própria de I1 está agora preparada; C01–C03 não autorizavam iniciá-la por conta própria.

## R1 executada e aceita

A [auditoria C01 CONFORME](../../../research/audits/AUDITORIA-OS-D0-I1-C01.md), SHA-256 `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3`, fechou BUG-004. A [OS-D0-R1](OS-D0-R1.md) e seu [contrato operacional](CONTRATO-ACEITE-D0-R1.md) foram executados e conferidos: [parecer R1 CONFORME](../../../research/audits/AUDITORIA-OS-D0-R1.md), SHA-256 `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9`. O [prompt R1](../../COORDENACAO.md#prompt-pronto--d0-r1) é histórico, não uma próxima execução.

R1 criou os nove arquivos de laboratório/evidências em `verification/d0-r1/`. A auditoria reproduziu dois fluxos com snapshots reais, pacotes explícitos, quatro negativos/restaurações e três amostras de recursos por carga; conferiu independentemente a matemática, os bytes e a matriz E01–E55. Produção, V1, I1, contratos e baselines foram preservados. Provas I1/C01 foram herdadas com hashes/versão conferidos, sem repetir suites/build inteiros. A coordenação registra R1 concluída/aceita; próxima capacidade requer escopo e OS próprios conforme o documento de produto.

## Paralelismo e integração

V1/I1 já concluídas tiveram fronteiras de escrita separadas. Na R1 não há benefício em escritores paralelos: um runner constrói as cargas, executa o binário estável e produz os artefatos. A conferência posterior é independente; não correr compilação/alteração de produção durante medições.

Os testes usam árvores e pacotes próprios em `TemporaryDirectory`. V1 não compila, não escreve em `target/` e não executa um binário enquanto I1 o substitui. A campanha real começa quando a coordenação disponibilizar um artefato estável identificado por hash. Corridas com barreiras e checkpoints internos são responsabilidade dos testes Rust I1 e da integração R1; V1 registra a correspondência dos casos, sem alterar Rust ou inventar flags públicas de teste.

Revisão de V1: conferir independência dos esperados, rejeição das mutações e cobertura declarada. Divergências fundamentadas geram BUG/ajuste da OS correspondente. A auditoria anterior não será repetida integralmente por padrão.

## Reversão e fronteiras

Nenhuma ação remota foi autorizada. As novas ferramentas operam em cópias temporárias; entradas congeladas e gabaritos Foundation permanecem protegidos. Para reverter V1, retirar apenas seus novos arquivos após conferir alterações locais. Uma mudança de contrato suspende consumidores afetados até reconciliação; não exige descartar trabalho independente já válido.
