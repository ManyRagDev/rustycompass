# Plano de implementação do Compass

Atualizado em 28/10/2026. Única referência de planejamento e progresso. Etapas 0–8 entregues; próximo incremento espectral planejado nas etapas 9–12, ainda sem implementação de produção.

## Objetivo e decisões autorizadas

Preparar acervos locais, conservar revisões, consultar conteúdo literal/lexical/semântico, navegar relações estruturais e selecionar evidências sob orçamento com expansão opcional por difusão de calor. Implementação independente de aplicações consumidoras. Rust compartilhado por CLI/MCP, Linux/CPU, Python somente verificação.

SQLite incorporado/FTS5; Tree-sitter para Python, Rust, JS/TS/JSX/TSX; embeddings locais E5-small/Candle; grafo factual separado da projeção simétrica unitária; calor livre exp(-tL)s via uniformização. Revisões conservadas sem exclusão automática, disco limitado inclusive staging. Sem execução de código do acervo, downloads implícitos em consulta ou envio de código a serviços.

Para o incremento 9–12, modelos, responsabilidades e escalonamento estão definidos em [Agentes e modelos](#agentes-e-modelos-do-incremento-espectral). Até três subagentes ativos além do coordenador, responsabilidades separadas e integração central. As atribuições registradas nas entregas 0–8 são históricas. Estados: pendente, em andamento, em verificação, concluída, bloqueada. Cada ramo inclui implementação, integração e correções; sem OS, listas fechadas, hashes administrativos ou aprovação entre fases. Hashes técnicos e gabaritos independentes são preservados.

## Árvore atual de execução

Raiz entregue: preparar → atualizar → recuperar → relacionar → difundir → entregar evidências.

Próximo incremento: descobrir sementes e escopo → observar escalas → selecionar e verificar um pack → encerrar aquisição → analisar no consumidor. As solicitações de 28/10 autorizam a avaliação das bibliotecas, a elaboração deste plano e a definição dos agentes/modelos; os itens pendentes abaixo descrevem a execução futura.

### 0. Consolidar — concluída
- [x] Inspecionar Git, reconciliar base e incorporar integração MCP externa sem alegar eficácia.
- [x] Preparar acervos independentes e separar desenvolvimento de avaliação final.
Resultado/conclusão: controles conhecidos antes da avaliação. Não localizar nem reconstruir gabarito PostSpark deslocado.

### 1. Captura e índice persistente — concluída
- [x] 1.1 Captura no mesmo fluxo seguro, objetos por hash, manifesto/pacote compatíveis, cancelamento/exclusões.
- [x] 1.2 SQLite incorporado, gerações imutáveis, staging, limite de disco, publicação consistente.
- [x] 1.3 FTS5/BM25, caminhos/conteúdo/identificadores segmentados, busca literal preservada.
- [x] 1.4 CLI prepare/status/files/query e fonte de acervo MCP.
Conclusão: preparar, reiniciar, consultar e ler sem árvore original. Responsável: Sol acervo; integração: coordenador.

### 2. Atualização e retomada — concluída
- [x] 2.1 Reconciliar inclusão/alteração/exclusão/rename/reversão por conteúdo.
- [x] 2.2 Reusar derivados por hash+produtor; invalidar dependentes ou escopo seguro do resolvedor.
- [x] 2.3 Lock escritor, geração atômica, retomada de interrupções, consultas antigas conservadas.
- [x] 2.4 Expor data de observação/cobertura, sem prometer sincronização contínua.
Conclusão: nenhuma mistura silenciosa de revisões durante atualização ou falha.

### 3. Grafo factual — concluída
- [x] 3.1 Parsers fixados, símbolos/escopos/imports/referências/chamadas com offsets.
- [x] 3.2 Resolução estática limitada nas linguagens selecionadas; ambiguidade/dinâmica/macros/external explícitos.
- [x] 3.3 Nós arquivo/símbolo, relações direcionadas tipadas e evidência.
- [x] 3.4 Vizinhança limitada, persistência por geração e invalidação interarquivos.
Conclusão: relações navegáveis com evidência aberta nos bytes. Responsável: Sol estrutura.

### 4. Semântica local — concluída
- [x] 4.1 Instalação explícita E5-small/Safetensors, revisão/hashes fixados, Candle CPU, offline após instalação.
- [x] 4.2 Segmentos por símbolos/blocos ou janelas, sem truncamento silencioso, prefixos/pooling/L2 corretos.
- [x] 4.3 Vetores identificados e reutilizáveis; cosseno exato em blocos, indisponibilidade explícita.
- [x] 4.4 Referência independente de embeddings; consultas PT/EN e perfil de recursos.
Conclusão: semântica retorna trechos verificáveis com modelo/geração. Responsável: Sol semântica.

### 5. Recuperação e evidence pack — concluída
- [x] 5.1 Literal/lexical/semântico com RRF60 pesos iguais, deduplicação e origens.
- [x] 5.2 BFS limitada como referência estrutural, sementes diretas identificadas.
- [x] 5.3 Orçamento bytes JSON incluindo metadados, expansão até25%, sobra reaproveitada.
- [x] 5.4 Objetos integralmente verificados, offsets/revisão, omitidos e continuação; envelope MCP medido separadamente.
Conclusão: pack limitado, verificável e continuável; acesso direto preservado.

### 6. Difusão de calor — concluída
- [x] 6.1 Projeção contains/imports/calls resolvidos, simétrica, pares únicos, peso1, sem autoarestas.
- [x] 6.2 L=D-W; exp(-tL)s; sementes normalizadas, theta=t*grau_max default1 faixa0..10.
- [x] 6.3 Uniformização esparsa/Poisson, cauda<=1e-8, cancelamento/limites explícitos, sem matriz densa.
- [x] 6.4 Expansão none/bfs/heat, parâmetros/cobertura/convergência expostos.
- [x] 6.5 Comparação mesma semente/orçamento; pesos experimentais separados; calor optativo.
Conclusão: calor numericamente verificado e acionável CLI/MCP, sem alegar probabilidade de relevância.

### 7. Integração e entrega — concluída
- [x] 7.1 Preservar search/read; tools files/query/neighbors/retrieve com fontes autorizadas.
- [x] 7.2 Fluxo completo CLI/MCP: preparar, buscar, ler, navegar, calor, atualizar, reiniciar, antigas/novas gerações.
- [x] 7.3 Diagnóstico/instalação, limites e recursos, launcher; preservar binário anterior ao atualizar.
Conclusão: testes de segurança/mecanismos e fluxo integrado passam; qualidade/custo/limitações registrados.

### 8. Aquisição verificável orientada por obrigações — concluída

**Resultado:** `compass_retrieve` e `compass retrieve --contract arquivo.json` recebem obrigações estruturadas e informam se o pack atual contém suas evidências obrigatórias. A interpretação da pergunta continua com o consumidor; o status não certifica a resposta final nem a fidelidade dessa interpretação.

- [x] 8.1 Fixar interface e controles independentes: contrato v1, 1–16 obrigações, até 16 KiB canônicos; declarações, ocorrências literais e relações estáticas diretas. Casos de desenvolvimento e avaliação separados.
- [x] 8.2 Resolver seletores sem escolher ambiguidades silenciosamente; conservar geração, hashes e intervalos. Incluir declaração e corpo completos, com wrappers reconhecidos. Verificar objetos antes de publicar texto.
- [x] 8.3 Selecionar a união das evidências obrigatórias antes de complementos, deduplicar sobreposições e respeitar o orçamento JSON completo. Quando não couber, priorizar obrigações completas na ordem declarada e depois trechos parciais.
- [x] 8.4 Avaliar somente evidências presentes na resposta atual. `SUFFICIENT_EVIDENCE_ACQUIRED` exige todas as obrigações satisfeitas e nenhuma pendência de interpretação. Omissão por orçamento, ambiguidade, indisponibilidade, tipo não suportado ou limite de trabalho impedem suficiência.
- [x] 8.5 Integrar CLI/MCP com contrato opcional e schema de pack novo. Preservar comportamento e cursores sem contrato; continuação nova fixa geração, contrato e parâmetros, sem suficiência cumulativa implícita.
- [x] 8.6 Executar controles independentes, revisão de falsa suficiência/proveniência/orçamento e regressões integradas; registrar cobertura na primeira resposta, fragmentação, redundância, bytes e recursos medidos.
- [x] 8.7 Documentar uso e limites, fornecer exemplos e cliente MCP independente; atualizar instalação local somente após verificação e conservar binário anterior.

**Conclusão:** casos positivos suportados cuja evidência cabe no orçamento ficam suficientes em uma chamada; nenhum controle negativo produz falsa suficiência; cada obrigação satisfeita aponta para testemunhos verificáveis no próprio pack; CLI e MCP concordam e as interfaces anteriores continuam funcionando.

**Execução:** Sol `gpt-6.1-sol` implementa núcleo e interfaces em módulos separados; outro Sol prepara controles sem consultar a implementação nova. Após a implementação, revisor independente confere os pontos de risco e o integrador fecha o fluxo público. Máximo de três subagentes simultâneos. Terra não está disponível na ferramenta de subagentes desta sessão; interfaces usam o fallback Sol. Lua/Luna pode executar apoio mecânico delimitado. Astra somente para defeito persistente ou conflito arquitetural demonstrado, registrando reprodução, tentativas e pergunta técnica específica; não há escalada por complexidade presumida.

**Progresso:** capacidade instalada e verificada. [Resumo e limites](../verification/acquisition/summary.json), [controles públicos](../verification/acquisition/results.json), [guia e exemplos](AQUISICAO-OBRIGACOES.md). Foram aprovados 166 testes Rust, fmt/Clippy e 82 controles independentes com 101 execuções CLI e 80 chamadas MCP. O teste real de embeddings também passou; foi executado antes do último ajuste exclusivo de seleção, e o módulo semântico permaneceu idêntico. O launcher final voltou a executar semântica, calor e contratos. Três respostas sem contrato são exatamente iguais às do binário anterior no mesmo acervo. Nenhum falso suficiente nos controles.

Nos dois acervos, todas as evidências obrigatórias foram entregues em uma chamada com orçamento de 16 KiB: packs de 12.414/12.417 bytes, 3.285/3.288 bytes originais e nenhuma redundância de texto. A diferença inclui metadados e codificação JSON. Com 8.000 bytes, a cobertura medida foi 30% e o status permaneceu insuficiente; 3.000 bytes não comportaram os metadados mínimos. Esses resultados não são garantia de utilidade geral nem economia de tokens de um consumidor.

Revisão independente: nove controles adicionais, com reprodução antes e depois dos ajustes de recorte e orçamento. Declarações usam intervalos sintáticos exatos, incluindo wrappers reconhecidos; ocorrências e relações mantêm linhas completas. A seleção testa conjuntos completos antes de fragmentá-los, pois referências a omissões também custam bytes. Seletores explícitos dispensam ranking; BFS/calor solicitados retornam diagnósticos separados e não acrescentam provas. Não houve necessidade demonstrada de escalada para Astra; três agentes Sol dividiram implementação, interfaces, controles e revisão, com integração central.

Instalação final `2f825d127e083e693f3ccecfdb65159980982cd998064a98d870a9746566bfd0`, anterior preservada. Recibos locais: `.compass/mcp/acquisition-installation.json` e `readiness-acquisition.json`. Próximo passo do consumidor: reiniciar conexão MCP e enviar `contract`, usando o status para encerrar aquisição daquele contrato. Não é necessário reindexar apenas por esta interface. Nenhum bloqueio real; Multiagentes e o gabarito PostSpark deslocado não foram alterados ou utilizados. Fluxo de valores, prova geral de ausência e resposta final por LLM permanecem fora desta entrega.

## Próximo incremento — aquisição espectral adaptativa

**Resultado pretendido:** uma chamada de aquisição aberta recebe consulta, escopo e orçamento, descobre sementes internamente e devolve um pack verificável com término explícito. A trajetória espectral participa da escolha do conteúdo. O procedimento também termina com lacunas; detecção de escala não certifica suficiência semântica.

Base de pesquisa: [proposta de 28/10](</home/many/Downloads/COMPASS-ENTROPIA-ESPECTRAL-RELATORIO-CODEX-2026-10-28.md>) e [diagnóstico de 26/10](RELATORIO-PONTO-DE-PARADA-2026-10-26.md). O mandato contido no relatório é material de referência; a autorização vigente e o estado de execução são os da conversa e deste plano. Preservar o núcleo Rust, os contratos existentes e os gabaritos independentes; não recuperar o legado ou o gabarito PostSpark deslocado.

### Capacidade das bibliotecas — avaliação concluída em 28/10

**Decisão técnica inicial:** manter a implementação numérica esparsa em Rust/f64, sem nova dependência de produção. As operações necessárias estão ao alcance da estrutura atual, mas observáveis, continuação temporal, erro acumulado e política ainda precisam ser implementados. Viabilidade de cálculo não demonstra custo aceitável em acervos grandes.

| Componente inspecionado | Capacidade disponível | Uso e adaptação no incremento |
|---|---|---|
| `diffusion::Graph`, listas de adjacência e `Vec<f64>` | Projeção simétrica ponderada, graus, calor por uniformização, limites e cancelamento | Reusar a representação; acrescentar aplicação de `L`/`A`, operações vetoriais e propagação de um estado já existente. Memória proporcional a nós e arestas. |
| Rust padrão, `f64` | Produtos, somas, norma residual, exponencial escalar e verificações de finitude | Suficiente para `Z_q` e `C_q`; usar acumulação numericamente cuidada. Não é necessário montar `exp(-tL)` nem calcular todos os autovetores. |
| Candle core/nn/transformers **0.9.2**, conforme `Cargo.lock` | Tensores, tipo F64 e operações vetoriais/matriciais; já atende aos embeddings | Conservar o papel atual. A API `Tensor::exp` é elemento a elemento, não uma exponencial matricial. A API inspecionada não oferece um solver pronto para este percurso espectral esparso. Migrar a difusão para tensores não é requisito. |
| NumPy **2.3.5**, Python **3.12.14** do runtime disponível | `numpy.linalg.eigh`, exponenciais e álgebra densa para matrizes pequenas | Referência independente offline, já usada por `verification/recovery/heat_oracle.py`; estender os controles, sem Python no núcleo de produção. O `python3` padrão do sistema não tinha NumPy nesta inspeção. |
| SciPy | `expm_multiply` é uma alternativa de referência para ação da exponencial | **Não instalado** no runtime conferido e dispensável para o primeiro incremento. Não planejar sua utilização como se já estivesse disponível. |
| `serde`/JSON, CLI/MCP e módulos de recuperação/aquisição | Transporte, orçamento serializado, revisão, proveniência, materialização e verificação | Reaproveitar; acrescentar política opcional e resultado terminal versionado. SQLite/Tree-sitter/E5 não precisam ser substituídos por esta proposta. |

Não há dependência direta de `nalgebra`, `faer`, `ndarray` ou `sprs` no núcleo atual. Uma biblioteca adicional só deve ser avaliada se perfil de custo ou falha numérica demonstrar necessidade. O algoritmo inicial não exige GPU, diagonalização global nem cálculo online da zeta. Uma matriz densa f64 de 100 mil nós ocuparia cerca de 80 GB apenas para seus coeficientes, antes de qualquer decomposição; a produção deve continuar esparsa.

Referências de API conferidas: [Candle Tensor 0.9.2](https://docs.rs/candle-core/0.9.2/candle_core/struct.Tensor.html), [DType/F64](https://docs.rs/candle-core/0.9.2/candle_core/enum.DType.html), [NumPy eigh](https://numpy.org/doc/stable/reference/generated/numpy.linalg.eigh.html) e [SciPy expm_multiply](https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.expm_multiply.html). As versões instaladas acima vêm do lockfile e dos imports locais, não da versão mais recente dessas páginas.

**Conferência executada para esta avaliação:** três testes Rust de difusão passaram; o oráculo existente passou em 74 controles contra o `heat_probe` recompilado, com maior erro L1 dos casos completos de `9,3374e-9`, dentro dos limites conferidos pelo oráculo. Um probe temporário calculou `Z_q` e `C_q` externamente em NumPy a partir de estados do solver Rust atual: 15 casos, incluindo dois nós e dois K6 com três sementes, tiveram erros absolutos máximos de `1,6005e-8` e `7,6029e-9`, respectivamente. Na avaliação preliminar, esses observáveis ainda não estavam implementados no Rust; a entrega posterior está registrada em 11A. Esses controles preliminares são pequenos e não avaliam a seleção de evidências.

Reprodução da base: `scripts/cargo-local.sh test -p compass-core --lib diffusion::tests --locked --offline`; executar `verification/recovery/heat_oracle.py --output <arquivo-novo>` com Python que contenha NumPy. Nesta sessão foi usado `/home/many/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3`. Os resultados e o probe exploratório estão em `/tmp/compass-spectral-feasibility-2026-10-28/`; são temporários e não substituem os controles permanentes a produzir em 11A. Os resultados históricos de recuperação foram preservados. Não foi reexecutada a suíte completa nesta avaliação de planejamento.

### Sequência e dependências

| Etapa | Entrega | Dependências |
|---|---|---|
| 9 | Referência de recuperação e conjunto de desenvolvimento | Base 0–8; começa imediatamente na execução do incremento. |
| 10 | Aquisição terminal no fluxo de Tarefas | Referência/interface de 9; pode usar a política atual enquanto 11 evolui. |
| 11A | Trajetória numérica e observáveis no Rust | Solver atual e controles pequenos; pode avançar enquanto 9/10 são verificados. |
| 11B | Seleção de evidências e política espectral experimental | 9 + 11A; primeiro uso pelo laboratório existente. |
| 12 | Avaliação e decisão | Comparação de packs requer 11B; benefício no fluxo real requer também 10. |

O caminho curto até testes da proposta é **9 → 11A → 11B → laboratório**. A integração **10** é necessária para concluir a aplicação da parada em Tarefas, mas não bloqueia o experimento numérico ou a comparação de packs. A numeração agrupa responsabilidades; não exige esperar pela reforma inteira do consumidor. Watcher, DG, GUI completa e serviço remoto permanecem posteriores.

### Agentes e modelos do incremento espectral

**Execução concluída em 28/10:** o usuário autorizou executar o plano com os devidos acionamentos. Foram acionados `numerico` (Sol/xhigh), `aquisicao`, `avaliacao` e `consumidor` (Sol/high), além de `revisao` (Astra/xhigh), em contextos próprios. A coordenação permaneceu neste chat, sem alteração automática de modelo. O apoio Luna não foi necessário; nenhum chat adicional foi criado.

| Agente | Modelo exato / esforço | Responsabilidade e entrega |
|---|---|---|
| **Coordenador e integrador** (`coordenador`) | `gpt-6.1-sol` / `high` | Conduzir o objetivo, fixar interfaces mínimas, distribuir trabalho, resolver conflitos e integrar núcleo/CLI/MCP/consumidor. Manter este plano, conferir a árvore efetivamente testada e entregar o resultado com limites. Responsável pelo fechamento das etapas 9–12. |
| **Engenheiro numérico** (`numerico`) | `gpt-6.1-sol` / `xhigh` | Implementar 11A: aplicação esparsa do operador, propagação sem renormalização intermediária, escalas/meias escalas, `Z_q`/`C_q`, erro, orçamento acumulado e cancelamento. Entregar API interna e testes de implementação; corrigir divergências reproduzidas pelo avaliador. |
| **Engenheiro de aquisição** (`aquisicao`) | `gpt-6.1-sol` / `high` | Implementar 11B: sementes/escopo, dependências, candidatos, escolha de janela, término e lacunas; integrar a opção experimental à CLI/MCP. Demonstrar que a escala muda o conteúdo do pack e preservar os caminhos anteriores. |
| **Engenheiro do consumidor** (`consumidor`) | `gpt-6.1-sol` / `high` | Implementar 10 e a integração do laboratório: coleção/geração, validação do pack, transição persistida para análise, bloqueio de novas buscas, reserva de resposta e métricas do input real. Trabalhar no Multiagentes quando houver acesso de escrita efetivo. |
| **Avaliador independente** (`avaliacao`) | `gpt-6.1-sol` / `high` | Preparar 9 e os controles de 11A/12 a partir da especificação e das fontes dos casos: referência NumPy, gabaritos, alternativas válidas, negativos, baselines e medição. Congelar os casos reservados antes de avaliar a política. Entregar resultados reproduzíveis sem escrever o algoritmo de produção. |
| **Revisor crítico** (`revisao`) | `gpt-6-astra` / `xhigh` | Revisar formulação/erro, conformidade da implementação, validade dos oráculos, fronteiras de evidência, falsa suficiência e comparação experimental. Receber contexto próprio; apontar defeitos com reprodução, impacto e critério de correção. Não aprovar código que tenha escrito. |
| **Executor de verificações** (`execucao`, opcional) | `gpt-6-luna` / `medium` | Executar comandos já definidos, recolher códigos de saída, conciliar artefatos/versões e resumir falhas literais. Usar apenas quando houver um lote mecânico independente suficiente para compensar outra delegação. Não alterar testes, gabaritos, tolerâncias ou política. |

**Motivo da escolha:** Sol concentra o trabalho de engenharia e continuidade; o cálculo numérico recebe `xhigh` pela propagação de erro e composição temporal. Astra recebe a revisão de maior risco, em contexto separado da autoria. Luna fica no trabalho repetível com resultado verificável. A [orientação oficial de modelos](https://learn.chatgpt.com/docs/models) e a [documentação de subagentes](https://learn.chatgpt.com/docs/agent-configuration/subagents) sustentam esses usos gerais; a alocação e os esforços específicos são decisões para este projeto, não um benchmark realizado entre modelos. Terra não tem papel obrigatório neste lote; sua disponibilidade atual não obriga distribuir trabalho por todas as famílias.

**Ordem de atuação, respeitando quatro agentes ativos no total**

| Onda | Agentes ativos além do coordenador | Resultado que libera a seguinte |
|---|---|---|
| 1 — referência e núcleo | `numerico`, `avaliacao`, `consumidor` | Casos de desenvolvimento e interface numérica estabelecidos; propagação/observáveis em verificação; consumidor avança com packs atuais ou simulados. |
| 2 — revisão numérica | `revisao`, `numerico`, `avaliacao` | Revisão das fórmulas, erro e referências; divergências críticas reproduzidas e corrigidas antes de depender do sinal para selecionar packs. O consumidor fica inativo durante esta intervenção se as três vagas estiverem ocupadas. |
| 3 — aquisição e integração | `aquisicao`, `consumidor`, `avaliacao` | Seleção espectral funcional na CLI/MCP e no laboratório; tarefa aplica o término; pacote e input real conferidos. O numérico retorna pontualmente no lugar de uma vaga, se necessário. |
| 4 — fechamento | `revisao`, `avaliacao`, `execucao` opcional | Revisão do conjunto e dos testes, regressões e comparação final; coordenador integra correções com seus autores e entrega. |

As ondas organizam dependências, não criam uma aprovação humana obrigatória por etapa. Liberar uma vaga ao concluir o trabalho; não manter agentes ativos apenas aguardando. O coordenador executa apoio mecânico quando isso dispensar o agente opcional. Se houver menos vagas ou orçamento menor, serializar mantendo a separação de autoria e revisão.

**Fronteiras e critérios de responsabilidade**

1. O numérico é responsável pelo solver e seus observáveis; aquisição é responsável pela política de seleção e entrada CLI/MCP; consumidor é responsável pelo uso do resultado em Tarefas/laboratório. Coordenador arbitra arquivos compartilhados. Cada arquivo ou trecho em alteração tem um escritor ativo; testes de build da árvore compartilhada são coordenados para não medir uma combinação incompleta de mudanças.
2. Avaliador escreve oráculos e casos a partir da especificação antes de consultar o algoritmo novo. Pode inspecioná-lo depois para diagnosticar divergências, sem ajustar o esperado para fazê-lo passar. O revisor Astra examina também os testes e gabaritos, evitando que o autor do oráculo seja seu único juiz.
3. Gabaritos reservados ficam fora do contexto dos implementadores e fora do acervo consultado. Dar a avaliador/revisor um contexto novo e somente as referências necessárias; não herdar automaticamente todo o histórico de implementação. Separação de agentes na mesma máquina é uma regra operacional, não isolamento técnico de arquivos. Após abrir os reservados, mudanças de política exigem nova avaliação reservada para alegar generalização.
4. O coordenador fixa a interface interna mínima antes de conectar os ramos: estado/escala/erro/trabalho no cálculo; candidato/dependências/cobertura na seleção; condição estrutural/término/lacunas no transporte. Acordos podem evoluir com evidência; não gerar contratos administrativos adicionais. O único plano/progresso permanece neste arquivo.
5. Cada agente devolve comportamento entregue, arquivos alterados, verificações executadas na versão indicada, limitações e próximo ponto de integração. O revisor devolve achados acionáveis, não apenas uma aprovação genérica. O coordenador não declara conclusão com problema crítico ou verificação essencial pendente.
6. Erro funcional volta ao autor correspondente. Erro matemático persistente ou desacordo sobre tolerância é analisado por Astra com reprodução mínima antes de outro patch. Se Astra passar a escrever a correção de produção, outra instância revisora, com contexto novo e sem autoria dessa correção, assume seu aceite independente.
7. O modelo/esforço deve ser enviado explicitamente na criação de cada agente. Nesta ferramenta, isso requer contexto novo ou parcial, em vez de herança integral. Conferir disponibilidade ao executar; indisponibilidade não autoriza substituição silenciosa. A alternativa técnica para um trabalhador Sol é serializar ou propor outra configuração; a revisão crítica continua pendente até haver revisor adequado.
8. Esta divisão não autoriza downloads, campanhas pagas, publicação ou promoção fora do escopo da execução. Os testes com provedores e o acesso de escrita ao consumidor continuam com os limites já descritos neste plano. Nenhum papel exige criar chats adicionais no aplicativo.

### 9. Referência de recuperação — concluída

- [x] 9.1 Confirmar executável, geração, capacidades MCP e fontes preparadas; usar novas saídas de verificação sem sobrescrever evidências históricas.
- [x] 9.2 Rodar um conjunto curto de desenvolvimento sem LLM: recuperação aberta sem expansão, BFS e calor fixo, mesma consulta, geração e orçamento. Começar pelo perfil existente de 16 KiB; usar orçamento insuficiente como controle distinto.
- [x] 9.3 Exercitar separadamente obrigações explícitas, completas e incompletas. Elas verificam o mecanismo atual; seus seletores não podem fornecer a resposta oculta à nova aquisição aberta.
- [x] 9.4 Preparar evidências esperadas e alternativas aceitáveis fora do acervo acessível ao recuperador. Cobrir pergunta local, dependência interarquivo, travessia de módulos, mesma semente com escopos diferentes, resposta negativa, dependência dinâmica/externa e orçamento insuficiente.
- [x] 9.5 Registrar suporte entregue no primeiro pack, omissões, bytes, trabalho e tempo. Manter os casos reservados fechados enquanto forem escolhidos parâmetros ou regras.

**Aceite:** a medição distingue candidatos, texto efetivamente entregue e suporte esperado; os braços usam os mecanismos declarados. Esta referência curta não precisa aguardar campanha com LLM nem redesenho de portal.

**Evidência desta execução:** `verification/spectral/legacy-baseline-dev.json`: 42 aquisições (7 casos × 2 orçamentos × 3 braços), quatro contratos completos/incompletos e cinco comparações integrais CLI/MCP. Conferência independente de bytes, hashes, intervalos e tamanho do pack. A 16 KiB, suporte entregue de 5/9 unidades sem expansão e 9/9 com BFS/calor; isso descreve estes controles e não demonstra vantagem espectral. Gabaritos reservados congelados e ainda não avaliados. A referência identifica o executável anterior, não o novo solver em desenvolvimento.

### 10. Aquisição terminal no consumidor — concluída

Na inspeção anterior, `Multiagentes/src/context/compass.py` reconhecia as seis ferramentas; `runtime.py` autorizava somente busca/leitura, e `profiles.py` resolvia identidades por manifesto/pacote. O laboratório em `src/measurements/` já trabalha com coleções, compara packs e oferece resposta ao primeiro pack sem ferramentas. Reutilizar essas capacidades sem confundir a avaliação de laboratório com uma tarefa real.

- [x] 10.1 Estender a identidade de fonte e validação do runtime para coleção/geração; verificar escopo também nos itens e referências do pack, antes de persistir ou expor conteúdo.
- [x] 10.2 Definir a transição persistida `aquisição → análise → resposta`, com política e geração fixas. A aquisição termina por pacote selecionado ou motivo limitado explícito; insuficiência não dispara repetição automática da mesma consulta.
- [x] 10.3 Entregar uma representação do pack à LLM, conservar evidências e lacunas e desabilitar buscas dessa aquisição na análise. A guarda do controlador deve impedir chamadas mesmo se o modelo as solicitar. Retomada não reabre uma aquisição já terminada; mudança de escopo/revisão inicia outra.
- [x] 10.4 Reservar recursos para a resposta e registrar o input efetivamente enviado, sem duplicar o envelope MCP. Qualquer corte ou transformação exige nova verificação do suporte; preferir conservar os testemunhos.
- [x] 10.5 Verificar com clientes simulados e MCP real: pacote completo, pacote com lacunas, orçamento, cancelamento, falha e retomada. O primeiro aceite é offline; ensaio com provedor usa destino e orçamento autorizados.

**Aceite:** a tarefa chega à análise com o pack esperado e não continua buscando para aquela aquisição. Esse mecanismo serve às políticas fixa e espectral; o status atual de obrigações mantém seu significado.

**Entrega:** 11 arquivos aplicados ao Multiagentes após conferência dos originais, revisão independente e 126 testes na cópia. Na árvore aplicada, 63 testes com MCP real e LLM simulada passaram. Aquisição opcional na tarefa fixa geração, conserva o pack integral e o recibo, registra o input efetivo, reserva saída e bloqueia ferramentas após término. Retomada revalida sem repetir retrieve. Um `SourceBind` explícito pode vincular nova coleção/geração/aquisição para revisão S1, preservando histórico S0. Aprovação exige união de intervalos cobrindo todos os hunks alterados e três linhas de contexto. Criação exige conteúdo novo integral; deleções, reduções de linhas e esvaziamento sem testemunho anterior permanecem bloqueados. Não há refetch automático nem envio de diff nativo fora do pack. O grafo graphify foi atualizado por AST, sem LLM. Backups locais em `.compass/spectral-consumer-backup-2026-10-28/`.

**Tempo de integração:** o cliente atual usa timeout de 30 s na chamada MCP, 35 s na sessão e 38 s na espera externa. O trabalho acumulado da trajetória precisa caber no prazo efetivo ou devolver limitação antes dele. Qualquer mudança desses valores deve ser justificada por medição e coordenada com o servidor, sem apenas ampliar todos os tetos.

### 11A. Trajetória e observáveis espectrais — concluída

Para a primeira implementação, manter operador simétrico livre e grafo de uma geração fixos. Usar `A = L/d_max`, `theta = t*d_max`, semente `v` não negativa normalizada uma vez e estados:

```text
h(theta) = exp(-theta*A) v                       # seleção do pack
y(theta) = exp(-(theta/2)*A) v                  # observável na MESMA escala
Z_q(theta) = dot(y,y) / dot(v,v)
mu = dot(y,A*y) / dot(y,y)
C_q(theta) = theta² * norm(A*y - mu*y)² / dot(y,y)
```

Usar `h(theta)` no lugar de `y(theta)` calcula uma medida relacionada à escala dupla; esse erro deve ser detectado pelos testes. Nomear `C_q` como curvatura espectral condicionada à semente. Entropia espacial, entropia espectral global e zeta permanecem grandezas distintas; as duas últimas podem ficar no laboratório.

- [x] 11A.1 Separar preparação da semente, aplicação esparsa do operador e propagação de um vetor já existente. Reutilizar buffers, observar finitude e preservar cancelamento. O caminho público atual mantém comportamento e faixa de `theta` anteriores.
- [x] 11A.2 Implementar continuação por incrementos temporais limitados, inicialmente usando uniformização e a propriedade de semigrupo. Construir uma agenda ordenada com as escalas e meias escalas necessárias; evitar recomputar cada trajetória desde zero e limitar os estados mantidos em memória.
- [x] 11A.3 Contabilizar erro de truncamento acumulado e erro de arredondamento separadamente. A propagação interna não pode renormalizar a massa a cada passo: `Graph::heat` hoje normaliza toda entrada, portanto chamá-lo repetidamente como está esconderia perda de massa e alteraria `Z_q`.
- [x] 11A.4 Implementar `Z_q`, `C_q`, checagem de denominadores e qualidade numérica. Transportar limites/estimativas de erro até a decisão de janela; zero calculado não implica estabilidade. Se não houver precisão para distinguir a classificação, retornar indeterminação numérica. Não apresentar uma estimativa empírica como certificação de arredondamento.
- [x] 11A.5 Aplicar um orçamento acumulado à requisição inteira: passos, aplicações do Laplaciano, visitas a arestas **e nós**, leituras, candidatos e tempo. A contagem atual de arestas sozinha não limita trabalho em grafos com muitos isolados. Não reiniciar o orçamento em cada escala/meia escala/refinamento.
- [x] 11A.6 Tratar sem arestas, semente nula, uniforme/estacionária, componentes desconectadas, NaN/overflow e cancelamento. Ausência de dinâmica é sinal não informativo, nunca suficiência.
- [x] 11A.7 Criar controles permanentes Rust/NumPy e comparar formulação espectral, momentos e residual. Usar tolerância absoluta perto de zero e relativa onde justificável; justificar a tolerância de `C_q` em função de escala, denominador e erro do calor, sem copiar `1e-8` como tolerância universal.

Para propagar a incerteza, aproveitar a contração do calor e o limite `||A||₂ ≤ 2` sob estas premissas. Dado um limite válido `delta` para o erro de `y`, um limite inferior da norma é `max(0, ||y_aproximado||₂ - delta)`. Se ele não for positivo, não avaliar a razão como sinal de parada. Derivar e verificar os limites para os produtos e o resíduo; distinguir a parte analítica da cauda da estimativa de arredondamento. Classificações que dependam de uma precisão não demonstrada ficam indeterminadas. Manter o grafo completo da geração como operador inicial; qualquer futuro recorte ou crescimento exige tratamento explícito da fronteira e nova época de comparação.

**Por que adaptar o solver:** hoje a faixa é `theta ≤ 10`, a série começa em `exp(-theta)` e termina em até 128 iterações. No exemplo K6, `d_max=5,02` e `t=360` exige `theta=1807,2`; apenas aumentar o máximo não resolve underflow nem truncamento. A composição por incrementos evita esse uso direto da série, mas pode custar muitas aplicações do operador. Avaliar outra aproximação esparsa somente se o perfil mostrar que a abordagem inicial não cabe no orçamento.

**Controles mínimos:** dois nós com `C_q(t)=t²/cosh²(t)`; dois K6 e ponte 0,02, incluindo escalas lentas em grafo pequeno; uma/duas sementes; uniforme e isolados; reescala de pesos; componente desconectada sem semente e com maior grau; igualdade entre propagação direta e composta dentro do erro; limite acumulado e cancelamento. Comparar componente desconectada em **tempo físico fixo**, convertendo `theta` se `d_max` mudar.

**Aceite:** concordância com referências independentes nos regimes suportados, rejeição/limitação explícita nos demais, sem matriz densa de produção, sem confundir perda de precisão com patamar. O desempenho em grafo grande continua sujeito à etapa 12.

**Checkpoint:** API incremental e probe implementados; 11 testes Rust focados, formatação e Clippy passaram. Astra repetiu 25 grafos/230 escalas contra NumPy, verificou intervalos separadamente e 13 controles de limites/cancelamento, sem defeito confirmado no solver. A lacuna encontrada no comparador de metadados numéricos foi corrigida: 25 grafos/230 escalas passam, 21 mutações são rejeitadas e 24 controles de execução mais duas entradas não finitas passam. Evidências permanentes em `verification/spectral/{numerical-comparison-final,comparator-mutations,runtime-controls}.json`; o núcleo mantém o arredondamento explicitamente não certificado.

### 11B. Seleção adaptativa e interface experimental — concluída

**Descoberta e representação antes da regra de parada**

- [x] 11B.1 Descobrir sementes com o ranking existente dentro da aquisição; fixar consulta, geração, pesos, normalização e projeção durante a trajetória. Tornar limites de sementes/candidatos observáveis, sem acesso ao gabarito.
- [x] 11B.2 Representar escopo sem exigir lista manual de símbolos ou obrigações. Começar com análise local de declaração/corpo e dependências/chamadores estáticos resolvidos. Registrar se o escopo foi fornecido ou inferido; ambiguidade não pode reduzir silenciosamente a pergunta. Todos os caminhos transitivos, comportamento dinâmico geral e valor efetivo de configuração externa não recebem garantia no primeiro incremento.
- [x] 11B.3 Manter as relações direcionadas e seus testemunhos fora da projeção simétrica. Implementar fechamento delimitado segundo o escopo: incluir evidências estáticas necessárias à interpretação e declarar as não resolvidas, externas, ambíguas ou excluídas por limite. Importação ou peso térmico, por si, não provam necessidade semântica. Tratar ciclos por identidade e deduplicação.
- [x] 11B.4 Compartilhar materialização e regras de fechamento entre calor fixo e adaptativo. Construir candidatos de região a partir dos escores térmicos; primeira regra a investigar: sementes mais prefixo ordenado que alcance uma fração de massa definida pelo perfil. Conservar desempate determinístico, massa omitida e cortes. Não deixar a quota antiga de expansão decidir silenciosamente o pacote experimental.
- [x] 11B.5 Selecionar unidades coerentes de evidência e dependências sob orçamento, verificar intervalos no payload final e contabilizar JSON/metadados. Se uma unidade não couber, marcar lacuna; não manter afirmação de cobertura que dependia da parte removida. Reutilizar leituras e verificação entre escalas.

O materializador pode reutilizar verificadores da aquisição por obrigações, mas o modo aberto não deve converter requisitos descobertos em até 16 obrigações truncadas e declarar suficiência. A lista de dependências descobertas tem limite próprio e relata a fronteira não explorada.

**Primeira política refutável**

Usar uma grade geométrica positiva com razão inicial 2, compatível com reutilização das meias escalas. O ponto inicial, alcance máximo, precisão, fração de massa, queda relativa ao pico e número de amostras de persistência pertencem a um perfil versionado, escolhido no desenvolvimento e congelado antes da avaliação reservada. Esses parâmetros são escolhas experimentais, não constantes universais derivadas do exemplo K6.

- [x] 11B.6 Identificar uma subida e descida de `C_q` distinguíveis do erro, seguidas por janela com curvatura abaixo de uma fração do pico precedente e persistência em escalas vizinhas. Requerer estabilidade dos intervalos de evidência e das dependências relevantes, com escopo suportado. Rejeitar como prova de janela qualquer estabilidade explicada por `top-k`, deduplicação, orçamento ou limite de trabalho.
- [x] 11B.7 Escolher a primeira janela que satisfaça essa regra e produzir o pack correspondente. Registrar separadamente escala escolhida e maior escala avaliada para confirmação. A regra pode falhar em tarefas entre módulos; esses casos devem poder rejeitá-la, sem exclusão posterior para melhorar resultado.
- [x] 11B.8 Se nenhuma janela for aceita, terminar com motivo explícito e o último candidato válido que caiba no orçamento, indicando lacunas. Se não houver candidato, devolver ausência de evidências/limitação; se nem os metadados mínimos couberem, preservar erro de orçamento explícito. Falhas de integridade e cancelamento mantêm seus canais próprios, sem aparecer como seleção bem-sucedida.

Antes da escolha online, fazer no laboratório uma varredura retrospectiva: existe ao menos um candidato com suporte adequado? Se não, corrigir semente/grafo/seleção/fechamento/orçamento. Se existe, avaliar se a política o reconhece sem gabarito. A varredura conta todo o trabalho e não é reportada como parada antecipada online.

**Interface e compatibilidade**

Interface mínima fixada para esta execução: `acquisition_policy: "spectral_window_v1"` e `scope: "local" | "dependencies" | "callers"`, obrigatório quando a política é fornecida. O modo exige fonte preparada e rejeita contrato, cursor, theta explícito e expansão diferente de `none`. O pack usa `schema_version: "3"`, campos de evidência/proveniência existentes, `next_cursor: null`, `acquisition: {policy, state: "terminal", reason}`, `structural: {status}` e cobertura com lacunas. Cancelamento e falhas de integridade permanecem erros. Baselines com fechamento/materialização iguais são instrumentação interna de laboratório, sem ampliar as políticas públicas.

- [x] 11B.9 Expor um seletor opcional de política em `compass_retrieve` e na CLI, com identificador versionado como `spectral_window_v1`. O nome final deve seguir a convenção do projeto. Ausência da opção conserva schemas 1/2, valores padrão, cursores e seleção anteriores.
- [x] 11B.10 No primeiro modo espectral, exigir fonte preparada e fixar geração; usar consulta/escopo/orçamento, sem contrato manual. Rejeitar combinações contraditórias com `contract`, cursor de paginação ou `theta` fixo explicitamente fornecido. Não reinterpretar esses parâmetros silenciosamente; CLI, schemas MCP e validação Rust devem concordar.
- [x] 11B.11 Definir saída versionada com três dimensões: condição estrutural, término da aquisição e cobertura/lacunas. Devolver um pack terminal com `next_cursor=null`; isso significa fim daquele procedimento, não exaustão do acervo. Não emitir `SUFFICIENT_EVIDENCE_ACQUIRED` por janela espectral. Preservar o significado desse status no caminho atual de obrigações.
- [x] 11B.12 Retornar diagnóstico compacto: política/perfil, identidade da geração/projeção, escopo, escalas escolhida/avaliada, qualidade numérica, motivos de término, limites e trabalho acumulado. Manter trajetória extensa em evidências de laboratório/log; seus custos também contam. O envelope MCP e os limites do consumidor devem comportar a resposta.
- [x] 11B.13 Integrar o modo ao laboratório existente; uma aquisição pública deve bastar para produzir o pack. Chamadas adicionais a `query`, leituras ou varreduras usadas pelo avaliador são instrumentação, contabilizadas separadamente e não ocultadas como aquisição única.

**Aceite:** ao menos um controle demonstra mudança de conteúdo provocada pela escala escolhida; outro termina corretamente com lacunas. Estabilidade térmica não altera a semântica de obrigações. A aquisição é limitada, cancelável e funcional pela CLI e pelo MCP, com testemunhos verificados após a seleção final.

**Entrega e limite funcional:** 11 testes core e dois CLI/MCP novos passaram; os controles demonstram mudança de conteúdo por escala, lacunas, orçamento, cache, cancelamento e conflitos de interface. Massa de 0,8 é medida entre nós não-file, com sementes sempre incluídas. Relações e imports relevantes são conferidos nos bytes de origem. Como o derivado é um cache mutável autoassinado, a aquisição revalida o universo estrutural inteiro contra fontes verificadas, preservando ambiguidades; adulterações reenvelopadas de alvo/testemunho são recusadas. O preflight admite **até 128 documentos e 22.369.621 bytes (~21,3 MiB) na geração inteira**. Acima disso, retorna pack vazio terminal `structural_work_limit`, sem janela ou declaração de cobertura. Esse limite da aquisição não se confunde com os grafos maiores do benchmark numérico. O orçamento usual testado é 16 KiB; nos controles de 3 KiB, os metadados completos não caberam e houve erro explícito.

### 12. Avaliação offline — concluída; política mantida experimental

Medição numérica inicial em build debug: `verification/spectral/numerical-benchmarks.json`, 42 execuções até 16.384 nós e aproximadamente 131 mil arestas, orçamento numérico de 250 ms; 22 completas e 20 limitadas por tempo, todas preservadas no denominador. Maior tempo de processo observado 599,4 ms e RSS 27.140 KiB. Construção de grafo/JSON conta no tempo de processo, separadamente do tempo interno. Esses dados não prometem desempenho para 100 mil nós nem equivalem ao build release.

- [x] 12.1 Comparar calor fixo e adaptativo com mesma consulta, semente, grafo, orçamento, fechamento e materialização. Incluir BFS ou ausência de expansão como alternativa disponível. Separar a regressão do recuperador antigo da comparação que isola a nova escolha de escala.
- [x] 12.2 Medir cobertura de suporte por conjuntos alternativos aceitáveis, atendimento, packs escolhidos por janela sem suporte suficiente, término com lacunas, bytes serializados, escalas calculadas, trabalho, latência e memória. Contar falhas e casos não atendidos nos denominadores. Não esconder omissões somente por nunca emitir um status de suficiência.
- [x] 12.3 Medir curvas de custo em grafos de tamanhos/graus distintos, incluindo isolados e casos com gargalos lentos. Ajustar alcance máximo ao orçamento real. A validade numérica em K6 não justifica extrapolar latência para 100 mil nós.
- [x] 12.4 Congelar perfil, classes de pergunta, orçamentos e metas de atendimento/omissão antes dos reservados. Se não houver meta acordada para promoção, entregar o modo experimental e suas medições; isso não bloqueia o aceite funcional do incremento.
- [x] 12.5 Verificar no consumidor o input real, a transição para análise e a ausência de novas buscas. Separar contribuição da escolha espectral, do encerramento imposto pelo controlador e da consolidação de contexto. Na comparação de custo com ferramentas nativas, usar condição de parada comparável e contabilizar também a interpretação da consulta.
- [ ] 12.6 **Condicional, não acionado nesta execução:** fazer piloto curto com LLM após controles offline, com provedor, destino e teto autorizados. Medir chamadas externas, chamadas LLM, tokens efetivamente reportados (acumulado e maior input), qualidade da resposta e custo. Uma chamada de aquisição não implica uma chamada total à LLM.
- [x] 12.7 Executar revisão independente dos riscos numéricos e de falsa suficiência com o revisor definido em Agentes e modelos, preservando gabaritos e separação de autoria. Executar regressões pertinentes, testes completos Rust, formatação e Clippy ao fechar as mudanças Rust; testar também CLI/MCP e consumidores afetados.
- [x] 12.8 Decidir com evidência: manter experimental, ajustar política/representação, rejeitar a regra testada ou recomendar no domínio avaliado. Instalação/promoção é uma ação posterior aos testes, respeitando destino e autorização da execução.

**Verificações de fechamento:** `scripts/cargo-local.sh test --workspace --locked --offline`, `scripts/cargo-local.sh fmt --all -- --check` e `scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings`; oráculos novos/atuais e controles públicos pertinentes em saídas próprias. As verificações Python devem usar ambiente com dependências confirmadas. Build e tipagem não substituem cobertura do pack, cancelamento ou transição do consumidor.

**Resultado e decisão:** `verification/spectral/evaluation-summary.json` reúne 224 aquisições de desenvolvimento/reservadas, das quais 112 são varreduras retrospectivas adicionais. Perfil, fontes e gabaritos permaneceram congelados antes dos reservados. A 16 KiB, o adaptativo atendeu os 10/10 casos respondíveis, contra 9/10 em theta=1; BFS e calor fixo em theta=256 também atenderam 10/10. A diferença veio de fallback `no_window`, não da regra de janela. As duas janelas aceitas nos 14 casos tinham suporte desde a escala inicial. Não houve janela aceita sem suporte nesses controles; isso não demonstra garantia geral. O adaptativo gastou 1,5055× o tempo acumulado do fixo theta=1 e mais 6.706 bytes JSON agregados. A 3 KiB, 112 pedidos falharam explicitamente por metadados; permaneceram nos denominadores, com custo individual indisponível. Decisão: **manter `spectral_window_v1` experimental e opt-in; não promover como solução geral de suficiência ou economia**. Campanha paga e meta quantitativa de promoção não foram autorizadas/acionadas.

**Fechamento verificado:** 187 testes Rust na suíte completa mais o teste semântico ignorado por padrão, executado separadamente com o modelo já instalado; fmt/Clippy aprovados; 25 grafos/230 escalas no oráculo do build integrado; 74 controles de calor legado; 82 controles públicos de obrigações; nova referência legado de 42 aquisições, quatro contratos e cinco comparações CLI/MCP. A revisão Astra fechou os defeitos de metadados numéricos, integridade do grafo e cobertura de alterações no consumidor com reproduções antes/depois. O teste de sockets Unix foi executado fora da restrição de IPC do sandbox. O release local `d76db898295a21cb0d461c9711e5d3e708c03583c8ee811a058feb875a46c80f` foi instalado preservando o anterior; o launcher real passou em `.compass/mcp/readiness-spectral.json`. Reiniciar a conexão MCP para descobrir a opção nova.

Os binários exatos da avaliação congelada estão preservados em `.compass/spectral-evaluation-build/`; a medição usa o build debug identificado nos relatórios. O build release e um relink posterior dos exemplos são identificados separadamente; não receberam indevidamente as medições do binário congelado.

### Superfícies previstas e continuidade

| Área | Pontos de integração previstos |
|---|---|
| Núcleo numérico | `crates/compass-core/src/diffusion.rs`; um módulo espectral separado se isso mantiver o solver e a política claros. |
| Recuperação e provas | `retrieval.rs`, `acquisition.rs`, acesso às relações de `structure.rs`; reutilizar leitura verificada e orçamento, preservar contratos existentes. |
| CLI/MCP | `crates/compass-cli/src/managed.rs`, `mcp.rs`, schemas, argumentos e testes públicos. |
| Verificação | Testes Rust pertinentes e controles independentes em `verification/`, com gabaritos fora dos acervos e resultados separados dos históricos. |
| Laboratório consumidor | `Multiagentes/src/measurements/`: variantes, resultado terminal, métricas e resposta ao pack. |
| Tarefas consumidoras | `Multiagentes/src/context/{profiles,compass,runtime}.py`, modelos/serviço de tarefa e testes; estender somente o fluxo necessário. |

Os módulos são orientação de integração, não uma lista fechada. O Multiagentes fica fora da raiz gravável ordinária do Compass; a aplicação dos 11 arquivos revisados foi feita com autorização técnica de escrita, hashes prévios e cópias preservadas. A integração foi conferida na árvore aplicada, sem modificar configurações globais ou chamar provedores pagos.

**Continuidade:** incremento funcional, avaliação offline e instalação local concluídos. A próxima decisão de produto deve partir das limitações e custos medidos: a regra de janela não demonstrou benefício próprio; a revalidação estrutural restringe o tamanho de geração. Um piloto pago, mudanças na representação ou ampliação desses limites são trabalho posterior explícito, sem ajuste retroativo aos casos reservados.

## Compatibilidade e limites

CLI: collection prepare/update/status, model install, files/query/neighbors/retrieve. MCP somente leitura: compass_files/query/neighbors/retrieve além search/read. Fontes legado conservadas; semântica opt-in por acervo, expansão default none. Preparação e instalação só CLI. Continuação fixa geração. Watcher, DG, priorização D0/DG, GPU, GUI e serviço remoto ficam posteriores.

Verificação: arquivos vazios/BOM/CRLF/Unicode, links/adulteração/concurrency/cancelamento/disco, restart/revisões; relações e semântica contra controles; calor em dois nós/cadeia/estrela/desconectados/isolados contra referência independente; precisão/cobertura e bytes/tempo/RSS/disco medidos. Sem campanha paga LLM necessária. Nenhuma etapa concluída com verificação essencial pendente.

## Evidência da entrega de 23/10/2026

O fluxo público preparar → buscar → ler → navegar → pack BFS/calor → atualizar → consultar revisões antigas passou na CLI e em cliente MCP independente. Dois acervos de controle separados (desenvolvimento/avaliação), sem árvore viva no fechamento; sem gabarito PostSpark e sem modificar Multiagentes.

- [Resumo verificável](../verification/recovery/summary.json), [fluxos e recursos](../verification/recovery/results.json): 138 testes Rust; teste ignorado por exigir pesos executado separadamente; fmt/Clippy aprovados. Leitura legada: 99 comandos, 6 mutações; protocolo legado: 48 mensagens.
- [Semântica](../verification/semantic/results.json): sete vetores384 contra PyTorch, erro máximo1.518e-7 < tolerância3e-5; hashes do módulo e oráculo identificados. Modelo pinado instalado explicitamente; consultas offline. Pico ~1.2GiB nas cargas pequenas, medido por processo; sem SLO extrapolado.
- [Calor](../verification/recovery/heat-results.json):74 controles por decomposição espectral independente, erro dentro da cauda declarada. [Pesos](../verification/recovery/weights-results.json):4 consultas sintéticas/3políticas, macro P@1=R@1=0.5 em todas; conservar unitários. Nos dois controles de recuperação, BFS e calor recuperaram a relação esperada; nenhuma superioridade geral demonstrada.
- Revisões independentes Sol corrigiram/verificaram descritores incompletos, substituição de ancestral, paginação/quotas, despacho MCP/timeout/perfis, layouts de módulos e colisões de nomes nativos não UTF8. Astra coordenou integração e operador numérico.
- Launcher atualizado e executável anterior preservado; configuração conserva fontes antigas e acrescenta `compass-demo`. Identidade e backup: `.compass/mcp/recovery-installation.json`; verificação real de launcher em outro cwd: `.compass/mcp/readiness-recovery.json`. Reiniciar cliente para descobrir ferramentas novas.

Decisões/limites: índice lexical incremental e vetores reutilizados; grafo recompõe o escopo completo de resolução. Resolução estática limitada, sem macros/typechecker/configs executáveis. Vetores exatos, sem ANN. Store conta bytes lógicos incluindo staging, sem GC automático; derivados limitados128MiB por geração. Memória da inferência não equivale ao orçamento legado de leitura: perfis reduzidos incompatíveis são recusados explicitamente nas tools indexadas. Sem bloqueio real pendente para o escopo autorizado. Próximo trabalho somente após escolha de evolução: avaliação representativa/escala, resolvedores mais abrangentes ou itens explicitamente posteriores.

## Evidências da base preservada

Registros abaixo referem-se aos builds anteriores e não substituem o estado da árvore atual.


## Base e adoção — concluídas

- [x] Incorporar Foundation, sensor D0 e busca literal corrigida como capacidades já entregues.
- [x] Criar plano único; apontar instruções e entradas da documentação para ele.
- [x] Resumir estado operacional; preservar histórico e evidências anteriores.
- [x] Retirar coordenação/OSs do fluxo obrigatório.

Conferência de retomada: 76 testes Rust, 48 controles C01, fmt e 81 artefatos D0 passaram. Release aceito `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8`; produtor dos 41 inputs `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`. Não atribuir esses resultados ao novo build. Git main sem commits; cópias dos arquivos e release anterior preservadas antes das alterações.

As antigas entregas D0 (incluindo verificador, correções e integração), RD-I1 e C01 estão incorporadas à base concluída. A antiga RD-I2 corresponde à etapa 1 abaixo. Não há entrega técnica antiga descartada pela mudança de processo.

## 1. Leitura verificável — concluída

**Resultado:** encontrar um resultado, abrir linhas vizinhas e continuar na mesma revisão, com bytes e origem conferidos.

- [x] Preparar exemplos independentes de linhas/offsets/EOF/continuação e controles de falha.
- [x] Acrescentar operação de leitura ao núcleo com leitor seguro e limites existentes.
- [x] Expor `compass read --snapshot <manifesto> --source <pacote> --path-key <chave> --start-line 1 --max-lines 20 --json`.
- [x] Preservar texto bruto, revisão, offsets e continuação após verificar o objeto inteiro; não ler outros objetos.
- [x] Demonstrar busca → vizinhança → continuação sem árvore viva e com revisões S0/S1 distintas.
- [x] Conferir falhas/limites/cancelamento, regressões, medição e revisão independente.
- [x] Atualizar instruções de uso e registrar evidência atual.

**Conclusão:** fluxo integrado passa; texto não verificado nunca é publicado como válido. Vazio, além EOF, u64 extremo, Unicode/BOM/CRLF, nomes nativos, fonte ausente/adulterada, links/substituição concorrente e limites são cobertos.

**Referência técnica:** [semântica de leitura já definida](entregas/rd/CONTRATO-RD-LEITURA-V0.1.md), reaproveitada sem nova rodada de especificação. `start_line` começa em 1; `max_lines` default 20, máximo 1000; continuação explícita no mesmo snapshot. Disponibilidade da seleção não promete cobertura completa do acervo.

**Progresso:** núcleo/CLI entregues. 85 testes Rust, fmt/Clippy/release passaram; laboratório independente: 8 fixtures, 6 mutações específicas e 99 comandos públicos, com três medidas nativas. [Resultados](../verification/reading/results.json) e [regressões](../verification/reading/regressions.json) identificam o release testado. C01: 48 controles; campanhas RD/Foundation/D0 intactas em cópia passaram. Revisor autorizado não identificou defeitos de produção; reproduziu 99 comandos, sete testes internos e 120 controles aleatórios adicionais. Duas lacunas de prova (promoção falsa de disponibilidade e S0/S1 do mesmo workspace) corrigidas e reverificadas. Primeira tentativa do verificador usou teto inicial de output insuficiente; corrigida a busca da fronteira, sem alterar produção/esperados. Limites: Linux, entradas sintéticas, cancelamento cooperativo, medição sem SLO. Próximo passo: MCP.

## 2. Uso por agentes — MCP — concluída funcionalmente

**Resultado:** um agente executa busca → leitura → continuação pelo MCP e recebe as evidências da CLI.

- [x] Implementar adaptador local por entrada/saída padrão, chamando diretamente o núcleo Rust.
- [x] Expor busca/leitura com revisão e continuação; configurar fontes acessíveis no processo.
- [x] Impedir acesso arbitrário fora das fontes autorizadas e execução de código do acervo; diagnósticos fora do conteúdo.
- [x] Testar chamadas, erros, cancelamento, indisponibilidade e equivalência com CLI.
- [x] Demonstrar o fluxo com cliente stdio de verificação e documentar configuração.
- [x] Preparar instalação local para o Multiagentes: launcher estável, binário fixo, configuração, fontes explícitas e teste de conexão.
- [x] Configurar e demonstrar o fluxo no aplicativo de agente escolhido pelo usuário (rodada externa de 23/10/2026).

**Conclusão:** fluxo real via cliente, sem duplicação do motor. Primeiro MCP usa pacotes explícitos existentes; preparação automática vem na etapa 3. Progresso: adaptador implementado, cliente stdio sintético passou por descoberta, busca/read/continuação, equivalência CLI, restrição de fontes, argumentos, falha de objeto, cancelamento, SIGINT e cap de mensagem. Configuração de um aplicativo de agente depende da escolha solicitada ao usuário; nenhum host foi alterado. protocolo MCP 2025-11-25 por stdio (versão explicitamente negociada), ferramentas somente leitura, fontes nomeadas em configuração local. Transporte limitado implementado com as dependências existentes; não anuncia suporte a HTTP, prompts, recursos, sampling ou extensões. Referências oficiais: [lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle), [tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools), [stdio](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [cancelamento](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation).

**Evidência atual:** [cliente MCP](../verification/mcp/results.json), [leitura no release com MCP](../verification/mcp/reading.json) e [regressões completas](../verification/mcp/regressions.json). Release `6cfd1e8020e9b9c76de75c34dcde0215811be01303ec1d86097e309a0f163dae`. 85 testes Rust, fmt/Clippy/build; cliente com 48 mensagens, 99 comandos de leitura/seis mutações e 48 controles C01 mais campanhas RD/Foundation/D0 passaram. Três leituras nativas da carga de 1000 arquivos leram somente 4096 bytes (reserva 4097), RSS 5840/5892/5780 KiB; medidas locais sem SLO. Os 149 arquivos históricos de pesquisa/verificação continuam idênticos ao início desta execução.

**Integração escolhida em 22/10/2026:** Multiagentes, cujo agente implementará cliente, seleção booleana de ferramentas e ciclo de consultas pelo LLM. O lado Compass está preparado em `.compass/mcp/`, com launcher `scripts/compass-mcp.sh` e [guia de conexão](MCP-INTEGRACAO.md). Fontes `demo` e `compass-source` (projeção congelada de 29 arquivos, identidade própria, não árvore viva). Teste pelo launcher a partir de outro diretório passou; campanha de protocolo com 48 mensagens passou novamente no executável instalado. Evidências locais: `readiness.json`, `protocol-verification.json`, `provisioning.json`. Montagem pontual de pacotes; etapa 3 não implementada.

**Integração externa em 23/10/2026:** as rodadas do Multiagentes demonstraram busca e leitura verificadas via MCP. A última realizou 15 buscas e 3 leituras, sem erro de ferramenta, mas encerrou pelo orçamento do harness antes de uma resposta final. Isso comprova a conexão e o uso, não eficácia de recuperação nem economia de tokens. Rodada encerrada pelo usuário.


## Incremento de consulta normalizada — 29/10/2026

Implementada a extensão opt-in `query_intent` em Rust, CLI e MCP, integrada ao novo
modo `normalized_context_v1` de Tarefas no Multiagentes. A normalização curta recebe
somente pedido e escopo autorizado; o Compass descobre sementes, explora relações
e entrega um único pacote verificado à etapa de resposta. Correspondência exata,
qualificação por caminho, homônimos, ausência e múltiplos alvos têm resultados explícitos.
Chamadas e registros anteriores permanecem compatíveis. Contrato público descrito
em `MCP-INTEGRACAO.md`, seção `query_intent`.

Verificação: 187 testes Rust e Clippy com warnings negados; 23 controles novos do
Multiagentes com MCP/CLI reais e LLMs simuladas, além das regressões de Tarefas,
retomada, aquisição terminal e interface. `renderChip` é a única semente do seu
controle, e `paintBadge` é recuperada pelo fechamento estrutural sem entrar na
entrada da normalização. Controle com `finish_order`/`record_total` confirma que
não existe regra específica para os nomes da regressão. Atribuição por intervalos
não atribui à trajetória código já coberto pelo fechamento.

O script local `scripts/prepare_normalized_context.py` no Multiagentes registra uma
fonte isolada e fixa o build experimental sem substituir o executável usado pelos
fluxos existentes. `scripts/eval_normalized_context.py` registra entradas, parâmetros,
sementes, pacote, cobertura, ruído, bytes e latência em diretórios novos por execução.
Não houve chamada a provedor real nem medição de economia total ou qualidade da
resposta de uma LLM real. A hipótese vetorial permanece fora deste incremento.
