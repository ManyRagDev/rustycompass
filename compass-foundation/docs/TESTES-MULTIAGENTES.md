# Avaliação do Compass pelo Multiagentes

Destinatário: agente responsável pela integração e pelos ensaios no Multiagentes. Preparado em 23/10/2026.

**Escopo desta referência:** roteiro da entrega até calor de 23/10. A aquisição por obrigações de 24/10 tem [interface e critérios próprios](AQUISICAO-OBRIGACOES.md), com controles executáveis em `verification/acquisition/`. Para essa nova etapa, meça a evidência declarada e efetivamente entregue; a resposta final do LLM não é critério de aceitação do Compass. As metas exploratórias abaixo não substituem esses controles nem os resultados observados.

O objetivo desta rodada é verificar se o agente recebe evidências suficientes para concluir tarefas com respostas corretas e rastreáveis, medindo qualidade e consumo. A simples utilização do MCP já foi demonstrada na rodada anterior. Agora existem busca lexical e semântica, grafo estrutural, seleção de contexto e expansão por BFS ou calor; essas capacidades precisam ser efetivamente exercitadas.

Este roteiro modela a próxima avaliação. Os números de qualidade abaixo são **metas iniciais propostas para o ensaio**, não resultados já obtidos nem garantias universais do Compass. Fixe-os antes de abrir os resultados reservados. Não execute uma campanha paga apenas por receber este documento: respeite a autorização e o orçamento da tarefa no Multiagentes.

## Evidência existente e expectativa desta rodada

Já foram registrados no Compass: 138 testes Rust aprovados, teste separado com o modelo semântico real, 74 controles numéricos de calor e fluxo integrado CLI/MCP em dois acervos sintéticos. Consulte `verification/recovery/summary.json` e `results.json`, relativos ao release SHA256 `f263dad974f60186eebae512b446d498557a72f2d39d3d72f6ce7588171dc09a`.

Nos dois acervos, o pack literal sem expansão encontrou um dos dois arquivos esperados; BFS e calor encontraram ambos. Isso comprova o comportamento nesses controles. Não comprova superioridade do calor nem recuperação suficiente em um projeto real. O antigo conjunto reservado de implementação já tem resultados conhecidos: sua reprodução será regressão, não uma nova avaliação cega.

| Pode ser exigido agora | Ainda precisa ser medido |
|---|---|
| Texto fiel, origem, revisão, limites e indisponibilidade explícitos | Quantidade de evidência necessária que aparece no primeiro pack |
| Consulta lexical e semântica em acervo preparado e habilitado | Relevância das primeiras posições em perguntas reais |
| Relações estáticas suportadas com evidência navegável | Cobertura do grafo nas construções usadas pelo projeto |
| Expansão BFS/calor acionável e identificada | Ganho do calor sobre BFS e ausência de expansão |
| Pack dentro do orçamento e continuação na mesma geração | Redução de chamadas, tokens, tempo ou custo do agente |

Não esperar compreensão completa do repositório, resolução de toda chamada dinâmica ou resposta pronta do Compass. Ele entrega evidências; o LLM interpreta a pergunta e redige a resposta. A busca semântica pode retornar vizinhos pouco úteis mesmo quando a resposta não existe. Calor não certifica relevância. Falta de atualização não é corrigida automaticamente por um watcher nesta versão.

## Preparação do ambiente

1. Reinicie o processo MCP para carregar o release e a configuração atuais. Use o launcher `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/scripts/compass-mcp.sh`. Obtenha os argumentos de cada ferramenta em `tools/list`.
2. Confira as seis ferramentas: `compass_search`, `compass_read`, `compass_files`, `compass_query`, `compass_neighbors`, `compass_retrieve`. A configuração precisa conter uma fonte `collection`.
3. Faça o teste inicial em `compass-demo`. As fontes antigas `demo`, `compass-source` e um eventual `postspark-control` de manifesto/pacote não passam a ter índices apenas por atualizar o executável.
4. Para avaliar um repositório real, prepare uma cópia congelada e somente leitura em um acervo gerenciado, com semântica habilitada. Registre fonte, geração, cobertura, modelo e executável. Use destino explícito fora da árvore avaliada. Veja `ACERVOS-RECUPERACAO.md` para os comandos.
5. Use uma configuração MCP própria para a campanha. Preparação e atualização são feitas pela CLI; o MCP permanece somente leitura. Não reduza campos legados de `limits` para tentar limitar a memória do modelo: as ferramentas indexadas recusam esse perfil. `max_output_bytes` precisa comportar o resultado MCP, que contém representações equivalentes além do payload.
6. No Multiagentes, `USE_COMPASS` é apenas o nome ilustrativo da variável. Com ela ativa, impedir acesso de busca/leitura do repositório por ferramentas nativas, inclusive shell, subprocessos e agentes auxiliares. Com ela desativada, usar a referência nativa. O avaliador pode ler os arquivos de controle fora da sessão do agente. Registrar qualquer desvio ou fallback; ele invalida a comparação como execução exclusiva pelo Compass.
7. Enviar ao contexto do LLM uma única representação da resposta MCP, preferencialmente `structuredContent`. Preservar revisão, cobertura, erros e referências. Não duplicar o JSON textual no histórico.

Não localizar nem reconstruir o gabarito PostSpark deslocado pelo usuário. Se essa avaliação depender dele, aguardar sua disponibilização explícita. Criar controles novos e independentes é suficiente para iniciar. Este roteiro não exige mudanças no código do repositório avaliado.

## Controles e separação do gabarito

Preparar 24 casos: seis para acertar a instrumentação e 18 reservados para avaliação, com três casos de cada categoria abaixo. Os casos reservados não serão usados para escolher consultas, prompts, orçamento ou parâmetros. Incluir Python, Rust e JS/TS/JSX/TSX em controles sintéticos; informar separadamente a composição do repositório real.

| Categoria | Resultado esperado |
|---|---|
| Identificador ou texto exato | Encontrar a ocorrência e ler os bytes corretos |
| Descoberta de arquivo | Localizar o caminho necessário e abrir seu conteúdo |
| Pergunta semântica em PT/EN | Encontrar a implementação por sua função, sem depender de copiar identificadores |
| Relação entre arquivos | Recuperar origem, destino e evidência de uma relação suportada |
| Pergunta com várias evidências | Entregar os trechos necessários para explicar condições, exceções e fluxo |
| Resposta ausente ou recurso não suportado | Reconhecer insuficiência ou limitação, sem inventar uma conclusão |

Para cada caso, o avaliador registra previamente: pergunta, revisão, fatos obrigatórios da resposta, evidências mínimas (arquivo, hash, intervalo), evidências alternativas aceitáveis, relações esperadas e construções fora de suporte. Uma evidência só conta se o texto entregue sustenta o fato; encontrar apenas o nome do arquivo não basta. Alternativas descobertas depois exigem julgamento documentado, sem favorecer uma variante.

O agente avaliado recebe apenas a pergunta e instruções de uso das ferramentas. Não recebe nomes de arquivos, consultas sugeridas ou respostas de controle. Gabaritos, relatórios e transcrições devem ficar fora do acervo e das ferramentas acessíveis a ele. Reiniciar o histórico entre casos e variantes.

## Sequência de execução

### 1 Conexão e integridade sem LLM

Reproduzir o teste existente:

```bash
python3 -B '/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/verification/mcp/check_ready.py'
```

Em `compass-demo`, executar `compass_retrieve` com `query: "checkoutSignal"`, `mode: "literal"`, `expansion: "heat"`, `budget_bytes: 16384`. Esperar evidências de `checkout.py` e `ledger.py`, geração identificada e informação do cálculo de calor. Executar `compass_query` semântico para `calcular o preço total de uma compra`: pelo menos um dos dois arquivos deve estar nos três primeiros resultados. Abrir os trechos com `compass_read`, passando a geração devolvida.

Em acervos descartáveis, conferir atualização e falhas: alterar declaração/import, incluir/excluir/renomear arquivo, atualizar, reiniciar, consultar gerações antiga e nova, continuar um pack antigo e consultar após remover a árvore original. Simular fonte/modelo indisponível, adulteração, cancelamento e interrupção de preparação somente nas cópias de teste. Verificar erro explícito e preservação da geração anterior quando aplicável. Não danificar a instalação pronta para reproduzir falhas já cobertas no núcleo.

Aceite: todos os controles determinísticos passam; nenhuma divergência de bytes, mistura de revisões ou acesso fora das fontes autorizadas. Um erro esperado e corretamente reportado aprova o controle negativo; não é falha operacional inesperada.

### 2 Qualidade de recuperação sem LLM

Enviar consultas fixas, preparadas antes dos resultados, aos 18 casos reservados. Comparar:

| Variante | Configuração |
|---|---|
| R0 | Literal, sem expansão |
| R1 | Lexical, sem expansão |
| R2 | Híbrida, sem expansão |
| R3 | Híbrida, BFS |
| R4 | Híbrida, calor com `theta=1` |

Usar a mesma geração, consulta e orçamento de 16.384 bytes. Em R2/R3/R4, confirmar que os canais disponíveis e os candidatos diretos coincidem; se não coincidirem, a comparação não isola expansão. Registrar também os dez primeiros candidatos de `compass_query`. Não presumir que o modo híbrido equivale à semântica isolada: conferir esse canal separadamente nos casos semânticos.

Medir primeiro pack e recuperação acumulada até três páginas, respeitando cursores e contabilizando os bytes reais totais. Não confundir benefício de três páginas com benefício sob o orçamento de uma página. Repetir consultas para tempo, sem contar repetições idênticas como novos casos de qualidade.

Depois, testar 8.192 e 32.768 bytes como análise de sensibilidade, sem substituir o resultado principal de 16.384. `theta` diferente de 1 só entra em experimento separado, escolhido no conjunto de desenvolvimento.

### 3 Resposta do agente e comparação com busca nativa

Executar os mesmos 18 casos com histórico novo, mesmo modelo efetivo, versão, parâmetros, instruções gerais e limites. Desabilitar seleção automática de provedor e registrar eventual fallback de modelo como mudança experimental.

Comparar inicialmente dois braços: busca/leitura nativa e Compass híbrido com expansão `none`. Não retirar casos difíceis ou falhos da média. Depois acrescentar BFS e calor aos mesmos casos para medir seu efeito na resposta. Se o orçamento não permitir todos os braços, registrar a comparação restante como pendente; o resultado sem LLM não comprova melhoria na resposta do agente.

No braço de expansão, configurar e registrar a variante para que o agente realmente a use; a mera existência de `compass_retrieve` não exercita calor. Manter busca e leitura diretas disponíveis pelo Compass. As consultas livres do LLM podem divergir entre execuções: esta etapa mede o sistema completo; a etapa anterior isola o mecanismo.

Fazer um piloto com os seis casos de desenvolvimento antes da campanha reservada. Alternar a ordem dos braços. Para alegar vantagem estável do sistema com LLM, repetir os braços comparados três vezes por caso e reportar dispersão e diferenças pareadas; com uma execução, classificar os resultados como exploratórios. Não executar todos os braços e repetições de uma vez sem estimar o custo pelo piloto.

## Métricas e critérios

Há 15 casos reservados com resposta e três negativos. Usar médias por caso, além dos numeradores e denominadores. Não dar peso maior a um caso apenas porque ele gerou mais chamadas.

| Métrica | Como calcular | Critério da rodada |
|---|---|---|
| Integridade | Evidências corretas em bytes, hash, revisão e origem / evidências verificadas | 100%; qualquer texto adulterado aceito reprova |
| Orçamento do pack | Bytes do JSON canônico do pack, incluindo metadados, contra `budget_bytes` | 100% dentro do teto; medir envelope MCP à parte |
| Recall de evidências | Evidências obrigatórias cobertas / obrigatórias, nos 15 casos positivos | Meta: macro ≥80% no primeiro pack e ≥90% até três páginas para R2 |
| Precisão dos trechos | Trechos que contribuem para a pergunta / trechos entregues, deduplicados | Meta: macro ≥70% no primeiro pack de R2; revisar trechos fora do gabarito antes de classificá-los |
| Perda no empacotamento | Evidências presentes nos candidatos examinados, mas ausentes do pack / evidências presentes nos candidatos examinados | Reportar primeiro pack e três páginas; separar perda de ranking e de orçamento |
| Posição da primeira evidência | Reciprocal rank e acerto nos três primeiros resultados | Reportar por categoria; nos controles semânticos simples, acerto em todos os casos previstos |
| Completude da resposta | Fatos obrigatórios corretamente respondidos e sustentados / fatos obrigatórios | Meta: macro ≥90% nos 15 positivos |
| Sucesso estrito da tarefa | Resposta final com todos os fatos críticos, citações válidas e sem erro crítico | Meta: pelo menos 12/15 positivos |
| Sustentação | Afirmações factuais verificáveis sustentadas por evidência entregue / afirmações factuais verificáveis | Meta: ≥95%; nenhuma afirmação crítica inventada |
| Casos negativos | Resposta reconhece ausência de evidência ou limitação, sem afirmar ausência universal a partir de busca incompleta | 3/3; não exigir resultados semânticos vazios |
| Conclusão operacional | Execuções que entregam resposta final / execuções planejadas | Meta: 100%; interrupções continuam no denominador |
| Consumo | Tokens reais por chamada e acumulados, chamadas LLM/tools, bytes, latência, pico RSS e disco | Medir por braço; ainda sem SLO universal ou economia comprovada |

As metas de recuperação referem-se à configuração R2 escolhida antes da campanha. R0/R1 são referências; R3/R4 são experimentos. Não escolher retrospectivamente a variante vencedora para declarar atingida a meta principal. Metas não atingidas revelam o que precisa melhorar, mesmo que os mecanismos funcionem corretamente.

Para precisão, agrupar trechos sobrepostos da mesma evidência; um trecho parcialmente útil conta como útil, mas registrar também volume redundante. No conjunto com resposta, nenhum trecho entregue implica precisão zero. Nos negativos, precisão/recall de relevância são não aplicáveis; avaliar a abstenção. Para candidatos examinados, fixar `k=10` na métrica de perda e declarar esse recorte: não é uma medida de todos os candidatos internos.

Checar bytes pelo formato canônico produzido pelo Compass, como faz `verification/recovery/acceptance.py`; reserializar JSON em outra linguagem pode mudar espaços, escapes e floats. Conferir separadamente limites e bytes efetivamente enviados pelo cliente ao modelo.

Uma tarefa interrompida tem qualidade final não avaliável, mas conta como insucesso operacional e insucesso estrito. Não excluí-la para melhorar o resultado. Julgar fatos e citações com revisão humana ou rubrica determinística; não depender apenas da autoavaliação do agente que respondeu.

## Orçamento do agente e medição de recursos

A rodada anterior tinha teto de 250 mil tokens acumulados. Parou após 202.226 porque a próxima chamada estimada, incluindo reserva de saída, não cabia. Aumentar o teto não elimina o custo de reenviar o histórico.

Perfil inicial **proposto**, igual nos braços e sujeito ao orçamento autorizado no Multiagentes: até 500 mil tokens acumulados de entrada + saída por execução, 16 chamadas LLM, 40 chamadas de ferramentas, saída máxima de 4.096 tokens por chamada e resposta final de até 1.000 palavras. Isso é teto, não consumo desejado. Contabilizar separadamente tokens de raciocínio quando o provedor os reportar e considerar sua regra de cobrança. O limite de contexto por chamada também precisa comportar entrada e saída.

Antes de cada nova rodada de exploração, estimar o custo da rodada seguinte **e de uma chamada final posterior**, incluindo o contexto que será reenviado. Quando ambos não couberem, encerrar a exploração enquanto ainda cabe a conclusão, desabilitando ferramentas na chamada final e pedindo resposta sustentada ou declaração explícita de insuficiência. Esse mecanismo deve estar no harness; a simples instrução ao LLM não garante a reserva.

Fixar um teto monetário por execução e por campanha após o piloto, com as tarifas do provedor efetivamente selecionado. Não reutilizar automaticamente US$0,50 do ensaio anterior nem inferir custo pelo nome do modelo. Se a autorização não comportar o desenho, reduzir previamente a campanha e declarar seu alcance; não elevar limites durante um caso.

Registrar tokens de entrada, saída, cache e raciocínio conforme disponibilizados, distinguindo estimado de reportado. Somar entrada reenviada em todas as chamadas; bytes do pack não são tokens. Reportar consumo médio por tarefa planejada e custo total do braço / tarefas bem-sucedidas, incluindo o custo das falhas. Se nenhum caso tiver sucesso, não calcular uma economia por sucesso.

Medir preparação e atualização fora da latência da consulta, mas incluí-las no custo total/amortizado. Medir processos Compass e harness separadamente. A inferência CPU atingiu aproximadamente 1,2 GiB nos pequenos controles locais; isso não é teto para o acervo real. Registrar hardware, tamanho/cobertura do acervo, objetos, índices, modelo e gerações retidas.

Para latência das ferramentas, usar ao menos cinco repetições por consulta, registrar primeira chamada após reiniciar o processo e chamadas seguintes separadamente. Não chamar isso de cache de disco frio sem controlar o cache do sistema. Reportar mediana, p95 e tamanho da amostra; com poucas observações, mostrar também os valores individuais. Acompanhar timeout de 30 segundos da configuração padrão; não aumentá-lo silenciosamente.

## Como decidir sobre o calor e sobre a integração

O calor passa no teste funcional quando é realmente executado, preserva evidências e informa parâmetros, cobertura e estado numérico. Nos controles pequenos completos, exigir convergência e cauda ≤1e-8. Em limites deliberadamente insuficientes, exigir estado incompleto explícito. Os controles numéricos independentes do núcleo já existem; não deduzir correção matemática a partir de uma resposta boa do LLM.

Para sugerir calor como padrão, exigir nesta avaliação: nenhuma regressão de integridade, ganho de pelo menos dez pontos percentuais de recall no primeiro pack sobre **BFS e none**, perda de precisão de no máximo cinco pontos percentuais e aumento de no máximo 25% na mediana da latência de recuperação. Esses são critérios experimentais propostos, com o mesmo orçamento e sementes. Examinar diferenças por caso e incerteza; uma amostra pequena não basta para anunciar superioridade geral. Se não cumprir ou se os resultados forem ambíguos, conservar calor opcional. Isso não reprova sua implementação.

Para alegar economia do sistema integrado frente ao nativo, exigir qualidade ao menos equivalente nesta amostra e comparar custo total por tarefa bem-sucedida, incluindo falhas. Uma queda de tokens sem resposta correta não é ganho. Uma resposta melhor com custo maior é um resultado possível e deve ser reportado como tal.

## Relatório esperado do Multiagentes

Entregar um relatório consolidado e os registros brutos associados, contendo:

- Identidade do build, fonte, geração, modelo local e LLM efetivo; hardware, cobertura, parâmetros e orçamento de cada braço.
- Tabela por caso com variante, evidências esperadas/recebidas, métricas do pack, resposta final, fatos atendidos, citações, tokens, tempo, custo e motivo de parada.
- Resumo pareado por variante e categoria, separando primeiro pack, continuação e resposta do agente.
- Falhas classificadas: captura/cobertura, formulação da consulta, ranking, resolução estrutural, seleção sob orçamento, continuidade, interpretação do LLM ou controle do harness. Registrar hipótese como hipótese até ter evidência.
- Conclusões independentes: integração funcional; integridade; qualidade da recuperação; qualidade da resposta; benefício da semântica; benefício de BFS/calor; custo e limites da amostra.

Não resumir todos os resultados como “Compass aprovado”. Um desfecho válido pode ser: integração e integridade aprovadas, recuperação suficiente na amostra, calor sem vantagem demonstrada e economia ainda inconclusiva. O ganho desta rodada é tornar essas conclusões mensuráveis, inclusive quando algum critério falhar.

Referências locais: `docs/MCP-INTEGRACAO.md`, `docs/ACERVOS-RECUPERACAO.md`, `verification/recovery/summary.json`, `verification/recovery/results.json`, `verification/recovery/heat-results.json` e `verification/semantic/results.json`, todos sob `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/`.
