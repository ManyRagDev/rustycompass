# Estado do Compass Foundation

Atualizado em 19 de outubro de 2026. Versão Foundation 0.1.0; schema 1; contrato 0.1.0.

Foundation está implementada e verificada localmente: núcleo Rust e CLI com `init`, `snapshot` e `compare`. O binário release está em `target/release/compass`. As instruções de execução estão no [README](README.md).

**Situação atual:** Foundation/D0/R1 e RD-I1 corrigida estão aceitas. A [auditoria C01](research/audits/AUDITORIA-OS-RD-I1-C01.md) foi **CONFORME** nos seis aceites: BUG-005 verificado e RD-AC-07 resolvido; BUG-001–004 e BUG-RD-C-001 permanecem encerrados. Próxima execução: [OS-RD-I2](docs/entregas/rd/OS-RD-I2.md), **LIBERADA** após [parecer prévio CONFORME](research/audits/AUDITORIA-CONTRATO-OS-RD-I2.md), para leitura de contexto por linhas e continuação na revisão declarada. Nova conversa, GPT-6.1 Sol / Alto, um escritor e conferência posterior. Read não foi implementado nesta preparação. HIP-I1-01 permanece observação separada.

## Autoridades

- Produto: [edição original, capítulos 7, 17 e 20](../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md).
- Regras implementadas: [contrato ativo](research/contracts/FOUNDATION.md), [schema](research/contracts/transport.schema.json) e fixtures congeladas.
- Planejamento de origem: [análise inicial](../docs/ANALISE_INICIAL.md), etapas F0–F4.
- Aceite e revisão: [verificação independente](verification/ACCEPTANCE.md).

## Entrega verificada

| Etapa | Resultado |
|---|---|
| F0 | Contrato, formato canônico, gabaritos e identidade dos estados definidos antes dos consumidores |
| F1 | Workspace Cargo, Rust 1.90.0 e dependências fixados; repositório Git próprio em `compass-foundation/` |
| F2 | Observação por streaming, exclusões, cobertura, revisões SHA-256 e publicação sem sobrescrever manifestos |
| F3 | Comparação offline determinística, diferenças conhecidas e regiões desconhecidas explícitas |
| F4 | Release compilada; testes, revisão independente e carga reproduzível de recursos executados |

A implementação foi dividida entre agentes de contratos/comparação e observação; um terceiro agente escreveu e executou verificações independentes. A integração conferiu os fontes, executou o conjunto e corrigiu dois problemas encontrados na revisão: classificação de erro de política inválida e perda de bytes nos caminhos absolutos dos recibos.

Passaram **40 testes Rust** (20 unitários e 20 de contrato), **23 testes pela CLI** e a rejeição de **três mutações deliberadas do protocolo**, sem skips ou testes ignorados. Formatação, Clippy com avisos como erro e build release passaram. As mutações não constituem uma campanha de mutação de todo o código.

O build é identificado pelo hash `producer_build` dos fontes e contratos; o binário também tem SHA-256 registrado. Os identificadores completos estão no [registro do release](verification/reference/release-verification.json) e no [registro de integração](verification/evidence/integration.json). Nenhum commit foi criado; os resultados não são atribuídos a um commit inexistente.

A [medição de recursos](verification/evidence/resource-profile.json) usa 1.000 arquivos de 4.096 bytes em 100 diretórios, três snapshots e duas comparações, com criação/modificação/remoção conferidas. Registra duração, CPU, RSS, bytes lidos e disco ocupado junto da máquina e do build. É uma execução local com cache não controlado, sem meta de escala ou inferência sobre eficácia de recuperação.

## Limites desta entrega

- Linux com `openat2` (kernel 5.6+). Windows e macOS ainda não foram implementados ou validados.
- Cada snapshot representa uma janela de observação; não promete atomicidade do acervo vivo.
- Permissões, instabilidade e limites produzem parcialidade, sem digest de um estado completo desconhecido.
- Duração e cancelamento são cooperativos; I/O bloqueado pode atrasá-los. Não há garantia global de RAM ou disco máximos com a política padrão sem limites.
- Manifestos guardam hashes e metadados; os conteúdos históricos não são retidos. Identidade/hash não autenticam a autoria de um manifesto.
- D0 e busca literal RD estão implementados e aceitos nos escopos contratados. Os recursos medidos são descritivos, sem SLO aprovado. Leitura de contexto está preparada; busca lexical/indexação, grafos, embeddings, MCP e GUI permanecem etapas posteriores.

## Próximo problema

Execução organizada em [coordenação das entregas](docs/COORDENACAO.md). **D0-C1 — Contrato do sensor D0** produziu a [proposta verificável](research/proposals/D0_V0.1.md), em 14 de outubro de 2026, usando `codex-arquiteto`, sem implementação. Recomenda pacotes externos de bytes verificados por hash/tamanho, diff mínimo de linhas e agregação por massa bilateral de linhas, com indisponibilidade explícita. Foram conferidos 35 exemplos documentais, incluindo 30 contas de LCS/contagens por enumeração independente, duas agregações, quatro decimais e seis vetores SHA-256; a suíte de produção não foi executada nesta entrega documental.

A coordenação leu a proposta e liberou **D0-C2 — Conferência independente**, com prompt em `docs/COORDENACAO.md` e recomendação GPT-6 Astra / Alto. A triagem destacou pesos novos, disponibilidade sob limites, orçamento lógico de memória e configuração da CLI como pontos de revisão. O hash recomputado dos 21 inputs de fonte/contrato continua correspondendo ao build Foundation verificado. As contas de C1 não foram repetidas pela coordenação; a conferência matemática independente pertence a C2.

Escolhas, schema, perfil de recursos e gabaritos D0 foram revisados em C2/A1 e aprovados em 19/10/2026. A ativação registra o conjunto autorizado por hashes, mantendo os artefatos originais preservados. V1 e a implementação Rust/CLI corrigida foram entregues, auditadas e aceitas para integração; R1 fecha o fluxo reproduzível, as medições reais e a consolidação de evidências.

R1 foi executada e aceita após conferência independente. A recuperação direta literal RD-I1 e a correção de transporte [RD-I1-C01](docs/entregas/rd/OS-RD-I1-C01.md) foram entregues, auditadas e aceitas. Próximo incremento: abrir e continuar o contexto dos resultados, conforme o [plano RD](docs/entregas/rd/PLANO-ENTREGAS.md). Os registros datados abaixo preservam o histórico; o estado atual é o informado no início deste arquivo.

## D0-C2 — conferência entregue; ativação pendente

Em 14/10/2026, `codex-arquiteto` concluiu exclusivamente a revisão independente C2: [parecer](research/proposals/D0_C2_REVISAO.md), [candidato e índice](research/proposals/d0-c2/INDEX.md), schemas, E01–E50 iniciais, fixtures-base e [evidência reproduzível](research/proposals/d0-c2/evidence/conferencias.json). Parecer **pendente para ativação**: coordenação/Emanuel precisam decidir pacotes externos, peso linear, universo/normalização/zeros e rigor de evidência/disponibilidade (P1–P4). Recomendações não foram tratadas como aprovadas.

Fechados tecnicamente binding/serialização/sharing, prioridades de unknown/exclusões/motivos, ledger lógico de memória (sem garantia RSS), reservas de bytes/células, CLI candidata limits/deadline, transporte/enums/null e validações cruzadas. E30 foi alterado com justificativa explícita: prefixo ao teto s+1 não recebe hash completo sem EOF. Na entrega C2 original, igualdade sobre teto de células permanecia indisponível; D0-C2-A1 substitui essa regra no candidato. As demais contas C1 foram confirmadas; causas/mudanças estão no parecer.

Conferências executadas: 39 LCS literais, 21 agregações, 961 pares pequenos de propriedades, normalização/hash/arredondamento, 5 schemas e 10 relatórios/inputs/reservas, negativos de transporte/semântica. O checker C2 não é laboratório V1. Os 21 inputs do build Foundation reproduzem a identidade registrada; inventário de 34 arquivos preexistentes confirma preservação de código/contratos/gabaritos/C1/documento principal. Nenhum teste/build de produção, sensor, retenção, CLI ou V1/I1 iniciado. Contrato e fixtures D0 são candidatos inativos; V1/I1 aguardam congelamento e liberação da coordenação.

## Conferência da coordenação — ajuste C2-A1 preparado

Em 14/10/2026, a coordenação reproduziu o checker documental C2, conferiu as 80 entradas de SHA256SUMS e os 34 arquivos protegidos; todos passaram e a identidade dos inputs Foundation permaneceu igual. Recomenda ajustar a reserva quadrática de T09/E36: sequências normalizadas exatamente iguais, com fontes verificadas e demais admissões satisfeitas, podem ter LCS exata sem reservar células de diff. Trata-se de alteração proposta da política de disponibilidade, ainda não aplicada ao candidato.

O prompt delimitado **D0-C2-A1** está em `docs/COORDENACAO.md`, para a mesma conversa C2, com recomendação GPT-6.1 Sol / Alto. P1–P4 seguem como recomendações de produto, sem aprovação atribuída a Emanuel. Nenhum contrato foi ativado, nenhum candidato foi alterado nesta conferência e V1/I1 permanecem pendentes. Somente estado e coordenação foram atualizados.

## D0-C2-A1 — ajuste documental da igualdade executado

Em 18/10/2026, a ordem delimitada D0-C2-A1 alterou somente os candidatos D0 e seus registros. A regra de células agora é `normalized_lines_exact_equal_zero_else_n0_times_nt_v1`: depois de fonte verificada, texto, normalização, limite de linhas e admissão de memória, sequências normalizadas exatamente iguais definem `l=n0=nt`, `a=r=c=0` e reservam zero células. Hash, tamanho e contagem de linhas isolados não acionam essa regra; sequências diferentes preservam `n0*nt`, os tetos e o débito anteriores.

E36 agora é complete 0/8000 mesmo com 4.000×4.000 linhas; E51–E55 cobrem saldo zero, normalização com bytes distintos, fontes ausentes/adulteradas antes da igualdade, mesma contagem diferente e saldo preservado para a unidade seguinte. O `metric_policy_id` candidato mudou para `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`. O checker C2 passou com 55 casos, 44 LCS literais, 23 agregações, 961 pares pequenos, 5 schemas e 10 relatórios/inputs. Foundation, proposta C1 e código de produção não foram alterados; V1/I1 continuam pendentes de P1–P4, ativação e liberação da coordenação.

## Encerramento D0-C2-A1 e próxima ordem

**D0-C2-A1 declarada CONCLUÍDA em 18/10/2026**, a pedido de Emanuel, após [auditoria CONFORME](research/audits/AUDITORIA-OS-D0-C2-A1.md). T09/E36 encerrada no nível documental; não há achado obrigatório aberto desse ajuste. A auditoria reproduziu o checker, os 81 hashes e a preservação dos 34 arquivos, com conferências independentes dos exemplos novos.

A skill `arquiteto-de-contratos` preparou o [plano de entregas D0](docs/entregas/d0/PLANO-ENTREGAS.md) e a [OS-D0-V1](docs/entregas/d0/OS-D0-V1.md), próxima entrega do plano: verificador independente e campanha executáveis. A OS está **PREPARADA**, com escritas fechadas em `verification/d0/`, aceites e retorno próprios; não foi executada. P1–P4 continuam sem decisão registrada e condicionam a ativação e a execução de V1. I1 receberá OS própria e poderá avançar paralelamente após liberação, com arquivos e testes isolados. Nenhuma produção, contrato candidato ou gabarito foi alterado nesta arquitetura.

## Aprovação e ativação D0 — 19/10/2026

Emanuel aprovou explicitamente P1–P4 com a resposta “Sim, aprovo”, após pergunta que enumerou as quatro recomendações. A coordenação criou `docs/entregas/d0/ATIVACAO.md`, congelou o conjunto de 81 artefatos pelo hash do índice `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` e liberou OS-D0-V1 versão 2. O `metric_policy_id` permanece `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Resolvido o `BLOQUEIO_DE_PRODUTO` reportado pelo implementador por ausência de ativação. Retomar V1 na mesma conversa, respeitando o escopo e os aceites existentes. Os 81 artefatos e os 34 arquivos protegidos foram conferidos íntegros; a liberação não altera código ou critérios nem declara implementação/runtime concluídos. I1 ainda depende de sua OS própria.

## V1 auditada — correções preparadas em 19/10/2026

A [auditoria de V1](research/audits/AUDITORIA-OS-D0-V1.md) reproduziu 126 verificações e 11 mutações, mas confirmou BUG-001 (path válido tratado como u64), BUG-002 (link intermediário pendente classificado como ausência) e BUG-003 (estímulo/mapeamento E52). Veredito **NAO_CONFORME**; entrega ainda sem aceite final. Os 135 arquivos auditados foram preservados e apenas o parecer foi acrescentado.

A pedido de Emanuel, `arquiteto-de-contratos` preparou [C01](docs/entregas/d0/OS-D0-V1-C01.md), [C02](docs/entregas/d0/OS-D0-V1-C02.md) e [C03](docs/entregas/d0/OS-D0-V1-C03.md). Ordem sequencial na mesma conversa do implementador, com regressões comportamentais por BUG, seguida de auditoria seletiva conjunta. Nenhuma correção foi implementada nesta arquitetura; P1–P4, o contrato ativo e os gabaritos permanecem aprovados e preservados. Não há novo bloqueio de produto.

## Aceite V1 e preparação de I1 — 19/10/2026

A coordenação registra **D0-V1 CONCLUÍDA/ACEITA**, com base na auditoria focada CONFORME de C01–C03, SHA-256 `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1`. A auditoria reproduziu 132 verificações, 119 cenários preparados e 11 mutações, com controles independentes dos três BUGs. Campanha real contra Rust D0 não foi executada; esse aceite é do laboratório V1.

A [OS-D0-I1](docs/entregas/d0/OS-D0-I1.md) prepara fonte/recursos → sensor/relatório → CLI, recomendando GPT-6.1 Sol / Alto, um escritor e revisão independente posterior. A V1 confere hashes históricos de fontes Foundation que I1 precisa alterar; a ordem prevê baseline arquivada e laboratório intacto em temporário, executando o binário I1 real. Contratos/esperados/verificador permanecem protegidos, com deltas Rust autorizados conferidos separadamente. Nenhuma lógica de produção foi alterada nesta preparação.

A auditoria prévia independente da OS-I1 foi CONFORME, sem achado obrigatório; comprovou também a montagem histórica e a rejeição de adulterações em cópia. A coordenação libera a execução. A preparação conferiu links, whitespace, 81 artefatos e 34 protegidos; os 21 inputs Foundation continuam reproduzindo seu build anterior. Não foram executados testes/build de produção nesta arquitetura.

## I1 auditada e correção BUG-004 preparada — 19/10/2026

I1 entregou Rust/CLI e campanha real contra o novo binário. A [auditoria da entrega](research/audits/AUDITORIA-OS-D0-I1.md), SHA-256 `4a9000d6e75c9d3e30634113c918ea32db7e9e6a795c77630c170e932f975910`, reproduziu 61 testes Rust, 23 testes Foundation, três mutações Foundation, 253 verificações V1 e 11 mutações D0; 90 casos matemáticos adicionais passaram. O veredito foi **NAO_CONFORME**: BUG-004 foi reproduzido por intervenção sincronizada no binário real e afeta I1-AC-03/04/07.

A [OS-D0-I1-C01](docs/entregas/d0/OS-D0-I1-C01.md) delimita a correção, regressão permanente antes/depois, preservação da leitura única e renovação de build/evidências. Recomendação: mesma conversa do implementador, GPT-6.1 Sol / Alto, um escritor e auditoria focada posterior. HIP-I1-01 não foi convertida em obrigação. Esta arquitetura altera somente planejamento/estado; código, contrato ativo, V1, baseline e parecer permanecem preservados.

A auditoria prévia da OS-I1-C01 foi CONFORME, sem achado obrigatório; a coordenação libera sua execução. Essa liberação não fecha BUG-004 nem declara I1 aceita. A OS-I1 original permanece idêntica para preservar o binding da baseline V1.

## C01 verificada, I1 aceita e R1 preparada — 19/10/2026

A [auditoria C01](research/audits/AUDITORIA-OS-D0-I1-C01.md), SHA-256 `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3`, foi **CONFORME**: BUG-004 verificado, C01-AC-01–06 cumpridos e não conformidade correspondente dos I1-AC-03/04/07 resolvida. A coordenação registra I1/C01 concluídas e aceitas para integrar. Foram reproduzidos 64 testes Rust, 23 Foundation, 253 verificações V1, 14 mutações e prova pública sincronizada antes/depois; não houve medição RSS.

Baseline R1: binário `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`, produtor dos 37 inputs `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`. A [OS-D0-R1](docs/entregas/d0/OS-D0-R1.md) e seu [contrato operacional](docs/entregas/d0/CONTRATO-ACEITE-D0-R1.md) estão liberados para integração em laboratório: dois fluxos sintéticos reais, seis amostras D0/RSS e matriz de provas novas/herdadas. Produção e evidências antigas ficam protegidas. R1 termina pendente de conferência independente; não se declara homologação por preparar a ordem.

## R1 verificada e D0 aceita — 19/10/2026

A [auditoria R1](research/audits/AUDITORIA-OS-D0-R1.md), SHA-256 `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9`, foi **CONFORME nos sete aceites**, sem achado obrigatório. Reproduziu em cópia isolada os 24 comandos públicos, M/L, quatro negativos/restaurações e seis novas medições nativas; conferência separada de bytes/matemática/cobertura e quatro adulterações específicas também passaram. As suites I1/C01 foram herdadas pela mesma versão, sem alegar reexecução nesta etapa.

A coordenação registra **D0-R1 CONCLUÍDA/ACEITA** e encerra a entrega técnica D0 contratada. Resultado original `4b714d6641069cb10e000a755ce81d918fafbb5e91df29cb2bf0f21211af2f2d` e proveniência `beb99704a1ecb0f55525aae327d03222025884a7734bae8f6fac5fdcef0e13fb` permanecem intactos, assim como os nove artefatos, binário/37 inputs, contratos e evidências anteriores. Somente estado/plano/coordenação recebem o registro administrativo posterior ao parecer, sem regravar a baseline histórica da R1. Próximas capacidades não foram iniciadas.

## Recuperação direta preparada e liberada — 19/10/2026

A [OS-RD-I1](docs/entregas/rd/OS-RD-I1.md) executará a primeira busca literal pela CLI, com pacotes verificados já suportados, linhas/offsets brutos, cobertura/seleção distintas e limites explícitos. O [contrato RD v0.1](docs/entregas/rd/CONTRATO-RD-V0.1.md), SHA-256 `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72`, não altera a semântica Foundation/D0. Busca lexical/indexação, grafo, embeddings, manutenção incremental e MCP continuam fora desta fatia.

A [auditoria prévia](research/audits/AUDITORIA-CONTRATO-OS-RD-I1.md), SHA-256 `339ce58e2e1518d722ebe7e0846381d161d5432adea96d1ead52663fbe4ab4f5`, foi **CONFORME**, após fechar a ambiguidade contratual BUG-RD-C-001 dos checkpoints. A coordenação libera RD-I1, um escritor, nova conversa GPT-6.1 Sol / Alto; nenhuma implementação ou suite de produção foi executada nesta arquitetura. Código, binário C01, 37 inputs, 81 artefatos D0 e evidências anteriores permanecem preservados. O novo build só incorporará o contrato RD quando a implementação alterar build.rs conforme a OS.

## RD-I1 auditada e BUG-005 preparado — 19/10/2026

A [auditoria de entrega RD](research/audits/AUDITORIA-OS-RD-I1.md), SHA-256 `7700e7e1f310994e2226167c177eb6aa377e3268d90f0a87463ba53d04fd964f`, reproduziu 74 testes Rust, 151 comandos RD/cinco mutações/três medições, 23 Foundation/três mutações e 253 verificações V1/11 mutações D0. Trinta controles independentes adicionais passaram. Não houve skip final; socket Unix exigiu permissão técnica.

Veredito **NAO_CONFORME**: BUG-005 confirmado, RD-AC-07. Query literal `--json` separada da opção faz o erro de parsing usar compacto indevidamente; a forma equivalente com `=` e sem flag real usa corretamente legível/newline. Os demais aceites passaram. Release auditado `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d`, produtor dos 41 inputs `sha256:6192a943a657f23ebaea7267eea67f63ee482741c85c37d44634817113bfebf9`.

A coordenação libera a [OS corretiva RD-I1-C01](docs/entregas/rd/OS-RD-I1-C01.md), porte curto: dois deltas existentes (CLI/teste) e cinco arquivos novos em verification/rd-c01, preservando integralmente verification/rd e o núcleo/contrato. Mesma conversa de implementação, GPT-6.1 Sol / Alto, prova antes/depois e conferência focada posterior. A correção não foi implementada nesta preparação; BUG-005 permanece aberto e próxima capacidade não foi iniciada.

## RD-I1-C01 verificada e leitura RD-I2 liberada — 19/10/2026

A [auditoria corretiva](research/audits/AUDITORIA-OS-RD-I1-C01.md), SHA-256 `1dd71986c3e533b34c2fc739d66322b858dfffaba917668afd6c37a6e9f6d9f8`, foi **CONFORME** nos seis aceites. O release anterior falhou em exatamente um dos 48 controles, por BUG-005.format; o corrigido passou em 48/48. A regressão permanente falhou com o fonte anterior pela mesma asserção; 76 testes Rust, fmt/Clippy, campanhas RD/Foundation/D0 e três medições novas passaram. Preservados os 187 arquivos protegidos da entrega e os dois deltas autorizados; nenhum achado obrigatório novo. A coordenação aceita RD-I1/C01 e fecha BUG-005/RD-AC-07.

Baseline aceita: release `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8`, produtor dos 41 inputs `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`. Resultado C01 `e507f97b915430c9b563921a4450ef6fc2cf00fbb685cb85cd4a625f45610637` e proveniência `57d83dee09befb2bff33e9af7d73ad569e865f801f7a650991b61fe6322bb247` conferidos e preservados.

Próximo incremento do documento principal §§6.3/17.3: [OS-RD-I2](docs/entregas/rd/OS-RD-I2.md), `compass read` por path_key/faixa de linhas, offsets brutos e continuação explícita na mesma revisão. O [contrato de leitura v1](docs/entregas/rd/CONTRATO-RD-LEITURA-V0.1.md), SHA-256 `54b4826516d9552598a71cba05c09403e64022ba9ba4f1a12f936c358c0e44d6`, reutiliza os controles/limites RD e mantém o objeto integralmente verificado antes de entregar conteúdo. Escrita fechada em cinco existentes/onze novos, um escritor em nova conversa GPT-6.1 Sol / Alto; três fases sequenciais e auditoria posterior.

A [auditoria prévia de contrato](research/audits/AUDITORIA-CONTRATO-OS-RD-I2.md), SHA-256 `ff62c4ff8e501a67a7811e072bf1ded5e8d49a045af766099d5e869e8003816a`, foi **CONFORME**, sem achado obrigatório. A coordenação liberou somente o estado administrativo da OS; SHA final `5703463f5ad37ace1c973b5b2d91ddc8b35c262b2355f3bc69dae6f92e45ba13`. Read ainda não foi implementado; fonte, testes, executável e evidências anteriores permanecem intactos durante esta arquitetura. Índices, manutenção e MCP não integram a OS.
