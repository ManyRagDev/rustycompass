# Compass: situação atual e investigação do ponto de parada do agente

**Data de referência:** 26 de outubro de 2026.  
**Finalidade:** contextualizar uma investigação técnica independente, sem pressupor conhecimento do projeto.  
**Base:** inspeção do código e da instalação atuais, verificações registradas em 24/10 e relatório de uso do Multiagentes. Este documento não corresponde a uma nova execução dos testes nem altera o comportamento das aplicações.

## 1. O problema a investigar

O Compass é uma infraestrutura local para localizar e entregar evidências de um repositório a agentes de inteligência artificial. O objetivo do usuário é que uma solicitação ao Compass execute internamente a recuperação necessária e devolva um pacote de contexto em uma única resposta. A interpretação e a elaboração da resposta ao usuário ficam com a aplicação consumidora.

Em um teste real, o agente recebeu cedo a implementação necessária para responder à pergunta delimitada, mas continuou buscando e lendo arquivos. Cada nova chamada ao modelo reenviava o histórico acumulado. O consumo total cresceu muito além do custo inicial de localizar a evidência.

**A dificuldade atual não é a ausência de qualquer condição de parada. Existem limites computacionais, limites de orçamento e uma verificação de obrigações de evidência. Falta estabelecer e validar como esses mecanismos produzem um encerramento da aquisição que o agente consumidor efetivamente respeite.**

Há também uma lacuna interna: o Compass já oferece recuperação aberta com expansão pelo grafo e, separadamente, aquisição por obrigações explícitas. Esses caminhos ainda não formam um mecanismo unificado no qual a descoberta e a expansão sejam conduzidas pelas lacunas de evidência até uma decisão final de aquisição.

O teste que revelou o problema **não exercitou esse mecanismo recente de obrigações**, nem os packs e o calor. Portanto, comprova a continuação do agente no fluxo testado, mas não demonstra que o mecanismo novo falhou ou resolveu a questão.

## 2. O que são Compass, Multiagentes e evidence pack

**Compass:** programa com núcleo em Rust, acessível por linha de comando e por MCP. MCP é o protocolo pelo qual uma aplicação apresenta ferramentas a um agente e recebe os resultados das chamadas. O Compass consulta fontes previamente autorizadas e conserva a identidade da revisão consultada.

**Multiagentes:** aplicação consumidora usada nos ensaios. Seu controlador de execução, também chamado de *harness*, envia mensagens ao modelo de linguagem — a LLM —, executa as ferramentas solicitadas, acrescenta os resultados à conversa e decide quando encerrar ou interromper a execução.

**Acervo e geração:** um acervo é uma coleção preparada de arquivos. Uma geração identifica uma revisão capturada e seus índices. Isso permite consultar conteúdo consistente mesmo depois de alterações na árvore original.

**Evidence pack:** resposta estruturada contendo trechos de código, arquivos de origem, revisão, linhas, intervalos de bytes, métodos de recuperação e informações sobre limites e omissões. O conteúdo é conferido contra os objetos armazenados antes de ser entregue como válido.

**Obrigação de evidência:** requisito estruturado sobre o material que deve estar no pack, como “entregar a declaração e o corpo completos desta função”. O termo `contract` usado na API representa esse pedido técnico; não é uma OS ou um rito administrativo.

O Compass não produz a resposta final em linguagem natural. Sua utilidade depende de entregar material adequado para que o agente possa analisá-lo. Entregar bytes autênticos, encontrar conteúdo relevante e reunir evidência suficiente para uma pergunta são propriedades relacionadas, porém distintas.

## 3. O que está implementado hoje

O núcleo atual prepara e atualiza acervos com revisões preservadas; oferece busca literal, lexical persistente e semântica local; extrai símbolos e relações estáticas; permite leitura verificável, navegação e montagem de packs sob orçamento. O suporte estrutural inclui Python, Rust, JavaScript e TypeScript, incluindo JSX/TSX, com resolução conservadora e limitações explícitas.

A recuperação pode funcionar sem expansão, com expansão em largura pelo grafo — BFS — ou com difusão de calor. A semântica utiliza um modelo local E5/Candle, instalado explicitamente. O ambiente inicial é Linux/CPU. Não há garantia de compreensão completa do comportamento dinâmico do programa ou de resolução de todas as dependências.

As ferramentas MCP são `compass_search`, `compass_read`, `compass_files`, `compass_query`, `compass_neighbors` e `compass_retrieve`. **`compass_query` retorna candidatos; `compass_retrieve` monta o pack.** Preparação e atualização do acervo permanecem na CLI.

### Dois caminhos de recuperação coexistem

| Caminho atual | Como escolhe o material | O que consegue afirmar |
|---|---|---|
| `compass_retrieve` sem `contract` | Recuperação por consulta, combinação de candidatos, expansão opcional e seleção sob orçamento | Origem e revisão dos trechos, limites e diagnósticos. Não certifica que a pergunta recebeu todas as evidências necessárias. |
| `compass_retrieve` com `contract` | Resolve seletores explícitos de símbolos, ocorrências e relações; prioriza as evidências obrigatórias | Se todas as obrigações declaradas estão comprovadas pelo conteúdo do pack atual. Não certifica a interpretação da pergunta. |

No segundo caminho, o código declara `selection: mandatory_evidence_only`. Quando solicitados, os cálculos de BFS/calor fornecem diagnósticos, mas não acrescentam material para descobrir ou completar obrigações. No primeiro caminho, o calor participa da recuperação, mas não existe a avaliação de suficiência por obrigações. Essa diferença é central para a investigação. [S1–S3]

### O significado exato de `SUFFICIENT_EVIDENCE_ACQUIRED`

O consumidor declara de 1 a 16 obrigações. Os tipos atualmente suportados são:

- definição de símbolo, com declaração e corpo completos;
- ocorrência literal exata, dentro de arquivo ou símbolo delimitado, com a linha correspondente;
- relação direta, direcionada e resolvida estaticamente, acompanhada de sua evidência.

O Compass resolve esses requisitos, verifica os objetos e compara os intervalos necessários com os intervalos efetivamente presentes em `items`, no pack devolvido. O status é suficiente somente se todas as obrigações forem satisfeitas e `interpretation_pending` estiver vazio. A avaliação tem escopo `current_pack`: material entregue apenas em páginas anteriores não torna a página atual suficiente.

Isso é uma verificação determinística da entrega solicitada. Não é uma probabilidade de relevância nem uma avaliação semântica da resposta final. Um pedido que omita uma dependência importante pode ser cumprido integralmente e ainda ser insuficiente para a pergunta original. O campo textual `intent` registra a intenção, mas o Compass não a interpreta com uma LLM. [S1–S2]

### O que as verificações existentes demonstram

O registro de 24/10 informa 166 testes Rust aprovados, além de 82 controles públicos, realizados em 101 invocações CLI e 80 chamadas MCP. Houve zero falsas classificações de suficiência nos controles executados. Isso não estabelece uma taxa de erro universal.

Nos dois acervos sintéticos separados de desenvolvimento e avaliação, as evidências obrigatórias couberam em uma resposta com orçamento de 16 KiB. Os packs ocuparam 12.414 e 12.417 bytes. Com orçamento de 8.000 bytes, a cobertura medida foi de 30% e o resultado permaneceu insuficiente. O orçamento inclui metadados e codificação JSON; não é uma contagem de tokens da LLM.

Esses testes demonstram o comportamento da aquisição explícita nos casos controlados. **Não demonstram redução das rodadas do Multiagentes, nem descoberta geral das obrigações a partir de linguagem natural.** O hash do executável instalado foi conferido em 26/10 e coincide com o identificado nessa campanha. [S4]

## 4. O que aconteceu no teste real

O relatório do Multiagentes de 24/10 comparou duas execuções sobre o mesmo acervo: uma usando Compass e outra usando ferramentas nativas. A pergunta era se a função que insere transações no histórico funde entradas consecutivas com a mesma chave de coalescência.

Segundo o relatório, a implementação completa de `pushTransaction` mostrou que a função guarda a chave, acrescenta a transação e não realiza a fusão. O marco de suficiência adotado foi a chegada da função completa, dentro de uma leitura de 67 linhas. **Esse marco foi determinado retrospectivamente**, considerando o escopo da pergunta. Não foi um sinal automático emitido durante a execução.

| Medida registrada | Compass | Ferramentas nativas |
|---|---:|---:|
| Tempo até a evidência central | 5,84 s | 8,85 s |
| Chamadas à LLM até obter essa evidência | 2 | 3 |
| Primeira chamada à LLM que já poderia utilizá-la | 3ª | 4ª |
| Chamadas totais à LLM | 9 | 9 |
| Chamadas totais de ferramentas | 12 | 12 |
| Tokens totais acumulados | 152.526 | 81.042 |
| Tokens de entrada da última chamada | 30.384 | 16.875 |
| Duração da execução | 45,2 s | 36,9 s |

Ambos os caminhos responderam corretamente segundo a conferência registrada. As consultas posteriores ampliaram a cobertura, mas não alteraram a conclusão sobre a função perguntada. O ensaio mostra que a evidência central chegou antes do encerramento; não determina sozinho um critério geral que reconheceria esse momento em outras tarefas.

**Os 152.526 tokens são a soma do consumo de várias chamadas, não o tamanho de um pack ou de uma única janela de contexto.** A entrada de uma chamada posterior inclui mensagens e evidências anteriores. Em termos de medição:

`consumo acumulado = soma das entradas e saídas de todas as chamadas`

`maior entrada individual = maior contexto enviado em uma chamada`

Essas grandezas devem ser registradas separadamente. Tokens reaproveitados por cache podem permanecer na contagem reportada pelo provedor; esta tabela também não equivale a uma comparação de cobrança monetária.

O relatório calculou um cenário hipotético de parada antecipada: 15.623 tokens com Compass e 15.670 com ferramentas nativas, adotando uma saída final comum. **São estimativas contrafactuais, não economias observadas.** A parada antecipada ainda precisa ser executada e sua qualidade conferida.

O par utilizou somente `compass_search` e `compass_read`, com DeepSeek `deepseek-v4-pro`. Não acionou `compass_retrieve`, semântica, calor ou suficiência por obrigações. É um diagnóstico de um fluxo de consumo, com uma pergunta e duas execuções; não um benchmark geral do Compass. [S5]

## 5. Como o consumidor funciona no código inspecionado

Na integração de Tarefas examinada, a lista de ferramentas Compass exposta ao modelo contém apenas busca e leitura. A cada rodada, o controlador mantém ferramentas disponíveis, acrescenta os resultados ao histórico e reenvia a conversa ao modelo.

O fluxo encerra quando o modelo responde sem solicitar ferramentas, ou quando ocorre limite, erro ou cancelamento. Nesse caminho não foi encontrada a aplicação do status de obrigações para encerrar a aquisição. Portanto, instalar o Compass recente não muda, por si só, a política desse agente. [S6]

Há um fluxo distinto de medição que chama `compass_query` para observar candidatos e depois `compass_retrieve`, com paginação configurável. O código inspecionado não envia o `contract` de obrigações nessa chamada. Seu preparo de consulta utiliza um objeto também chamado de contrato, mas isso não significa que ele implemente a interface de obrigações do Compass.

Chamadas adicionais desse fluxo de medição podem ser necessárias para comparar candidatos, packs e perdas. Elas devem ser contabilizadas separadamente das chamadas de aquisição de uma tarefa real. Não se deve classificar toda paginação de laboratório como um loop desnecessário do agente. [S7]

## 6. As diferentes decisões de parada

Neste relatório, “parada do agente” significa principalmente encerrar a aquisição de contexto para uma solicitação. Isso não exige encerrar a aplicação inteira ou impedir a análise do material já recebido.

| Decisão | Critério atual ou condição | O que não comprova |
|---|---|---|
| Terminar o cálculo do calor | Aproximação numérica no tempo configurado; limites de erro, trabalho e cancelamento | Que as evidências necessárias foram encontradas |
| Terminar uma busca/expansão | Conclusão do procedimento ou limite de trabalho | Que nenhum fato relevante ficou fora dos candidatos |
| Fechar um pack | Seleção dentro do orçamento, com omissões e continuação quando aplicável | Que o pacote basta para a pergunta |
| Declarar obrigações satisfeitas | Todos os testemunhos exigidos presentes no pack atual | Que as obrigações representam completamente a intenção |
| Encerrar buscas no agente | Decisão aplicada pelo controlador consumidor | Que o encerramento foi adequado, se não houver critério verificável |
| Produzir a resposta final | Análise do material pelo agente | Que o custo da aquisição foi necessário ou eficiente |

### Calor, tempo de difusão e Dirichlet

A implementação atual calcula difusão livre `h(t) = exp(−tL)s`, com `L = D − W`. O parâmetro público é `theta = t × grau_máximo`; ele controla o tempo de difusão. O cálculo utiliza uniformização e série de Poisson, com limite de cauda de `10⁻⁸`, separado do erro de ponto flutuante.

Isso significa aproximar o estado de calor no tempo solicitado, e não rodar até que uma pergunta esteja semanticamente resolvida. O código testa o decaimento de energia em controles numéricos, mas **não usa a energia de Dirichlet como critério adaptativo de encerramento da aquisição**. Também não implementa condições de contorno de Dirichlet nesse operador livre.

Logo, não há evidência de um parâmetro Tau que, apenas ajustado, faça o agente parar no instante correto. Mudar o alcance da difusão e definir suficiência de contexto são decisões diferentes. Qualquer proposta para relacioná-las precisará de uma justificativa e de validação próprias. [S3]

### Evidência não encontrada

“Não encontrado” pode resultar de inexistência no escopo, seletor incorreto, ambiguidade, resolução estática incompleta, fonte indisponível ou restrição de recursos. O Compass não possui uma prova geral de ausência.

Por exemplo: exigir uma ocorrência de um operador de fusão para responder se a função funde transações cria uma obrigação impossível quando a resposta correta é negativa. Entregar o corpo completo pode permitir essa análise; buscar indefinidamente uma ocorrência positiva não resolve o problema.

O mecanismo atual retorna insuficiência e motivos quando não cumpre as obrigações. **Insuficiência não é uma ordem para repetir a mesma consulta.** Continua faltando estabelecer, no fluxo integrado, como encerrar com lacunas e como distinguir uma limitação terminal de uma condição recuperável. [S1–S2]

## 7. Objetivo desejado e lacunas que permanecem

A direção definida pelo usuário é:

`intenção interpretada pelo consumidor → uma chamada de aquisição ao Compass → processamento interno → um pack e resultado de aquisição → análise pelo consumidor`

Uma chamada ao Compass pode conter muitas operações internas. Também não significa uma única chamada à LLM em toda a aplicação: a interpretação da intenção pode ocorrer antes. O ponto é evitar que sucessivas rodadas externas sejam o funcionamento normal necessário para montar o contexto.

Esse objetivo admite uma resposta que encerre a aquisição com lacunas explícitas. Um orçamento finito não permite prometer evidência completa para qualquer solicitação. Encerrar corretamente sem conseguir provar algo também precisa ser um resultado compreensível.

As lacunas atuais são:

1. **Da intenção às obrigações:** os seletores atuais precisam de nomes ou identidades. Falta definir como descobrir o que deve ser localizado quando o usuário não conhece esses nomes, sem exigir buscas externas sucessivas ou acrescentar requisitos inventados.
2. **Da recuperação à cobertura:** busca aberta, expansão e avaliação de obrigações não estão unificadas em um processo que use as lacunas para orientar o trabalho interno.
3. **Da insuficiência ao término:** existe diagnóstico de obrigações, mas ainda não uma política integrada para encerrar por ausência delimitada, indisponibilidade, ambiguidade, orçamento ou falta de progresso.
4. **Do sinal à execução:** o consumidor precisa reconhecer o resultado e aplicar a decisão. O Compass não consegue desabilitar remotamente as ferramentas de outra aplicação.
5. **Do pack à entrada real da LLM:** uma certificação do pack original não garante que o consumidor entregue todo esse conteúdo ao modelo. Cortes ou transformações posteriores precisam ser observados.
6. **Do mecanismo à utilidade:** os controles mostram correção em casos definidos; ainda falta validar se o processo integrado obtém contexto suficiente em tarefas representativas, sem paradas prematuras ou buscas desnecessárias.

Essas lacunas distribuem responsabilidades entre produto, recuperação e integração. Atribuir todo o problema ao Multiagentes ou esperar que o calor sozinho o resolva não é sustentado pelas evidências disponíveis.

## 8. Perguntas e medições para orientar a investigação

A investigação deve definir o significado do encerramento e demonstrar seu efeito, antes de escolher um limiar ou um número de rodadas.

### Questões centrais

- Quais classes de solicitação podem ser atendidas em uma chamada com as capacidades atuais? Quais exigem declarar uma limitação?
- Como representar as evidências necessárias sem assumir antecipadamente a resposta, especialmente em perguntas negativas ou com dependências entre arquivos?
- Como distinguir “obrigações cumpridas” de “obrigações adequadas à intenção”? Quem faz a segunda avaliação e quanto ela custa?
- O que informa que operações internas adicionais ainda podem melhorar a cobertura? O que justifica encerrar quando a evidência continua insuficiente?
- Que resultado permite ao consumidor parar de buscar sem confundir insuficiência, erro, orçamento e resposta negativa?
- Qual é a contribuição do calor para a cobertura, usando as mesmas sementes e orçamento das alternativas? Sua convergência numérica não deve ser usada como substituto dessa medição.

### Hipóteses a testar, sem tratá-las como conclusões

| Hipótese | Ensaio que pode confirmá-la ou refutá-la |
|---|---|
| A continuação depois da evidência explica grande parte do consumo observado | Reexecutar casos comparáveis, registrar um critério de término aplicado durante a execução e medir consumo real e qualidade após a parada |
| O sinal de obrigações reduz novas buscas em tarefas delimitadas | Integrá-lo ao consumidor e comparar positivos, lacunas e obrigações mal especificadas |
| A insuficiência na primeira resposta decorre de seleção/orçamento | Medir evidência nos candidatos, na seleção, no pack serializado e na entrada efetiva do modelo |
| A expansão estrutural melhora a aquisição em uma resposta | Comparar sem expansão, BFS e calor com controles independentes, mesmas condições e custo registrado |

### Métricas que precisam permanecer separadas

| Métrica | Como medir e interpretar |
|---|---|
| Cobertura na primeira resposta | Fração das evidências de controle presentes no primeiro pack; separar obrigações declaradas de requisitos independentes da tarefa |
| Falsa suficiência | Casos declarados suficientes que não entregam seus testemunhos obrigatórios; qualquer ocorrência exige investigação |
| Parada prematura para a tarefa | Encerramento com obrigações cumpridas, mas faltando evidência necessária no controle independente; revela lacuna na formulação ou no critério |
| Continuação após o marco | Chamadas e tokens posteriores à disponibilidade da evidência e ao sinal de término; registrar ambos os instantes separadamente |
| Encerramento com lacunas | Se motivos são preservados e se casos não resolvidos terminam sem repetição improdutiva nem falsa conclusão de inexistência |
| Tamanho e composição | Bytes de texto, metadados, envelope MCP, omissões e tokens efetivamente apresentados ao modelo |
| Custo total e contexto | Entradas/saídas acumuladas, maior entrada individual, chamadas por finalidade, cache e custo monetário quando disponível |
| Trabalho do Compass | Latência, memória, disco e trabalho de busca/grafo/calor, separados das chamadas à LLM |
| Qualidade da resposta consumidora | Correção e sustentação das conclusões; avaliar separadamente da fidelidade e cobertura do pack |

Os controles devem incluir evidência existente e inexistente no escopo, símbolos ambíguos, relações não resolvidas, perguntas negativas, dependências entre arquivos, orçamento insuficiente e conteúdo indisponível. Casos reservados à avaliação não devem ser usados para ajustar os mecanismos.

Para aceitação de uma capacidade implementada, o esperado deve ser o que ela promete entregar. Para avaliar o produto integrado, também é necessário medir a utilidade desse material. Não é preciso esperar um produto “completo” para medir cobertura; tampouco acertar a resposta final, isoladamente, comprova que a recuperação foi suficiente ou eficiente.

Não há, nas evidências consultadas, base para impor um limiar universal de relevância, uma quantidade fixa de nós ou um número mágico de chamadas. Metas de custo e qualidade devem declarar a classe de tarefa, o orçamento e o conjunto de avaliação.

## 9. Restrições e cuidados para a investigação

O usuário determinou que ideias trazidas de pesquisa sobre o código legado fossem descartadas como base para a evolução. Este relatório não recomenda restaurar suas políticas, parâmetros ou protocolos de turnos. O ponto de partida é o núcleo Rust atual, suas garantias verificadas e o objetivo de aquisição em uma chamada.

A investigação não deve presumir que já existe uma solução apenas desativada. Também não deve tratar cursor ausente, orçamento consumido, cálculo terminado ou calor elevado como prova de suficiência.

O Multiagentes foi inspecionado somente por leitura. Nenhuma implementação foi feita para produzir este relatório. O gabarito PostSpark deslocado pelo usuário não foi consultado nem reconstruído; o caso real foi contextualizado a partir do relatório já existente.

**Pergunta que a investigação precisa responder:** como fazer uma aquisição limitada, verificável e encerrável em uma chamada produzir um resultado que o consumidor respeite, sem confundir término do algoritmo, cobertura de evidências e correção da resposta?

## 10. Fontes e identificação da versão

Os resultados de ensaios abaixo são registros de 24/10, não testes repetidos em 26/10. A inspeção atual confirmou o código descrito e a identidade do executável instalado. Sem identificação por commit nesta entrega, usa-se o hash técnico do binário:

`2f825d127e083e693f3ccecfdb65159980982cd998064a98d870a9746566bfd0`

- **S1 — Interface e limites das obrigações:** [AQUISICAO-OBRIGACOES.md](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/docs/AQUISICAO-OBRIGACOES.md>).
- **S2 — Critério de suficiência e seleção:** [completion em acquisition.rs](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/crates/compass-core/src/acquisition.rs:614>) e [aquisição e diagnósticos de expansão](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/crates/compass-core/src/acquisition.rs:720>).
- **S3 — Recuperação e operador numérico:** [retrieval.rs](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/crates/compass-core/src/retrieval.rs>) e [diffusion.rs](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/crates/compass-core/src/diffusion.rs:102>).
- **S4 — Resultados da aquisição por obrigações:** [summary.json](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/verification/acquisition/summary.json>), [results.json](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/verification/acquisition/results.json>) e [registro anterior de recuperação/calor](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/verification/recovery/summary.json>).
- **S5 — Ensaio real, fórmulas e referências aos eventos brutos:** [RELATORIO-PARADA-CONTEXTO-COMPASS-2026-10-24.md](</home/many/Documentos/Projetos linux/Multiagentes/docs/Bussola-SGC/RELATORIO-PARADA-CONTEXTO-COMPASS-2026-10-24.md>).
- **S6 — Ferramentas e ciclo do consumidor:** [runtime.py](</home/many/Documentos/Projetos linux/Multiagentes/src/context/runtime.py:20>), [preparo das chamadas ao modelo](</home/many/Documentos/Projetos linux/Multiagentes/src/context/runtime.py:240>) e [condição de encerramento](</home/many/Documentos/Projetos linux/Multiagentes/src/context/runtime.py:374>).
- **S7 — Fluxo distinto de medição:** [service.py](</home/many/Documentos/Projetos linux/Multiagentes/src/measurements/service.py:275>) e [preparation.py](</home/many/Documentos/Projetos linux/Multiagentes/src/measurements/preparation.py:188>).
- **S8 — Estado e escopo registrados:** [COMPASS_STATE.md](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/COMPASS_STATE.md>) e [PLANO-IMPLEMENTACAO.md](</home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/docs/PLANO-IMPLEMENTACAO.md>).
