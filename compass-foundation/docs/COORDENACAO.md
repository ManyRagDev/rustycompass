> **Histórico encerrado como instrumento de execução em 21/10/2026.** O [plano único](PLANO-IMPLEMENTACAO.md) substitui as OSs, as liberações administrativas e os prompts abaixo. Conteúdo preservado para consulta histórica; não representa a fila atual.

# Coordenação das próximas entregas

Emanuel definiu em 14 de outubro de 2026: esta conversa coordena; cada entrega relevante recebe prompt próprio e recomendação de modelo; a coordenação organiza dependências, paralelismo e conferência. Modelos abaixo são recomendações, não configurações já aplicadas.

## Estado e regras de execução

Foundation/D0/R1 e RD-I1 corrigida estão aceitas. A [auditoria C01](../research/audits/AUDITORIA-OS-RD-I1-C01.md) foi **CONFORME**: BUG-005 verificado, RD-AC-07 resolvido e demais fechamentos mantidos. Próxima execução: [OS-RD-I2](entregas/rd/OS-RD-I2.md), **LIBERADA** após [auditoria prévia CONFORME](../research/audits/AUDITORIA-CONTRATO-OS-RD-I2.md), leitura de contexto por linhas e continuação. Nova conversa, GPT-6.1 Sol / Alto, um escritor; prompt corrente ao final. HIP-I1-01 continua observação separada.

Cada nova conversa deve trabalhar no diretório `compass-foundation/`, ler `AGENTS.md` e receber somente o contexto pertinente. O estado durável permanece em `COMPASS_STATE.md`; este arquivo organiza a execução, sem substituir contratos ou criar outra visão de produto.

- A coordenação libera o próximo prompt após conferir a entrega anterior. Ajustes da mesma entrega continuam na conversa correspondente.
- Para retornar, informar arquivos alterados, decisões, verificações realmente executadas e pendências. Não declarar conclusão apenas porque o agente terminou.
- Uma conversa por entrega relevante. Não abrir conversas para cada arquivo ou correção pequena.
- Somente um escritor de código de produção por vez nesta fase. Verificação independente pode avançar em paralelo em arquivos separados, depois de congelados o contrato e os exemplos esperados.
- Nenhum consumidor começa com interface pendente. Mudança de contrato interrompe os consumidores afetados até a reconciliação explícita.
- Escalar modelo/esforço por dificuldade demonstrada; Extra Alto é excepcional. Preservar os critérios de qualidade ao reduzir o consumo.

## Sequência prevista

| ID | Entrega | Modelo / esforço recomendado | Dependência e paralelismo | Condição de encerramento |
|---|---|---|---|---|
| D0-C1 | Proposta de contrato e exemplos manuais | GPT-6.1 Sol / Alto | Primeira; sequencial | Fonte dos bytes, significado do sensor, disponibilidade, formato e aceite explícitos |
| D0-C2 | Conferência independente do contrato | Astra / Alto | Após C1; sem implementação concorrente | Contradições resolvidas; decisões de produto necessárias encaminhadas; contrato e gabaritos congelados |
| D0-V1 | Verificador independente e fixtures executáveis | GPT-6.1 Sol / Médio | Após C2; pode correr em paralelo com I1, em `verification/` | Esperados vêm do contrato e dos exemplos congelados; divergências não são corrigidas copiando a produção |
| D0-I1 | Implementação Rust e CLI conforme contrato | GPT-6.1 Sol / Alto | Após C2; fonte de conteúdo antes de seus consumidores; paralela somente a V1 | Fluxo público funcional, revisão de bytes verificável, compatibilidade e erros conforme contrato |
| D0-R1 | Integração, revisão e aceite | GPT-6.1 Sol / Alto; Astra / Alto se restar questão crítica | Depois de V1 e I1 | Regressão pertinente, fluxo ponta a ponta, recursos, fonte/build e limitações registrados |

A decomposição interna de I1 está fechada na sua OS: fonte/recursos → sensor/relatório → CLI, sequencialmente na mesma conversa de implementação. P1–P4 já foram aprovadas; pacotes externos explícitos são o contrato desta entrega. Não há bloqueio de produto remanescente. R1 recebe o binário estável e as provas reais produzidas por I1, sem repetir indiscriminadamente a preparação documental.

Depois de R1, recuperação direta, manutenção e outras capacidades exigirão seus próprios escopos e prompts. Esta sequência não autoriza construir todos os módulos de uma vez.

## Prompt pronto — D0-C1

Título sugerido da conversa: **Compass — Contrato do sensor D0**.

Modelo recomendado: **GPT-6.1 Sol**, esforço **Alto**. A tarefa é planejamento; não requer agentes auxiliares.

Copie o bloco abaixo para uma nova conversa no projeto:

```text
Use a skill codex-arquiteto para preparar o contrato do sensor D0 do Compass, com base na Foundation existente. Esta tarefa termina na proposta verificável; a implementação terá uma tarefa própria.

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Leia primeiro AGENTS.md e COMPASS_STATE.md. Consulte research/contracts/FOUNDATION.md, sobretudo as seções 5–8, e apenas as partes pertinentes de ../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md: capítulos 7, 8, seção 9.2 e seções 17.1–17.4. O documento original é preservado. A fórmula do legado em 9.2 é uma referência histórica, não uma escolha automaticamente aprovada para o novo motor.

Inspecione seletivamente model::Snapshot e Snapshot::validate em crates/compass-core/src/model.rs; storage::snapshot e load_snapshot em storage.rs; compare::compare em compare.rs; e a interface em crates/compass-cli/src/main.rs. Amplie a leitura somente quando uma dependência exigir.

Estado já estabelecido: Rust é a produção, Python é a verificação independente; Linux é o alvo inicial. Existem init, snapshot e compare. Snapshots identificam arquivos por caminhos nativos e SHA-256, representam uma janela de observação, distinguem parcialidade e não guardam conteúdos antigos. Hashes sozinhos não permitem reconstruir esses conteúdos.

Produza research/proposals/D0_V0.1.md, criando a pasta se necessário, com uma proposta recomendada e justificativa. Resolva ou identifique explicitamente:
1. Fonte verificável dos bytes antigos e atuais, correspondência aos hashes e comportamento quando os bytes faltarem ou divergirem. Avalie custo e privacidade de eventual retenção, compatibilidade com snapshots existentes e política de exclusões.
2. Universo comparado, elegibilidade textual/binária, codificação, linhas, BOM, CRLF/LF, vazios, criação/remoção e renomeação. Distinga identidade de conteúdo de eventual normalização própria da métrica.
3. Divergência local, agregação, pesos, denominador zero, precisão e determinismo. Se propuser a fórmula histórica, justifique sua adoção e preserve a distinção entre proxy textual e qualidade de contexto. Especifique o significado matemático do diff e os limites de seu cálculo.
4. Resultado completo, parcial e indisponível; limites de recursos e cancelamento; versões, proveniência e formato do relatório. Indisponibilidade nunca vira zero silenciosamente, e o baseline não avança sozinho.
5. Exemplos esperados calculados independentemente, antes do algoritmo de produção: igualdade, substituição, adição, remoção, vazios, retorno A→B→A, renomeação, binário, normalização, parcialidade e conteúdo ausente/adulterado. Mostre as contas onde houver resultado numérico e a razão nos casos sem valor.
6. Plano de implementação por fatias funcionais, dependências, áreas prováveis e critérios observáveis de aceite. Reaproveite os comandos reais do projeto. Separe o que poderá ser executado em paralelo depois de congelados contrato e gabaritos.

Não altere a Foundation, os contratos ativos, os gabaritos existentes ou o documento original. Não implemente o sensor, retenção, busca, grafos, embeddings, MCP ou GUI nesta tarefa. Não importe o legado. Não use agentes auxiliares nesta etapa e não rode a suíte de produção apenas por editar uma proposta documental.

Use escolhas recomendadas e evidência do repositório para avançar. Caso falte uma decisão de produto indispensável, apresente opções concretas, recomende uma e identifique exatamente o que depende da resposta; não a trate como já aprovada.

Ao terminar, responda de forma curta com o caminho da proposta, decisões recomendadas, cálculos/conferências realizados e pendências que impeçam iniciar a implementação. Não inicie a fase seguinte nem ajuste gabaritos para combinar com uma implementação.
```

## Retorno e próximo prompt

Depois de C2, a coordenação confere o retorno, resolve decisões de produto pendentes com Emanuel e registra a ativação do contrato/gabaritos. Então fornece prompts concretos para V1 e I1, com contrato, fixtures e áreas de escrita delimitados. Os prompts seguintes serão baseados nos resultados reais; não em interfaces presumidas hoje.

## Prompt pronto — D0-C2

Título sugerido: **Compass — Revisão do contrato D0**. Modelo recomendado: **GPT-6 Astra**, esforço **Alto**. Esta é a revisão crítica prevista; Extra Alto e agentes auxiliares não são necessários como ponto de partida.

Triagem da coordenação: a proposta cobre a entrega C1 e preserva a Foundation. Recalcular o hash dos 21 inputs de fonte/contrato continuou reproduzindo o build Foundation verificado. As contagens de verificações numéricas são relatos de C1; a coordenação não repetiu a campanha matemática, reservada a C2.

```text
Use codex-arquiteto para executar exclusivamente D0-C2: revisão independente e preparação do contrato D0 para congelamento. A entrega anterior é uma proposta, não uma especificação já aprovada.

Diretório:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Leia AGENTS.md, COMPASS_STATE.md e research/proposals/D0_V0.1.md. Consulte somente as partes relevantes do contrato Foundation, os símbolos Rust citados na seção 2 da proposta e o documento principal preservado (capítulo 7, seção 9.2 e capítulo 17). Não repita a auditoria geral da Foundation.

A C1 recomenda pacotes externos de bytes vinculados aos manifestos, UTF-8 estrito sem NUL, normalização de BOM/CRLF/LF terminal, LCS exata, massa bilateral n0+nt, razões inteiras, indisponibilidade explícita e limites estáticos. Essas escolhas precisam ser examinadas, não apenas confirmadas. Não use a saída de futura produção para criar esperados.

Concentre a revisão em:
1. Matemática e significado: recalcular exemplos representativos e casos de fronteira da seção 5 por procedimento independente. Conferir LCS, massa incluindo unchanged, zeros, arredondamento, simetria e A→B→A. Comparar a proposta de pesos com a referência histórica e expor a consequência de produto, sem alegar superioridade científica.
2. Disponibilidade e universo: provar por casos a distinção entre zero, sem textos, unknown, parcialidade e transição de escopo; verificar prioridades quando coexistem motivos. Diferenciar simetria matemática completa de disponibilidade/subconjuntos sob orçamentos e leitura baseline antes de current. Definir o tratamento de todas as exclusões e candidatos sem inventar arquivos em regiões desconhecidas.
3. Fonte de bytes e binding: especificar descritor, bytes canônicos versus aceitação de outras serializações, hashes dos inputs, referências, pacotes ausentes/ilegíveis, objetos adulterados e fontes compartilhadas. Conferir que o hash é dos mesmos bytes usados no cálculo, que exclusões não são percorridas e que links/FIFO/substituições não escapam das raízes. Descrever uma preparação reproduzível de pacote sem pressupor ferramenta de captura inexistente.
4. Recursos: fechar limites e regras de reserva antes de I1. A proposta ainda remete a contabilização de memória ao desenho futuro. Resolva capacidades/overhead ou delimite honestamente uma política verificável, sem alegar limite de RSS. Resolva o saldo insuficiente de células e a precedência dos limites. Examine o caso de arquivos iguais cuja reserva n0*nt excede o teto: manter ou alterar a regra exige justificativa e esperado explícito, sem fazer a disponibilidade depender de otimização oculta.
5. Interface: definir como configurar limites/perfil reduzido e eventual prazo na CLI, ainda ausentes da assinatura proposta. Fechar todos os campos, enums, ausências/null, ordenações, domínios numéricos, códigos de saída, hashing da metric_policy e validações cruzadas. Preservar os três comandos e os contratos Foundation.
6. Aceite: conferir E01–E35, corrigindo somente com justificativa registrada, e acrescentar casos de fronteira necessários. Distinguir schema de transporte de invariantes semânticas que exigem validação no núcleo. Definir quais artefatos V1 e I1 receberão antes de começarem em paralelo.

Entregue:
- research/proposals/D0_C2_REVISAO.md: achados com evidência/contraexemplo, gravidade, decisão e resolução; decisões técnicas resolvidas; escolhas de produto ainda necessárias; parecer pronto_para_ativacao ou pendente. Um parecer favorável deve explicar por que os pontos críticos estão fechados.
- research/proposals/d0-c2/: contrato consolidado candidato, schemas e fixtures pequenas suficientes para congelar o transporte e os exemplos-base. Defina nomes e registre um índice com hashes dos artefatos revisados. Não os apresente como contratos ativos nesta etapa. Preserve a proposta C1 como histórico e registre as mudanças no consolidado.
- Evidência reproduzível das contas/conferências documentais, com inputs literais e resultados esperados independentes. Não é necessário construir o laboratório completo V1. Valide schemas/fixtures com ferramenta disponível apropriada; se alguma validação não puder ser executada, registre a limitação, sem afirmar sucesso.
- Registro curto do progresso em COMPASS_STATE.md e docs/COORDENACAO.md, mantendo V1/I1 pendentes da liberação da coordenação.

Resolva escolhas técnicas rotineiras com base nas evidências. Para decisões que mudam o significado do sensor ou o trabalho exigido do operador, apresente uma recomendação concreta e consequências; não atribua aprovação a Emanuel. A coordenação fará a liberação final com os artefatos revisáveis prontos.

Não implemente o sensor, adaptador de conteúdo, retenção ou CLI; não altere research/contracts/, gabaritos Foundation, código de produção ou documento original. Não rode a suíte de produção por uma mudança documental. Não use agentes auxiliares nesta revisão. Leia seletivamente, mantenha saídas de ferramentas curtas e evite ciclos de investigação repetidos.

Ao terminar, informe o parecer, caminhos dos artefatos, achados resolvidos, verificações realmente executadas e decisões que ainda impeçam a ativação. Não inicie V1/I1 e não envie mensagens para outros chats; o resultado será conferido pela coordenação.
```

## Retorno D0-C2 — 14/10/2026

Entregues [parecer](../research/proposals/D0_C2_REVISAO.md), [contrato/schema/fixtures candidatos e índice SHA-256](../research/proposals/d0-c2/INDEX.md) e [conferências](../research/proposals/d0-c2/evidence/conferencias.json), usando codex-arquiteto e sem agentes/implementação. Resolvidos binding, universo/prioridades, transporte/CLI candidata, ledger lógico de memória e reservas determinísticas. E30 mudou com justificativa de EOF; contas restantes C1 conferidas. Evidência inclui 39 LCS literais, 21 agregações, 961 pares pequenos, 5 schemas e 10 relatórios completos candidatos, sem suíte de produção. Código/contratos/gabaritos Foundation, C1 e documento original preservados.

**Ação da coordenação:** conferir o conjunto pelo índice; resolver P1 pacotes/custo do operador, P2 peso linear vs log, P3 elegibilidade/normalização/zeros e P4 rigor/evidência/disponibilidade. Somente depois registrar ativação dos hashes/versões e fornecer prompts V1/I1 com áreas delimitadas. Parecer pendente não libera consumidores. Nenhuma mensagem foi enviada a outros chats; não há novo prompt de implementação nesta entrega.

## Conferência da coordenação e ajuste D0-C2-A1 — 14/10/2026

A coordenação leu o parecer e o contrato candidato, conferiu as 80 entradas de `SHA256SUMS`, repetiu o checker documental C2 com dependências locais e conferiu os 34 arquivos do inventário de preservação. As conferências passaram, incluindo 39 contas LCS, 21 agregações, 961 pares pequenos, 5 schemas, 10 relatórios e a identidade dos 21 inputs do build Foundation. Isso reproduz a evidência documental; não é uma nova validação da implementação ou da segurança operacional. Nenhuma suíte de produção foi executada.

**Parecer da coordenação:** a entrega C2 é consistente com o escopo solicitado, mas recomendo ajustar T09/E36 antes do congelamento. A indisponibilidade de textos iguais de 4.000 linhas é uma consequência deliberada do candidato, não erro nas contas: 16 milhões de células excedem o teto de 10 milhões. Entretanto, após verificação das fontes e comparação exata das sequências normalizadas, `LCS(X,X)=|X|` determina o resultado sem trabalho quadrático. Transformar esse caso em regra explícita de admissão mantém o determinismo e o rigor da evidência. A regra atual também consome o saldo de células com arquivos inalterados, prejudicando a disponibilidade dos demais.

**Recomendações de produto, ainda não aprovadas por Emanuel:** P1 pacotes externos para esta primeira entrega técnica, explicitando que o preparo manual não resolve a experiência final de captura; P2 massa bilateral linear como sensor textual identificado/versionado, sem alegação de superioridade preditiva; P3 normalização e zeros explícitos conforme o candidato; P4 verificação rigorosa de bytes, integral ausente sob bloqueio e subconjunto separado. A revisão da reserva de células não dispensa nenhuma evidência de fonte. O documento principal §9.2 trata pesos logarítmicos como históricos, portanto P2 é compatível com a direção inicial, mas precisa de decisão explícita por alterar os valores.

**Próxima ação:** ajuste documental delimitado na mesma conversa C2. Recomendação de execução: GPT-6.1 Sol / Alto, sem agentes auxiliares. Não repetir a revisão geral. V1/I1 permanecem pendentes do ajuste, das decisões de produto e da ativação registrada pela coordenação.

### Prompt pronto — D0-C2-A1

```text
Continue a entrega D0-C2 nesta conversa, usando codex-arquiteto. Execute somente D0-C2-A1, ajuste documental da regra de igualdade, no diretório:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Leia AGENTS.md, COMPASS_STATE.md e a seção “Conferência da coordenação e ajuste D0-C2-A1” em docs/COORDENACAO.md. Consulte seletivamente research/proposals/D0_C2_REVISAO.md e research/proposals/d0-c2/CONTRATO_CANDIDATO.md, schemas, política, fixtures e checker afetados. A coordenação já reproduziu com sucesso a evidência C2 e a preservação da Foundation; não refaça a auditoria geral.

Problema: T09/E36 reserva n0*nt inclusive para textos comprovadamente iguais. Assim, dois textos iguais com 4.000 linhas ficam indisponíveis e arquivos unchanged gastam o saldo dos demais. Isso é uma escolha explícita do candidato atual; a coordenação recomenda substituí-la antes do congelamento.

1. Especifique uma regra obrigatória e determinística: depois das verificações de fonte, classificação textual, normalização, limites de linhas e admissão de memória, compare exatamente as sequências de linhas. Se forem iguais, use l=n0=nt, a=r=c=0 e reserve zero células de diff. Igualdade somente de hashes declarados, tamanho ou contagem de linhas não basta. Se forem diferentes, mantenha a reserva n0*nt, os tetos e as regras de débito existentes. Preserve os demais limites, a verificação dos vazios, cancelamento, a precedência dos bloqueios e o ledger conservador de memória; não amplie esta tarefa para otimizações gerais de LCS ou redesenho de memória.
2. Justifique a exatidão por LCS(X,X)=|X|. Registre a alteração de disponibilidade como política explícita, não como otimização facultativa da implementação. Atualize o identificador da regra cell_budget, sua definição, schemas pertinentes, metric_policy_id e os artefatos derivados afetados.
3. Reconcilie E36 e acrescente ou confira casos de igualdade acima do teto de células, igualdade com saldo de células zero, igualdade após normalização mas com bytes diferentes, fontes ausentes/adulteradas que continuam bloqueando, e sequências diferentes com a mesma contagem que continuam sujeitas aos tetos. Verifique também que uma unidade igual não consome células das seguintes. Preserve massa/denominador/zero_basis e os resultados matemáticos já corretos; toda mudança de esperado deve ter justificativa contratual.
4. Atualize candidato, parecer, índice, evidências e SHA256SUMS de modo consistente. Reexecute o checker documental e as validações afetadas; confirme os hashes finais e a preservação dos arquivos protegidos. Não crie esperados com código de produção nem anuncie validação de filesystem/performance que não foi executada.
5. Registre o retorno em COMPASS_STATE.md e docs/COORDENACAO.md. P1–P4 continuam recomendações pendentes da decisão de Emanuel; este prompt não aprova produto nem ativa contrato. Não altere research/contracts/, produção, gabaritos Foundation, proposta C1 ou documento original. Não inicie V1/I1, não rode a suíte de produção e não use agentes auxiliares.

Ao concluir, informe mudanças, conferências realizadas, hash da política atualizado e pendências para ativação. Não envie mensagens a outros chats.
```

## Retorno D0-C2-A1 — 18/10/2026

Executado exclusivamente o ajuste documental: [contrato candidato](../research/proposals/d0-c2/CONTRATO_CANDIDATO.md), [parecer](../research/proposals/D0_C2_REVISAO.md), [fixtures e schemas](../research/proposals/d0-c2/INDEX.md) e [resultado da ordem](../research/proposals/d0-c2/RESULTADO-D0-C2-A1.md) foram atualizados. A política obrigatória passou a comparar sequências normalizadas exatas após todas as admissões anteriores: iguais reservam 0 células; diferentes mantêm `n0*nt`. O novo `metric_policy_id` é `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

E36 agora cobre igualdade 4.000×4.000 acima do teto; E51–E55 cobrem saldo zero, CRLF/LF, fontes inválidas, mesma contagem diferente e saldo da unidade seguinte. O checker documental passou com 55 casos, 44 cálculos LCS, 23 agregações, 961 pares pequenos, 5 schemas e 10 relatórios/inputs; `SHA256SUMS` foi regenerado após as verificações. Nenhuma suíte de produção, V1/I1, retenção, CLI ou contrato Foundation foi iniciada/alterada. P1–P4 e a ativação pela coordenação seguem pendentes.

## Encerramento auditado e próxima entrega — 18/10/2026

Emanuel solicitou declarar A1 concluída e gerar o próximo contrato conforme o plano. **D0-C2-A1 está CONCLUÍDA** após parecer `CONFORME`; T09/E36 não exige outra rodada de ajuste. A auditoria permanece preservada como registro da versão conferida.

Próxima ordem: [OS-D0-V1](entregas/d0/OS-D0-V1.md), com recomendação GPT-6.1 Sol / Médio. O [PLANO-ENTREGAS.md](entregas/d0/PLANO-ENTREGAS.md) concentra a sequência e o isolamento entre V1/I1; esta coordenação mantém o histórico. A OS tem escopo fechado, AC-01–AC-08, comandos a entregar e separação entre self-test e prova do binário real. O executor escreve somente em `verification/d0/` e devolve seu resultado à coordenação.

P1–P4 continuam sem decisão registrada. A arquitetura prepara a ordem, sem atribuir aceite de produto ao pedido de gerar documentos. Após decisão de Emanuel, a coordenação registrará `entregas/d0/ATIVACAO.md` com os hashes autorizados; então V1 poderá executar e I1 receberá sua própria OS. O prompt de execução está ao final da OS-D0-V1 e só deve ser usado após essa liberação. Não foi iniciada implementação, novo laboratório ou execução em outro chat.

Verificação desta arquitetura: 31 links locais e formatação documental conferidos; 81 hashes candidatos e 34 arquivos protegidos preservados; apenas estado/coordenação alterados e os dois documentos de entrega criados. Uso da conta no início/fim: 8%→9% na janela compartilhada de 10.080 minutos (+1 ponto percentual arredondado, não contagem de tokens desta tarefa). Nenhuma suíte de produção foi necessária ou executada.

## Ativação e desbloqueio de V1 — 19/10/2026

Após o implementador reportar a ausência de `ATIVACAO.md`, a coordenação apresentou novamente a decisão conjunta P1–P4. Emanuel respondeu **“Sim, aprovo”**. Essa aprovação foi registrada em [ATIVACAO.md](entregas/d0/ATIVACAO.md), identificando versões e hashes; P1–P4 estão decididas para a primeira entrega técnica.

**OS-D0-V1 versão 2 LIBERADA.** Alterados somente seu estado e referência de liberação, preservando escopo, AC-01–AC-08 e comandos. O bloqueio anterior está resolvido; o implementador deve retomar na mesma conversa com executor-blindado, conferir a ativação e executar a OS. Não é necessário repetir planejamento ou solicitar novamente P1–P4.

O conjunto auditado mantém o hash do índice `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` e a política `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`. A coordenação conferiu os 81 hashes e os 34 arquivos protegidos, sem alteração de candidatos, auditorias ou produção. I1 receberá sua própria OS antes de executar; R1 fará a integração.

## Auditoria V1 e ordens corretivas — 19/10/2026

O parecer [AUDITORIA-OS-D0-V1](../research/audits/AUDITORIA-OS-D0-V1.md) é **NAO_CONFORME**, com BUG-001/002/003 confirmados. Self-test e mutações passaram; conferências independentes expuseram dois defeitos do validador e uma divergência de rastreabilidade E52. Não há alteração de produto ou do contrato necessária.

A skill `arquiteto-de-contratos` preparou [OS-D0-V1-C01](entregas/d0/OS-D0-V1-C01.md), [C02](entregas/d0/OS-D0-V1-C02.md) e [C03](entregas/d0/OS-D0-V1-C03.md). Usar executor-blindado na mesma conversa do implementador, GPT-6.1 Sol / Médio, em sequência e sem escritores paralelos. C02/C03 preservam a baseline acumulada pelas correções anteriores. Escopos, aceites, regressões e retorno estão fechados nas ordens e no plano; estado dos BUGs permanece confirmado até execução e auditoria posterior.

Após a sequência, realizar auditoria seletiva dos mesmos IDs/ACs e das regressões afetadas. A revisão não reabre a Foundation nem as decisões P1–P4. Este trabalho alterou somente planejamento/estado; não implementou correções, não modificou a OS original v2 ou o parecer e não iniciou I1/R1.

Conferência da arquitetura corretiva: 60 links locais e formatação documental validados; 81 artefatos congelados e 34 arquivos protegidos preservados. Criadas somente as três OS; plano, estado e coordenação atualizados. A implementação V1 permaneceu idêntica à versão auditada; testes funcionais serão executados pelo corretor. Consulta inicial de uso: 11% na janela compartilhada de 10.080 minutos; a consulta final não retornou e foi interrompida, portanto não há diferença de consumo mensurável nesta etapa.

## V1 aceita e próxima execução — 19/10/2026

O parecer [C01–C03 CONFORME](../research/audits/AUDITORIA-OS-D0-V1-C01-C03.md), SHA-256 `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1`, encerra os três BUGs e permite aceitar V1. As 132 verificações, 119 cenários preparados e 11 mutações foram reproduzidos na auditoria. O sensor Rust ainda não existe; não confundir aceite do laboratório com homologação de D0.

Próxima conversa de implementação: **Compass — D0-I1 Rust e CLI**. Modelo recomendado: **GPT-6.1 Sol / Alto**, usando `executor-blindado`; um escritor, sem subagentes de implementação nesta entrega. A OS prevê testes internos sincronizados e campanha pública com V1 intacta; uma auditoria independente posterior confere o resultado. Não é necessário usar Astra Extra Alto para iniciar.

Auditoria prévia do contrato: **CONFORME**, sem achados obrigatórios. A coordenação libera a execução da OS; não há aprovação adicional pendente para começar. Nenhuma implementação foi realizada durante esta preparação.

### Prompt pronto — D0-I1

Prompt histórico da implementação já entregue. Para corrigir BUG-004, usar a ordem e o prompt I1-C01 ao final deste documento; não reiniciar I1 pelo prompt original.

```text
Use a skill executor-blindado, em /home/many/.agents/skills/executor-blindado/SKILL.md.

Execute exclusivamente /home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/docs/entregas/d0/OS-D0-I1.md.
Diretório de trabalho: /home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation.

Leia AGENTS.md, COMPASS_STATE.md, a OS, ATIVACAO.md, a auditoria de contrato da I1 e as referências delimitadas. P1–P4 estão aprovadas; V1 está aceita após C01–C03. O repositório não possui HEAD: registre Git, alterações existentes e hashes antes de escrever.

Implemente I1-a → I1-b → I1-c sequencialmente, com um único escritor. Preserve contratos, esperados e verification/d0/. Prepare a baseline histórica antes das edições Rust/README; execute a V1 intacta contra o binário I1 real conforme o adaptador previsto na OS. Respeite a lista fechada de escrita e os aceites I1-AC-01–11; não ajuste gabaritos para acomodar a produção.

Execute as provas comportamentais, corridas sincronizadas, regressões e comandos previstos. Registre falhas/restrições sem convertê-las em aprovação. Não inicie R1 nem envie mensagens a outros chats.

Entregue verification/d0-i1/RESULTADO-D0-I1.md com AC→evidência, arquivos/deltas, comandos realmente executados, hashes do build/binário, resultados do laboratório e pendências para a auditoria. Ao terminar, traga um resumo à coordenação.
```

Conferência desta preparação: 61 links locais, whitespace, 81 artefatos congelados, 34 protegidos e V1 preservados; 21 inputs reproduzem o build Foundation. A auditoria confirmou a liberação sem mudança normativa: OS final SHA-256 `25283d74828bd65fc5bebc350122835759dc1d2ac7f2c338362c655dff831eb4`; parecer SHA-256 `4005b0bb5c3b6d3a1f7adee6fc6ce034bd2468d76a8977b6ff9a7a40cbf485df`. Nenhum teste/build de produção executado nesta arquitetura. Uso compartilhado consultado: 14% → 15% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta entrega.

## Correção BUG-004 — 19/10/2026

A [auditoria I1](../research/audits/AUDITORIA-OS-D0-I1.md), SHA-256 `4a9000d6e75c9d3e30634113c918ea32db7e9e6a795c77630c170e932f975910`, confirmou que a leitura compartilhada pode ignorar a troca de componente na referência current. A campanha e os testes existentes passaram, mas a prova sincronizada no binário real produziu complete/ambos verified indevidamente. I1-AC-03/04/07 ficaram NAO_CONFORME.

A [OS-D0-I1-C01](entregas/d0/OS-D0-I1-C01.md) delimita correção e testes, sem alterar produto, V1 ou baseline. Recomendação: **mesma conversa do implementador I1, GPT-6.1 Sol / Alto**, usando executor-blindado, um escritor e auditoria focada posterior. HIP-I1-01 não é obrigação de implementação nesta ordem. Não há nova decisão de produto pendente.

A [auditoria prévia da OS corretiva](../research/audits/AUDITORIA-CONTRATO-OS-D0-I1-C01.md) foi CONFORME, sem achados obrigatórios. A coordenação libera a execução, preservando escopo e aceites auditados; a única alteração administrativa da OS foi seu campo `estado`. BUG-004 permanece confirmado até a conferência posterior da implementação.

### Prompt pronto — D0-I1-C01

Prompt histórico da correção já implementada e verificada. Para avançar, usar o prompt R1 ao final deste documento; não reexecutar C01 como entrega nova.

```text
Use a skill executor-blindado:
/home/many/.agents/skills/executor-blindado/SKILL.md

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Execute exclusivamente docs/entregas/d0/OS-D0-I1-C01.md, correção do BUG-004 confirmado na auditoria de I1. Continue sobre a implementação existente; não reinicie a OS-I1 original.

Leia AGENTS.md, COMPASS_STATE.md, a OS corretiva, sua auditoria prévia de contrato e as referências delimitadas. Inspecione Git, diff e hashes antes de escrever; o repositório não possui HEAD.

Acrescente regressão permanente de operação completa, com barreira no cenário da referência current substituída durante uma leitura compartilhada. Demonstre a falha específica antes de mudar a lógica e o sucesso depois do patch. Preserve controles estáveis, leitura única, reservas, fatos EOF, fonte ausente/adulterada e detecções existentes. Erro genérico ou timeout não é rejeição correta.

Respeite a lista fechada de escrita e C01-AC-01–06. Preserve V1, contratos/gabaritos, baseline JSON/tar e OS-I1 original. HIP-I1-01 fica fora do escopo; não reabra BUG-001/002/003 nem inicie R1. Não envie mensagens a outros chats.

Execute os comandos e regressões previstos, renove build/proveniência e rode a V1 intacta contra o binário corrigido estável. Registre permissões ou falhas sem convertê-las em aprovação.

Acrescente a seção OS-D0-I1-C01 / BUG-004 em verification/d0-i1/RESULTADO-D0-I1.md, preservando o histórico. Traga baseline/deltas, prova antes/depois, AC→evidência, comandos realmente executados, hashes do build/binário/JSONs e pendências. Termine como IMPLEMENTADO_PENDENTE_VERIFICACAO; o fechamento do BUG depende da conferência independente posterior.
```

Conferência da preparação C01: OS liberada SHA-256 `c5a47f6cb4b76ec3b9311f4c114e28bc06b97ef8fcf5d1e9e0c17b30a0f66866`; parecer de contrato SHA-256 `c32b9cdf635ee134ad4f94effe2459e0e899f8dbddfebe5eb2bbb566b5b3ff4a`. Confirmado o hash normativo auditado, alterando somente o estado da OS para liberar. Links/whitespace e 81 artefatos foram conferidos; código, V1, baseline e auditoria de entrega ficaram intactos. Nenhuma correção ou suíte de produção executada nesta arquitetura. Uso compartilhado consultado: 16% → 17% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta entrega.

## C01 aceita e R1 preparada — 19/10/2026

A [auditoria C01](../research/audits/AUDITORIA-OS-D0-I1-C01.md), SHA-256 `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3`, é CONFORME. A coordenação registra C01 verificada/encerrada e I1 aceita para integração; os achados anteriores não bloqueiam R1. O binário corrigido é `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`, com produtor `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`.

Próxima conversa: **Compass — D0-R1 integração e recursos**. Modelo recomendado: **GPT-6.1 Sol / Alto**, executor-blindado, um escritor. A [OS-R1](entregas/d0/OS-D0-R1.md) autoriza nove arquivos novos e protege produção/contratos/evidências anteriores. O [contrato operacional](entregas/d0/CONTRATO-ACEITE-D0-R1.md) fecha cargas, esperados, seis amostras RSS e matriz de provas herdadas/novas. Porte padrão, somente laboratório sintético; conferência independente após retorno.

### Prompt pronto — D0-R1

Prompt histórico da entrega executada e aceita após conferência independente. Não iniciar outra R1 por este bloco.

```text
Use a skill executor-blindado:
/home/many/.agents/skills/executor-blindado/SKILL.md

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Execute exclusivamente docs/entregas/d0/OS-D0-R1.md e seu CONTRATO-ACEITE-D0-R1.md. Leia AGENTS.md, COMPASS_STATE.md e as referências delimitadas. V1 e I1/C01 estão aceitas para integrar; BUG-004 foi verificado. Não existe HEAD: inspecione Git, diff e hashes antes de escrever.

Implemente o runner e as nove saídas autorizadas, sequencialmente. Fixe fixtures/esperados antes do sensor. Execute os dois fluxos pela CLI real com snapshots e pacotes externos sintéticos, retirada da árvore viva, controles negativos e três amostras D0 de tempo/CPU/RSS por carga. Confira R1-AC-01–07; não confunda RSS com ledger nem imponha SLO não aprovado.

Use o binário/37 inputs auditados, conferindo sua estabilidade. Preserve Rust, CLI, Cargo, contratos, V1, I1, baselines e evidências anteriores. Herdar prova verificada é permitido; identifique-a como herdada, sem alegar reexecução. Consolide E01–E55 e as provas internas/BUG-004 na matriz R1.

Não corrija produção, não implemente HIP-I1-01 ou próximas capacidades, não acesse acervos reais/rede e não envie mensagens a outros chats. Registre falhas, permissões ou medições ausentes sem convertê-las em aprovação.

Entregue verification/d0-r1/RESULTADO-D0-R1.md com AC→evidência, comandos reais versus provas herdadas, identidades completas, métricas e limites. Termine como IMPLEMENTADO_PENDENTE_VERIFICACAO; o aceite final depende da conferência posterior da coordenação.
```

Conferência desta preparação: 79 links locais e whitespace validados; 81 artefatos congelados intactos; 37 inputs reproduzem o produtor C01 e o binário mantém o SHA aprovado. Do inventário inicial de 160 arquivos, somente estado/plano/coordenação mudaram e duas peças R1 foram acrescentadas; os outros 157 arquivos permaneceram idênticos. OS liberada SHA-256 `6be86427ba5d3e0473361c4ccb1b0ad04fec87080b8c75c8ffbccc31364aa635`; contrato SHA-256 `10f6463b91269147d59752ec8799469341d743c60f0d15158879ad840205afa1`. Contas/receitas documentais conferidas, sem execução de R1, suíte de produção ou medição RSS nesta arquitetura. Uso compartilhado consultado: 18% → 18% na janela de 10.080 minutos; sem diferença visível na resolução apresentada, sem inferir consumo zero ou contagem exata de tokens desta entrega.

## R1 aceita e entrega técnica D0 encerrada — 19/10/2026

A [auditoria R1](../research/audits/AUDITORIA-OS-D0-R1.md), SHA-256 `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9`, concluiu **CONFORME nos sete aceites**, sem achado obrigatório. Reproduziu M/L, 24 comandos públicos, quatro negativos/restaurações e seis medições nativas em cópia; conferiu os bytes/matemática/cobertura por ferramenta separada e quatro adulterações de relatório por motivos específicos. Provas I1/C01 foram herdadas na mesma identidade, sem alegar nova execução de suas suites.

A coordenação aceita R1 e encerra a entrega técnica D0 contratada. Os nove artefatos entregues e sua proveniência histórica não foram regravados; código, binário, contratos e esperados permanecem intactos. O registro posterior de aceite altera somente estado, este documento e o plano. Próxima capacidade será delimitada a partir do plano de produto, com contrato e prompt próprios; nenhuma implementação seguinte ou mensagem a outro chat foi iniciada. Uso compartilhado 19% → 20%, aproximação de +1 ponto percentual, sem atribuição exata de tokens.

## Próxima execução — recuperação direta literal RD-I1

Documento principal §17.3: depois do sensor D0 vem busca e entrega de trechos da revisão declarada. A primeira fatia é [RD-I1](entregas/rd/OS-RD-I1.md), com [contrato](entregas/rd/CONTRATO-RD-V0.1.md) e [plano breve](entregas/rd/PLANO-ENTREGAS.md). A CLI `compass search` buscará literal em um snapshot/pacote, entregando linhas e offsets verificados; scan parcial, seleção limitada e zero hits terão significados distintos. Sem índice, retenção, ranking, grafo, embedding ou MCP nesta OS.

Porte crítico pelo conteúdo e fronteira de leitura. [Auditoria prévia CONFORME](../research/audits/AUDITORIA-CONTRATO-OS-RD-I1.md), SHA-256 `339ce58e2e1518d722ebe7e0846381d161d5432adea96d1ead52663fbe4ab4f5`; BUG-RD-C-001 foi corrigido e verificado no contrato antes da liberação. Não há achado obrigatório ou decisão de produto pendente nesse escopo. Contrato final SHA-256 `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72`; alteração administrativa da OS apenas para LIBERADA_PARA_EXECUCAO.

Nova conversa: **Compass — RD-I1 busca literal verificável**. Modelo recomendado: **GPT-6.1 Sol / Alto**, executor-blindado; um escritor, três fases sequenciais na mesma conversa. Auditoria independente depois da entrega. A produção precisa de nova identidade/release e regressão Foundation/D0, mantendo o laboratório histórico e os resultados antigos intactos. Não repetir R1 fixada ao binário antigo como se aceitasse automaticamente o novo.

### Prompt pronto — RD-I1

Prompt histórico da implementação já entregue e aceita após C01; não reiniciar a OS original. O prompt corrente RD-I2 está ao final deste documento.

```text
Use a skill executor-blindado:
/home/many/.agents/skills/executor-blindado/SKILL.md

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Execute exclusivamente docs/entregas/rd/OS-RD-I1.md e CONTRATO-RD-V0.1.md, versão 1. Leia AGENTS.md, COMPASS_STATE.md, o parecer prévio de contrato e as referências delimitadas. A OS está liberada após auditoria CONFORME; D0/R1 estão aceitas. Não existe HEAD: registre Git, diff, inventário, cópias e hashes antes de escrever.

Implemente sequencialmente: fixtures/oráculo independentes antes de Rust → núcleo e CLI search → integração, release, regressão e recursos. Respeite a lista fechada e RD-AC-01–10/RD01–RD24. Reutilize o leitor seguro e o binding existentes sem mudar D0; os novos laços RD têm checkpoints próprios, sem prometer preempção interna de hashing/codec/JCS compartilhados.

Entregue trechos com bytes/linhas/offsets da revisão declarada, cobertura e truncamento explícitos; continue a busca após atingir max_matches e não exponha texto não verificado. Não ajuste esperados ao sensor.

Execute os comandos previstos contra o release final estável. Monte V1 histórica por verify/mount em temporário para regressão do novo binário; não sobrescreva JSONs I1/V1/R1 nem remova pins históricos. Registre as três medições nativas separando RSS de créditos lógicos. Falha genérica, skip ou restrição não conta como aprovação.

Não instale dependências, acesse acervos reais/rede, implemente próximos módulos ou envie mensagens a outros chats. Entregue verification/rd/RESULTADO-RD-I1.md com AC→evidência, comandos, deltas, identidades, métricas e limitações. Termine IMPLEMENTADO_PENDENTE_VERIFICACAO para auditoria posterior.
```

Conferência da preparação RD: 78 links locais/whitespace e 81 entradas C2-A1 aprovados; os 37 inputs e o executável C01 continuam idênticos. Dos 173 arquivos iniciais inventariados, só estado/coordenação mudaram; outros 171 foram preservados. Acrescentados contrato/OS/plano RD e parecer prévio; nenhuma implementação ou suite de produção executada. OS liberada SHA-256 `dedbf49a02f8b9c774e4a3b11f1c1be80a28f79901873ae08cd1d2028b02d419`; inverter somente seu estado administrativo reproduz o SHA auditado `3e48d1e40adab74b6e4eab7fcd1022499418e94088d21d5424754a8fcb25f11f`. O plano recebeu apenas o registro de liberação depois da revisão. Uso compartilhado consultado: 20% → 20% na janela de 10.080 minutos, sem diferença visível na resolução apresentada; isso não implica consumo zero nem mede tokens exatos da entrega.

## RD-I1 auditada — correção BUG-005 liberada

[Parecer de entrega](../research/audits/AUDITORIA-OS-RD-I1.md), SHA-256 `7700e7e1f310994e2226167c177eb6aa377e3268d90f0a87463ba53d04fd964f`: **NAO_CONFORME** somente no RD-AC-07, por BUG-005 confirmado. A detecção prévia de --json conta o valor literal da query como opção quando outro argumento falha. Exits/valor do erro são corretos, mas sem flag real uma das sintaxes emite compacto sem LF em vez de legível com newline. Os 74 Rust, campanhas RD/Foundation/D0 e três medições passaram; 30 controles independentes adicionais não revelaram outro achado.

A [OS-RD-I1-C01](entregas/rd/OS-RD-I1-C01.md), SHA-256 `4ea245f6137b41e4c33ca279360e8bbb4d40ca60dd334a99f06ffcc37b44bae6`, é o contrato mínimo da correção, **LIBERADA_PARA_EXECUCAO**. Porte curto porque altera transporte de parsing, sem modificar leitura, autorização ou conteúdo; não há novo portão de produto ou auditoria prévia obrigatória de contrato. Exige prova pública antes/depois, regressão permanente e conferência posterior do mesmo BUG/AC. Escrita: dois existentes CLI/teste e cinco saídas novas em verification/rd-c01; a entrega RD original inteira fica preservada.

Recomendação: **mesma conversa do implementador RD, GPT-6.1 Sol / Alto**, executor-blindado, um escritor. Nenhuma correção foi executada pela coordenação, e nenhuma próxima capacidade está liberada.

### Prompt pronto — RD-I1-C01

Prompt histórico: correção já executada, conferida e aceita. Usar RD-I2 como próxima entrega.

```text
Use a skill executor-blindado:
/home/many/.agents/skills/executor-blindado/SKILL.md

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Execute exclusivamente docs/entregas/rd/OS-RD-I1-C01.md, correção do BUG-005 confirmado na auditoria RD-I1. Continue sobre a entrega existente; não reinicie a OS original.

Leia AGENTS.md, COMPASS_STATE.md, a OS corretiva, auditoria e contrato RD delimitados. Inspecione Git, diff e hashes antes de escrever; não existe HEAD. Preserve o binário anterior para a prova antes/depois.

Acrescente regressão permanente e controles públicos que distingam --json como valor literal de query de uma flag real, nas duas sintaxes de query. Mostre a falha específica antes do patch; depois, sem flag deve haver JSON legível com newline, e com flag real, JCS compacto sem newline. Preserve erro invalid_arguments/exit 2, stderr vazio e queries hifenizadas. Não proíba a query nem use '=' como workaround.

Respeite os dois deltas existentes/cinco arquivos novos e C01-AC-01–06. Núcleo, contratos, fixtures e verification/rd inteira são protegidos. Execute campanhas históricas somente em cópia temporária; incorpore resultados novos em verification/rd-c01, sem sobrescrever evidências antigas.

Execute testes, fmt, Clippy, release e regressões previstos sobre a identidade final. Registre restrições sem convertê-las em aprovação. Não implemente próxima capacidade nem envie mensagens a outros chats.

Entregue verification/rd-c01/RESULTADO-RD-I1-C01.md com AC→evidência, antes/depois, deltas, comandos, hashes, métricas e limitações. Termine IMPLEMENTADO_PENDENTE_VERIFICACAO; BUG-005 só fecha após conferência independente.
```

Uso compartilhado consultado nesta auditoria/preparação: 21% → 22% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta entrega. A auditoria preservou os 188 arquivos inventariados; depois do parecer, a arquitetura acrescentou somente a OS corretiva e atualizou estado/plano/coordenação. Fonte, release, contrato e evidências do executor permaneceram intactos.

## RD-I1-C01 aceita e próxima execução RD-I2 — 19/10/2026

[Auditoria corretiva CONFORME](../research/audits/AUDITORIA-OS-RD-I1-C01.md), SHA-256 `1dd71986c3e533b34c2fc739d66322b858dfffaba917668afd6c37a6e9f6d9f8`: C01-AC-01–06 atendidos e BUG-005/RD-AC-07 encerrados. Reproduzidos antes 47/48 por formato, depois 48/48; regressão permanente falha no fonte antigo e passa no atual. Os 76 Rust, fmt/Clippy, RD151/cinco mutações/três medições e Foundation23/três/V1 253/11 passaram sem skips finais. Núcleo, contrato e evidências históricas preservados; dois deltas autorizados e 187 arquivos preexistentes intactos.

Baseline atual aceita: binário `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8`, produtor dos 41 inputs `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`. Não atribuir provas da versão antiga ao próximo build.

Próxima conversa: **Compass — RD-I2 leitura verificável de contexto**. Modelo recomendado: **GPT-6.1 Sol / Alto**, skill executor-blindado; um escritor e três fases sequenciais na mesma conversa. A [OS-RD-I2](entregas/rd/OS-RD-I2.md) e o [contrato de leitura](entregas/rd/CONTRATO-RD-LEITURA-V0.1.md) especificam seleção por path_key/faixa, texto/offsets brutos, EOF e continuação depois de verificar o objeto completo. É a continuação do fluxo search→evidência do documento principal §§6.3/17.3; nenhuma escolha adicional de produto/banco/modelo é necessária para esta fatia.

Porte crítico por seleção/entrega de conteúdo. [Auditoria de contrato CONFORME](../research/audits/AUDITORIA-CONTRATO-OS-RD-I2.md), SHA-256 `ff62c4ff8e501a67a7811e072bf1ded5e8d49a045af766099d5e869e8003816a`, sem achado obrigatório. Contrato SHA-256 `54b4826516d9552598a71cba05c09403e64022ba9ba4f1a12f936c358c0e44d6`; OS liberada `5703463f5ad37ace1c973b5b2d91ddc8b35c262b2355f3bc69dae6f92e45ba13`. Inverter somente o estado administrativo reproduz a OS auditada `e699051a55bc45d7af4be72e4aa238ac9ba15c5084f958d2fa5e82aee0e1771f`. Conferência da entrega será independente após o retorno.

### Prompt pronto — RD-I2

```text
Use a skill executor-blindado:
/home/many/.agents/skills/executor-blindado/SKILL.md

Diretório de trabalho:
/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation

Execute exclusivamente docs/entregas/rd/OS-RD-I2.md e CONTRATO-RD-LEITURA-V0.1.md, versão 1, liberados após auditoria prévia CONFORME. Leia AGENTS.md, COMPASS_STATE.md, o plano RD, o parecer prévio e as referências delimitadas na OS. RD-I1/C01 está aceita e BUG-005 verificado. Não há HEAD: inspecione Git/diff, inventário, cópias e hashes antes de escrever; preserve o release aceito.

Implemente compass read por snapshot/pacote/path_key/faixa, com seleção e continuação exatas sobre o objeto integralmente verificado. Preserve bytes/offsets, revisão declarada, binding e estabilidade, LF-only, EOF/u64, disponibilidade, limites e cancelamento; não leia outros objetos. Preserve search e a correção que distingue flag --json de valor literal.

Um escritor, sem subagentes de implementação. Três fases sequenciais na mesma conversa: (1) fixtures RL01–RL16 e oráculo independente com self-test sem produção; (2) módulo read/CLI e testes comportamentais; (3) fmt/Clippy/testes/release final, campanha real, três medidas read e regressões. Esperados vêm de bytes/fórmulas do contrato, nunca do sensor. Teste estável e intervenções sincronizadas devem provar falhas específicas; erro genérico não basta.

Respeite I2-AC-01–10 e a lista fechada de cinco existentes/onze novos. search.rs só admite as quatro alterações de visibilidade; D0, contratos, testes antigos e verification/rd/rd-c01 são protegidos. Não instale dependências nem implemente índice/manutenção/MCP.

Execute os comandos da OS sobre a identidade final. Campanhas antigas somente em cópia temporária, via rd-c01/check.py intacto contra o release novo; incorpore os JSONs completos em verification/rd-i2, sem regravar evidências antigas nem retirar pins. As três medidas read são distintas das medidas RD desse checker; não repita suites sem motivo. Socket Unix pode exigir permissão técnica: registre restrições e não declare aprovação com prova obrigatória ausente.

Entregue verification/rd-i2/RESULTADO-RD-I2.md com AC→RL/teste/evidência, comandos/resultados reais, deltas/preservação, inputs/build/binário, métricas e limitações. Informe hashes finais de resultado/proveniência e termine IMPLEMENTADO_PENDENTE_VERIFICACAO. Sem acervo real, rede, commit ou mensagens a outros chats. Auditoria posterior nesta coordenação.
```

Conferência final: 95 links locais e whitespace aprovados, 81 artefatos C2-A1 intactos, 41 inputs e executável C01 mantêm a identidade aceita. Dos 195 arquivos inventariados antes da auditoria, somente estado/plano/coordenação mudaram após o parecer; os outros 192 ficaram idênticos. Acrescentados os dois pareceres, contrato e OS de leitura. Fonte, testes e evidências anteriores preservados; nenhum build/teste/medição de produção adicional foi feito durante a arquitetura. Uso compartilhado consultado nesta auditoria/preparação: 23% → 24% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não mede tokens exatos desta entrega.
