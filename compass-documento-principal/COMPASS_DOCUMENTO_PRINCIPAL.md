---
title: "Compass D0"
subtitle: "Documento principal de produto, pesquisa e engenharia"
author: "Idealização e direção: Emanuel Junior | Consolidação: ChatGPT"
date: "14 de outubro de 2026 • Edição 1.0"
lang: pt-BR
---

# Identidade, alcance e leitura deste documento

**Projeto:** Compass D0, historicamente também chamado Bússola SGC.  
**Direção vigente:** reconstrução integral em repositório independente.  
**Linguagem do motor:** Rust.  
**Laboratório científico:** Python, separado da produção.  
**Primeiro marco:** Compass Foundation — estados, revisões, snapshots e comparação determinística.  
**Estado da edição:** consolidação para revisão do idealizador; preserva as decisões tomadas e sinaliza o que continua pendente. Não certifica uma implementação nova nem autoriza, por si, executar alterações no projeto.

> **Finalidade:** tornar possível compreender o projeto, localizar a justificativa de cada decisão e iniciar uma implementação fiel aos contratos, sem reconstruir a história por meio de documentos contraditórios.

Este é o documento principal da nova construção, não outro plano de refatoração do aplicativo antigo. Reúne a origem da ideia, a evolução do escopo, o que aprendemos com o legado, a visão de produto, as decisões tecnológicas, a matemática discutida, as hipóteses de pesquisa, o método de validação e os limites da primeira entrega.

A consolidação utiliza a conversa disponível, a recuperação de contexto das discussões iniciais, o plano de 23/09/2026, o whitepaper de 11/10/2026, o caderno de investigação de 138 páginas e o relatório da Missão 00 de 14/10/2026. Os materiais têm autoridades diferentes. A conversa posterior que determina uma reconstrução do zero **supera a orientação histórica de preservar e refatorar o código existente**. Isso não altera retroativamente o que o plano antigo afirmava. [S-CONV; S-HIST; S-P23; S-WP; S-CAD; S-R00]

## Como distinguir uma decisão de uma hipótese

| Marca | Significado e autoridade |
|---|---|
| **VIGENTE** | Direção escolhida na conversa para o novo projeto. Uma alteração requer decisão explícita. |
| **PROPOSTA** | Forma de concretizar a direção, ainda não homologada em contrato detalhado. Não é licença para implementar silenciosamente. |
| **HIPÓTESE** | Afirmação de utilidade ou mecanismo que precisa de investigação. Pode ser limitada ou rejeitada. |
| **HISTÓRICO** | Decisão, código, parâmetro ou resultado de uma versão anterior. Não entra automaticamente no produto novo. |
| **RELATADO** | Informação de uma fonte inspecionada; não equivale a reprodução independente nesta consolidação. |
| **PENDENTE** | Informação, escolha ou verificação ainda ausente. Ausência não será convertida em resultado favorável nem em defeito. |

Uma mesma linha de trabalho pode ser matematicamente definida, estar implementada no legado e continuar sem eficácia demonstrada. Essas dimensões não formam uma única escala automática de aprovação.

**Fronteira de completude.** A matriz de cobertura no final relaciona os temas e decisões identificados às suas seções. Ela é a conferência explícita contra omissões. Não seria honesto garantir a reprodução de toda conversa ou arquivo que nunca foi disponibilizado. Onde a origem é apenas uma lembrança recuperada, e não um artefato primário, isso aparece indicado. Onde não há contrato, a lacuna permanece visível em vez de receber uma solução inventada.

**Leitura recomendada.** Para direção e implementação inicial, ler as seções 1–8, 13–17 e 20. Para revisar a pesquisa, acrescentar as seções 9–12 e os apêndices. Para conferir a trajetória e os resultados anteriores, usar as seções 2–4 e o apêndice histórico. O corpo é autossuficiente para entender as decisões; verificar arquivos de código e recibos históricos ainda exige suas fontes originais.

# 1. A decisão essencial

## 1.1 O produto que queremos

> **O Compass D0 é uma infraestrutura local de inteligência de contexto para agentes de IA, capaz de localizar evidências relevantes em grandes bases de código, considerar suas relações estruturais e manter autonomamente suas representações atualizadas, com eficiência, rastreabilidade e garantias explícitas de confiabilidade.**

A formulação foi construída e aceita na conversa. A preocupação com consumo acrescentou uma exigência: operar continuamente ou sob demanda sem comprometer o ambiente em que o desenvolvedor e seus agentes trabalham. “Hiperpreciso” é a ambição; não é uma garantia universal já demonstrada. [S-CONV]

A unidade de valor não é um arquivo aquecido, um escore elevado ou uma curva bonita. É **uma evidência atual, verificável e útil para a tarefa**, entregue com custo controlado. Encontrar rapidamente o arquivo inicial é insuficiente quando o problema depende de funções, configurações, contratos e testes distribuídos.

## 1.2 O que está decidido

A nova implementação nascerá em um repositório fisicamente separado do legado. Não haverá migração automática de código, testes, `AGENTS.md`, skills, perfis, roadmaps ou decisões antigas. O legado será preservado como laboratório histórico, não usado como autoridade operacional do novo motor.

Rust é a linguagem escolhida para o motor de produção. Python existe somente no laboratório de verificação e experimentação. Não serão dois servidores Compass concorrentes. A primeira interface será uma CLI; MCP será acrescentado depois, chamando o mesmo núcleo. A linguagem foi encerrada como decisão arquitetural, não como resultado de benchmark nem prova de correção científica.

O produto terá responsabilidades distintas de aquisição de evidências, manutenção das representações e consistência/proveniência. D0 e DG poderão orientar a manutenção; não terão autoridade para transformar um derivado conhecido como obsoleto em informação atual.

O primeiro marco é observar e comparar estados com identidade e rastreabilidade. Embeddings, grafo, difusão, MCP, interface gráfica e serviço multiusuário não fazem parte dessa primeira entrega. A fórmula definitiva de D0 não será fixada antes do contrato dos estados. [S-CONV]

## 1.3 O que não está decidido

Não estão homologados o schema final de snapshots, a semântica de leitura concorrente, as linguagens inicialmente analisadas, as bibliotecas de grafo e álgebra linear, o modelo de embeddings, a política de atualização nem os parâmetros de difusão. SQLite e Tree-sitter são candidatos prioritários, não implementações automaticamente aprovadas.

A difusão continua como hipótese de ordenação e expansão complementar. O operador final — calor livre, reinjeção, outra propagação ou ausência de difusão — precisa de contrato e comparação apropriada. Condutâncias diferenciadas são uma hipótese registrada, não uma funcionalidade já autorizada.

## 1.4 A regra que não pode ser violada

> **A implementação não pode definir sozinha o que significa estar correta.**

O contrato define a operação pretendida; exemplos independentes e oráculos conferem sua realização; o experimento mede sua utilidade. Se o contrato prevê 2 e o programa devolve 9, não se ajusta o gabarito para devolver 9. Se o contrato estiver errado, sua revisão é uma decisão científica explícita, com nova versão.

# 2. Origem e evolução do Compass

## 2.1 A intuição inicial: o mapa envelhece

A ideia original era comparar a base anteriormente representada para recuperação, $S_0$, com o estado posterior do repositório, $S_t$, e produzir um indicador $D_0$ que ajudasse a decidir quando atualizar o RAG. A expressão “diferença entre massas de dados” era uma intuição sobre a magnitude das mudanças, não uma autorização para comparar apenas o tamanho total dos diretórios. Dois estados podem ter o mesmo número de bytes e conteúdos completamente diferentes. [S-HIST; S-CAD, caps. 1–2]

O raciocínio inicial pode ser conservado sem impor uma fórmula final:

$$
\Delta_t=\Delta(S_0,S_t),\qquad D_0=f(\Delta_t;\text{representação e convenções}).
$$

$\Delta_t$ é uma descrição das diferenças; $D_0$ é um resumo dessas diferenças sob regras escolhidas. $S_t=S_0+\Delta_t$ só pode ser usado como metáfora ou operação explicitamente definida de aplicação de mudanças. Um repositório não é automaticamente um vetor no qual se faz adição comum.

Na discussão inicial, o RAG foi tratado como bússola para localizar informação; o código observado, a leitura direta e mecanismos como busca textual ou resolução de símbolos eram fontes de conferência. Diffs podiam medir alterações do código sem exigir reconstruir todos os chunks a cada observação. Um ledger serviria como registro/cache de observações, não como prova automática de verdade. [S-HIST]

## 2.2 A ampliação para recuperação de contexto

O idealizador identificou uma dependência prática: um mapa atualizado não ajuda se o agente não consegue recuperar o que precisa. A preocupação foi expressa como “se não tiver a informação completa, não adianta pesquisar; e, se não pesquisar, não tem D0”. A evolução para grafos e recuperação não foi um desvio arbitrário, mas uma tentativa de resolver esse problema instrumental. [S-CONV]

A revisão posterior tornou a dependência mais precisa. **Calcular um D0 textual entre snapshots não exige recuperação topológica. Avaliar se esse sensor antecipa perda de qualidade de contexto exige um sistema de recuperação e uma medida externa dessa qualidade.** Completude da indexação, atualidade das representações e qualidade da recuperação são problemas diferentes. O caderno identifica a independência do cálculo textual em relação ao grafo. [S-CAD, pp. 14–16]

Essa distinção evita dois extremos: abandonar recuperação como se fosse irrelevante, ou bloquear indefinidamente a hipótese original até construir um recuperador sofisticado.

## 2.3 O que foi surgindo depois

Foram discutidos representação dual textual/vetorial e estrutural, sensores D0/DG, diferenças entre arestas e espectro, laplacianos, energia de Dirichlet, difusão de relevância, sementes, fronteiras, empacotamento e experimentos com agentes. Também surgiram ideias exploratórias de novelty, homotopia/deformação e Langevin. Nem toda ideia chegou a um contrato; nem todo contrato correspondeu ao caminho público executado. [S-HIST; S-P23; S-R00]

Em setembro, o plano histórico passou a privilegiar aquisição progressiva, âncoras, difusão por canal, observação de energia e seleção de uma janela útil. Em outubro, a documentação registrou coexistência de calor livre, reinício discreto e equilíbrio com reinjeção. A revisão dos resultados mostrou que “calor” não identificava uma intervenção única. [S-P23, §§1–9; S-R00, §§3–4]

## 2.4 A mudança decisiva de processo

O projeto acumulou implementações, correções, documentos, testes e agentes, sem correspondência suficientemente clara entre intenção científica e comportamento executado. A dificuldade deixou de ser apenas construir o algoritmo e passou a ser saber **qual algoritmo estava sendo avaliado**.

O idealizador propôs estudar a teoria e o código em papel. Foi produzido o caderno autossuficiente, com matemática gradual, trechos reais, exercícios e auditoria. Depois, a Missão 00 inventariou o estado local sem executar o produto. Finalmente, a discussão sobre refatoração foi encerrada em favor da reconstrução integral. Não se reinicia a pesquisa; reinicia-se o software com fronteiras mais claras. [S-CONV; S-CAD; S-R00]

## 2.5 Registro da trajetória, sem cronologia inventada

| Período/fonte | Movimento principal | O que continua válido |
|---|---|---|
| Agosto, contexto recuperado | $S_0$, $S_t$, diffs, D0 e atualização de RAG | O problema original; fórmulas iniciais são candidatas históricas. |
| Setembro, discussão e plano de 23/09 | Evidence/Drift Engines, espectro e aquisição progressiva | Separação de responsabilidades e hipóteses; o plano de refatoração foi superado. |
| Início de outubro, caderno | C1–C6 e caminhos matemáticos diferentes | Evidências delimitadas, não uma escada de aprovação do produto. |
| 11/10, whitepaper e sua auditoria | Campanha PostSpark e questionamento das conclusões | Dados históricos úteis, com limitações e correções explícitas. |
| 14/10, Missão 00 | Inventário estático e identidade da árvore local | Base histórica de rastreabilidade; snapshot integral ainda proposto. |
| Conversa posterior | Reconstrução limpa, produto local e escolha de Rust | Direção vigente da nova construção. |

# 3. Lições do legado e fronteira da reconstrução

## 3.1 O diagnóstico central

A dificuldade não foi simplesmente “usar uma linguagem inadequada”. Houve mistura de cinco níveis: problema do usuário, hipótese científica, formulação matemática, implementação e avaliação. Um resultado ruim podia vir de uma hipótese limitada, de um algoritmo errado, de uma integração diferente ou de um instrumento de medição inadequado. Tratar tudo como bug de software produzia novas correções sem uma conclusão confiável. [S-CONV]

A analogia $1+1=2$ versus uma implementação que retorna 9 descreve duas falhas possíveis. Na primeira, o código viola um contrato correto e o teste reproduz o erro. Na segunda, código e testes realizam corretamente uma operação diferente daquela que se pretendia investigar. A reconstrução precisa detectar ambas.

## 3.2 O que a Missão 00 efetivamente é

O relatório de 14/10/2026 é um levantamento estático. Não executou testes, builds, consultas, indexações, solver, servidores, chamadas externas nem abriu o banco do produto. Suas categorias distinguem confirmação estática, intenção documental, teste histórico identificado, alegação experimental e desconhecimento. **Esta consolidação não transforma essas categorias em uma auditoria funcional nova.** [S-R00, §§1 e 7]

O objeto era a árvore local sobre HEAD `231d0917d8a3dc669d8b021808302f69bcfdda45`, cujo assunto era “antes da refatoração”. O relatório registra 1.529 entradas rastreadas, 998 arquivos/links não rastreados e módulos centrais fora do commit. As 73 entradas `??` da saída compacta incluíam diretórios e não eram 73 arquivos. [S-R00, §2]

Foram conferidos 611 arquivos de um manifesto de 11/10, sem divergências de hash. O inventário da missão reuniu 2.666 caminhos: 2.398 arquivos, 51 links e 217 ausências. Houve reconferência dos 2.449 hashes de arquivos/links dentro desse perímetro. Isso identifica bytes; não prova correção matemática, completude ambiental, consistência atômica nem backup integral. [S-R00, §§2.1 e 7; ap. C]

## 3.3 Os achados que não podem desaparecer

| Achado relatado | Consequência para o novo projeto |
|---|---|
| `compass_query`, `context_query` e servidor Python são caminhos distintos | Toda execução deve declarar interface, versão e configuração efetiva. |
| Seis perfis selecionáveis na interface anterior | Não expor alternativas quase homônimas como se fossem um motor único. |
| DG diferente em TypeScript e Python | Nomes não substituem contratos; fórmula e identidade precisam viajar com a medição. |
| Calor livre descrito para um tratamento que chamou reinjeção | Vincular a afirmação científica ao operador realmente executado. |
| `mode: hybrid` pode aparecer com vetor indisponível | Registrar canais efetivamente utilizados e fallback. |
| `enable_heat` omitido/false não chama solver v3 | Disponibilidade de algoritmo não comprova participação no experimento. |
| Sensores Python não encontrados no fluxo do servidor | Presença do módulo não demonstra integração operacional. |
| Supervisor pode traduzir sinal inesperado em saída zero | Término do processo, conclusão da operação e sucesso da tarefa são estados distintos. |
| Segundo turno incompleto registrado como `completed` | Confrontar estado local com recibo primário; preservar resultados parciais. |
| Lease pode ser retomado pelo mesmo PID vivo no cenário identificado | Concorrência exige protocolo e testes próprios, não apenas nomes de locks. |
| Calor muda também apresentação/diagnósticos da resposta | Uma ablação de ranking precisa controlar o envelope entregue. |
| Parte do arquivo histórico é link para conteúdo vivo | Um caminho “legacy” não é necessariamente uma cópia histórica imutável. |

Fontes: [S-R00, §§3–6]. O relatório localiza código e, em alguns casos, provas históricas. Não determina frequência atual de falhas nem demonstra que todas ocorreram em cada campanha.

## 3.4 D0 e a consulta principal

O levantamento da v3 encontra reconciliação de arquivos salvos, busca por símbolos, difusão opcional e leitura atual. A política D0/DG anterior não aparece como gate escalar geral de toda consulta v3. Isso **não prova que a reconciliação está errada ou que D0 é inútil**. Mostra que um bom resultado da v3 não pode ser atribuído automaticamente à hipótese original de atualização orientada por drift. [S-R00, §§3–4]

A interpretação preservada é: a contribuição operacional de D0/DG ainda precisa de um experimento próprio. Não se descartam resultados reais de componentes; não se somam esses resultados para anunciar validação de uma arquitetura integrada diferente.

## 3.5 Arquivamento: uma operação, não outra reforma

**VIGENTE:** encerrar o desenvolvimento do legado e preservar seu estado local antes de descartá-lo ou modificá-lo. A cópia deve incluir o que for necessário para reproduzir código e evidências, inclusive arquivos pertinentes não rastreados e ignorados. O procedimento da Missão 00 ainda estava apenas proposto; não há confirmação posterior, nas fontes desta edição, de que o snapshot integral foi realizado. [S-R00, §2.2]

A preservação deve respeitar privacidade, permissões, links e consistência de bancos. Não se presume autorização para copiar credenciais, interromper serviços, seguir links externos ou transmitir arquivos. Não executar limpeza, reset, reorganização ou correção como parte do arquivamento. Guardar uma cópia verificável e trabalhar no novo repositório; não transformar a operação em obrigação de sanear o antigo.

**Proibição de transferência automática:** o novo projeto não copia o código antigo nem o usa como oráculo. Conhecimento, exemplos matemáticos e dados históricos podem ser consultados de modo pontual e explícito, sem reativar instruções antigas ou chamar dados conhecidos de holdout.

# 4. O que os experimentos anteriores permitem afirmar

## 4.1 Quatro níveis de evidência

É necessário separar: conformidade matemática, funcionamento da integração, aquisição de evidências e benefício para o agente. Um solver pode calcular corretamente e ordenar contexto inadequado. Um recuperador pode entregar bytes autênticos e ainda omitir um fato indispensável. Um agente pode terminar o processo sem produzir uma correção funcional.

As campanhas antigas são preservadas pelo que mediram. Nenhuma será “zerada” porque apresentou problemas, nem promovida a prova de eficácia completa. Os números abaixo são **relatados pelas fontes**, salvo as conferências aritméticas explicitamente identificadas nos apêndices. Não foram repetidas chamadas de modelos para esta edição. [S-CAD, caps. 16–17; S-R00, §6]

## 4.2 C1–C6: perguntas diferentes, não milhares de réplicas da mesma tese

| Experimento histórico | Resultado/alcance relatado | Limite que deve acompanhar a citação |
|---|---|---|
| C1, lei do platô | 240 grafos, 960 soluções; controle de clique expõe incompatibilidade do corte relativo à semente | Igualdade dos nós não-semente não implica passar o corte de metade do máximo. |
| C2, mapa de $\beta$ | 61 valores em um componente real e controle sintético | Uma varredura local e um agrupamento por diretório não calibram política universal. |
| C3, barris | Alteração de top-20 dependente da semente ao remover/reponderar barris | Não reproduz integralmente as consultas antigas com outra dinâmica. |
| C4, ordem certificada | 4.170 comparações em 35 casos, zero violações observadas | PCG independente do runtime; não comprova integração nem relevância. |
| C5, testemunhas plantadas | Benefício dentro do módulo e perda no módulo vizinho; 127.360 registros | Unidade artificial por visita não inclui solver; a distribuição da testemunha muda o resultado. |
| C6, tempo global | 987 medições mapeadas a apenas dois pares origem/destino distintos | Sete nomes de fluxo não significam sete problemas independentes; top-5 e top-10 mudam o veredito. |

Fonte: [S-CAD, pp. 78–82]. Os rótulos C1–C6 também apareceram em listas antigas de correções, que não são os mesmos experimentos. O novo registro utilizará identificadores não ambíguos.

## 4.3 Setembro: tokens brutos não são preço

O plano de 23/09 relata seis tarefas pareadas, 12 sessões, 555.078 tokens no Off e 628.852 no Heat. A diferença bruta, 73.774, convive com menos entrada sem cache e mais entrada em cache no Heat. A rodada utilizou acesso por autenticação ChatGPT, sem uma fatura por chamada que permitisse converter esses números em custo financeiro do experimento. Shell estava disponível nos dois braços. [S-P23, §2]

Esse resultado não provou economia, equivalência estatística nem oversmoothing como causa do excesso. Nos pares com evidências iniciais iguais, custos de sessão ainda diferiam. A lição é medir o que foi selecionado, entregue, reenviado, observado pelo provedor e efetivamente usado, sem inferir um desses estados apenas do outro.

## 4.4 Já Comprei e diferenças de protocolo

A campanha relatada de Já Comprei teve 36 sessões e 165 requisições. A cobertura estrita B/C0/C1 foi 100%/75%/75%; uma sensibilidade pós-hoc de bordas elevou os Compass a 100%. Foram identificadas 36 chamadas iniciais sem histórico e 129 com histórico acumulado. O resultado descreve o loop executado, não o desenho pretendido de chamadas independentes. [S-CAD, pp. 81–82; S-R00, §6.2]

Não se transfere esse defeito automaticamente para PostSpark: a auditoria deste último reconstruiu um segundo turno limpo. Inversamente, a limpeza do segundo turno não resolve todas as outras limitações de PostSpark.

## 4.5 O whitepaper de 11/10

A campanha PostSpark 3 relata 12 tarefas, três braços, duas repetições, 72 sessões e 144 requisições, sobre 313 arquivos. B utiliza busca/leitura convencional; C0 e C1 utilizam Compass sem/com ativação do calor. “Grafo estático” é o nome publicado de C0, **não uma comprovação de que ele executava uma BFS ou expansão estrutural equivalente**. [S-WP, §§3–5; S-R00, §4]

| Medida | B | C0 | C1 |
|---|---:|---:|---:|
| Cobertura média de critérios | 22,2% | 44,4% | 47,2% |
| Tokens de ferramenta médios, auditados no caderno | 1.402,21 | 2.309,92 | 1.740,58 |
| Entrada média publicada | 2.502,3 | 2.617,9 | 2.175,2 |
| Entrada média recalculada no caderno | 2.223,25 | 2.338,83 | 1.896,13 |
| Saída média publicada | 1.258,8 | 1.234,0 | 1.168,5 |
| Saída média recalculada no caderno | 1.639,25 | 1.614,50 | 1.548,92 |
| Custo de inferência relatado, USD | 0,0213168704 | 0,0201868352 | 0,0188378456 |

Fonte: [S-CAD, pp. 83–88]. Os totais absolutos publicados de entrada e saída divergiam dos recibos segundo a auditoria. Esta tabela conserva os dois registros, sem uma correção silenciosa. O custo somado relatado é US$ 0,0603415512; não inclui o custo operacional completo do Compass.

O contraste C1−B foi +25,0 pontos percentuais, com IC de 95% de −13,89 a +63,89. O contraste C1−C0 foi +2,78 pontos percentuais, com intervalo de 0 a +8,33, concentrado em T01. C1 venceu B em seis tarefas, perdeu em quatro e empatou em duas; teve cobertura zero em seis tarefas. A tabela por tarefa está preservada no apêndice A. [S-CAD, pp. 84–86]

O whitepaper descrevia $e^{-\alpha tL}h_0$, enquanto o runtime identificado usava $(L+\beta I)h=\beta s$. Os 72 encerramentos do executor incluíam nove sínteses `incomplete/max_output_tokens`: duas em B, três em C0, quatro em C1. O caderno distingue 72 sessões encerradas de 63 respostas finais marcadas completas pelo provedor. Essas interrupções não apagam automaticamente evidências adquiridas antes da síntese. [S-CAD, pp. 87–88]

Há ainda uma limitação importante do baseline: uma rodada única de ferramentas impede o agente de pesquisar, ler os resultados e decidir uma segunda leitura. Compass agrega descoberta e leitura em uma operação. O ensaio mede interfaces sob esse regime, não a melhor capacidade de busca convencional iterativa. Tarefas bem-sucedidas também continham identificadores explícitos; não sustentam a alegação de que ripgrep é incapaz de localizar aqueles nomes. [S-CAD, pp. 84–85]

**Conclusão preservada:** existe sinal exploratório e evidência localizável, mas não uma demonstração geral de superioridade, redução de ruído, suficiência semântica ou economia integral. A reserva efetiva do holdout e a revisão independente das respostas continuam questões a qualificar. Isso é motivo para melhorar a investigação, não prova de inutilidade do projeto.

# 5. Contrato do produto e limites de escopo

## 5.1 Encontrar, manter, otimizar e verificar

O Compass deve localizar evidências para uma intenção, considerar relações relevantes e entregar conteúdo com origem e revisão. Deve acompanhar mudanças e reconstruir apenas o necessário, em vez de usar reindexação integral como resposta automática. Deve controlar tokens, latência, CPU, RAM e disco. Deve expor quando uma fonte ou relação não pôde ser verificada.

Essas quatro responsabilidades são complementares. Precisão alta não garante recall alto. Recall de linhas não garante suficiência causal. Hash de conteúdo não garante relevância. Atualidade na leitura não garante que o arquivo não mude depois dela. O contrato precisa declarar o instante ou snapshot ao qual a evidência corresponde.

## 5.2 Cenários de valor

| Situação do usuário/agente | Entrega esperada |
|---|---|
| Encontrar uma função nomeada | Declaração, localização e revisão, sem ser deslocada por um vizinho apenas “quente”. |
| Diagnosticar um bug entre módulos | Evidências e relações necessárias, com limites da resolução explicitados. |
| Alterar um contrato público pequeno | Invalidar os derivados afetados; D0 global baixo não mascara a mudança. |
| Consultar durante atualização | Entregar uma visão verificável ou declarar parcialidade/indisponibilidade. |
| Retomar após desligamento | Reconciliar mudanças não observadas; não confiar cegamente no watcher. |
| Trabalhar com repositório grande | Respeitar orçamento de recursos, sem truncamento sem aviso. |
| Encontrar uma referência fora da primeira página | Permitir acesso/continuação conforme contrato e revisão válida. |
| Consultar um grafo incompleto | Distinguir relação não encontrada, não resolvida e realmente ausente. |

Esses são requisitos de comportamento. O mecanismo particular para realizá-los depende dos contratos e das decisões de implementação. [S-CONV]

## 5.3 O que o produto não pretende garantir

Não há promessa de compreender perfeitamente qualquer programa, descobrir todas as dependências dinâmicas, decidir sozinho a verdade de uma regra de negócio ou produzir patches corretos. A correção de tarefas continua sendo um desfecho a avaliar com o agente. “Contexto suficiente” é objetivo e métrica em tarefas com referência conhecida; não uma certeza universal emitida a partir de um escore.

O Compass não começa como IDE, agente implementador, gerenciador de equipes de IA, plataforma multiagente, banco vetorial universal ou serviço comercial completo. Não precisa de geração adicional por LLM para calcular D0 ou resolver um sistema linear. Uma eventual camada generativa futura exigirá justificativa de utilidade, custo, privacidade e avaliação própria.

## 5.4 Local primeiro, servidor sem duplicar o motor

A direção é infraestrutura persistente de contexto, mas persistência de índices não exige processo pesado sempre ativo. Foram propostos dois modos do mesmo núcleo: operação sob demanda, com reconciliação necessária, e observação contínua leve para antecipar trabalho. O servidor poderá usar o mesmo núcleo com políticas de concorrência e cache diferentes. Os detalhes desses modos ainda precisam de contrato. [S-CONV]

Multiplataforma é objetivo de distribuição. Uma matriz concreta de sistemas, arquiteturas, instaladores e máquinas de referência permanece pendente. A máquina do idealizador é útil para desenvolvimento, mas não foi escolhida como representante de todos os clientes.

# 6. Arquitetura lógica e fluxos de informação

## 6.1 Três responsabilidades; não três aplicativos

| Responsabilidade lógica | Deve conhecer | Não deve assumir |
|---|---|---|
| **Evidence Engine** | Consulta, fontes, candidatos, relações, revisões e orçamento | Que score elevado representa verdade ou que ausência de resultado significa ausência de código. |
| **Maintenance Engine** | Mudanças, dependências de derivados, custos, D0/DG e prioridade | Que drift baixo torna um derivado conhecido como obsoleto atual. |
| **Consistência e proveniência** | Identidade do acervo, revisão, validade, publicação e recibos | Que o grafo recupera tudo ou que o solver certifica a semântica. |

São fronteiras lógicas, não uma obrigação de criar microserviços, três bancos ou múltiplos processos. No início, uma biblioteca modular e uma CLI bastam. [S-CONV]

## 6.2 Fluxo de construção das representações

O ponto de partida é observar conteúdo elegível no repositório. A observação produz identidades e revisões. Parsing e resolução extraem unidades e relações quando houver suporte. Índices lexicais, vetores e projeções estruturais são derivados associados às revisões e à configuração que os produziu. Uma publicação torna uma geração disponível sob um contrato de consistência.

**Não existe uma cadeia obrigatória “repositório → embeddings → grafo → laplaciano → Dirichlet → D0”.** Lexical, vetorial e estrutural são representações complementares. O grafo factual pode nascer do código analisado, não da proximidade entre embeddings. O laplaciano é um operador sobre uma projeção do grafo. Dirichlet é uma energia/interpretação matemática, não um arquivo intermediário. D0 pode comparar snapshots diretamente.

## 6.3 Fluxo de consulta

A sequência lógica discutida é: validar a intenção e o estado; localizar candidatos exatos, lexicais e vetoriais conforme disponibilidade; escolher sementes quando uma estratégia estrutural as exigir; recuperar ou reordenar por relações; selecionar evidências sob orçamento; ler/verificar conteúdo; entregar trechos e possibilidade de continuidade; registrar diagnósticos fora do conteúdo principal enviado ao modelo.

Esse fluxo preserva recuperação tradicional. A difusão, quando contratada e habilitada, é complementar. Não substitui uma busca exata válida nem precisa entrar em toda consulta. O ranking deverá distinguir candidatos encontrados diretamente dos acrescentados pela expansão, sem considerar a origem topológica um bônus incondicional de verdade.

Uma implementação de BFS com desempate e uma implementação de difusão são estratégias comparáveis; “desligar calor” não define, sozinho, uma BFS. A identificação de cada braço será feita pelo algoritmo real.

## 6.4 Fluxo de manutenção

Detectar mudança, invalidar dependentes, priorizar recomputação e publicar novos derivados são operações diferentes. D0/DG entram como informação para a política de manutenção, ao lado de custo, abrangência da alteração e necessidade da consulta. Recomputar pode ser adiado; declarar atual um derivado sabidamente obsoleto, não.

A detecção por eventos ajuda a reduzir latência, mas o estado observado do acervo continua sendo a referência. Uma retomada deve poder reconciliar mudanças perdidas. A política detalhada de invalidação imediata, leitura viva, geração estável, overlay ou atualização sob demanda será decidida explicitamente; o comportamento do legado não é copiado como obrigação. [S-CONV]

## 6.5 Um exemplo comum aos componentes

Considere `Carrinho`, `calcularTotal`, `aplicarDesconto`, `estoque` e um arquivo de preços. A consulta menciona um total incorreto. A busca direta localiza `calcularTotal`; relações podem indicar quem a chama ou quem fornece desconto. Um embedding pode localizar o tópico mesmo sem o nome literal. O calor pode atribuir valores graduais a candidatos conectados. A seleção final continua escolhendo trechos discretos.

Se o preço mudar, D0 pode indicar divergência textual enquanto DG permanece igual. Uma revisão antiga do embedding deve ser invalidada ou claramente tratada como histórica. O empacotador precisa entregar bytes da revisão declarada. Nenhuma dessas operações isoladamente prova que o bug está resolvido. O exemplo é didático, não um resultado do Compass novo.

# 7. Estados, revisões e consistência

## 7.1 A separação conceitual já acordada

**VIGENTE:** estado observado do código e estado das representações indexadas são objetos diferentes. Um commit não identifica necessariamente a árvore local; um arquivo salvo pode ser mais recente que seu vetor; uma relação pode depender de dois arquivos e de uma configuração do resolvedor. Um número D0 sem identificação dos estados comparados perde seu significado operacional.

Para evitar ambiguidade, esta consolidação usa a notação abaixo como **PROPOSTA editorial para o contrato Foundation**, não como schema já implementado:

| Símbolo/termo | Significado proposto |
|---|---|
| $R_t$ | Estado do acervo observado sob uma política e uma janela de observação. |
| $S_0$ | Snapshot de referência explicitamente identificado para uma comparação. |
| $S_t$ | Snapshot observado que será comparado a $S_0$. |
| $I_g$ | Geração publicada de representações derivadas, vinculada a fontes e configurações. |
| $A_r$ | Artefato derivado específico, associado a revisão e versão do produtor. |
| $\Delta(S_0,S_t)$ | Conjunto de diferenças calculado sob convenções versionadas. |

A introdução de $R_t$, $I_g$ e $A_r$ apenas torna a separação legível. Sua adoção exata, campos e ciclo de vida são pendências do contrato.

## 7.2 Perguntas que o contrato precisa fechar

Quais arquivos são elegíveis? Como distinguir ignorado, vazio, removido, inacessível e não observado? O que um caminho identifica e o que um hash identifica? Como tratar renomeações, symlinks, case sensitivity, codificações e fins de linha? Uma leitura aceita mistura temporal de arquivos ou exige snapshot consistente? O que fazer se o arquivo muda durante a própria leitura?

Essas perguntas não serão respondidas implicitamente pelo comportamento conveniente de uma biblioteca. Em especial, a varredura de uma árvore viva não deve ser anunciada como snapshot atômico apenas porque cada arquivo recebeu um hash. Uma mudança após uma leitura válida exige uma semântica de validade temporal, não uma garantia impossível de que o mundo não mudou.

## 7.3 Estados de disponibilidade não são valores numéricos

Um cálculo indisponível não é zero. Um arquivo inacessível não é removido. Um índice parcialmente disponível não é índice completo. Um resultado sem matches não é uma prova de inexistência. Essa distinção já orienta o produto; a forma de modelá-la em Rust será estabelecida no contrato.

Exemplo meramente didático de modelagem, não código de produção nem schema aprovado:

```rust
enum Measurement<T> {
    Available(T),
    Partial { value: Option<T>, reason: String },
    Unavailable { reason: String },
}
```

O enum não substitui validação de domínio, valores finitos, consistência das revisões ou tratamento de erro. Também não atribui significado semântico a um `f64` só por estar dentro de um tipo.

## 7.4 D0 não é idade cronológica nem atividade acumulada

Se A muda para B e depois retorna exatamente a A, o drift contra o mesmo $S_0$ deve refletir os extremos conforme sua definição, não a soma de toda atividade intermediária. Um contador de edições pode crescer; a divergência final pode voltar a zero. Quando uma nova geração passa a ser baseline, medições anteriores não são reescritas retroativamente. [S-CAD, cap. 2]

Isso preserva a intuição de “envelhecimento”, mas impede uma leitura literal de que D0 cresce necessariamente com o relógio. O que queremos estudar é a inadequação ou divergência da representação em relação à tarefa e ao estado observado.

# 8. Decisão tecnológica: ADR-001

## 8.1 Linguagem eleita e justificativa

**VIGENTE: Rust para o motor de produção.** A escolha foi feita depois de explicitar o objetivo de um motor local, eficiente, modular, auditável e utilizável também em servidor. Python foi reservado ao laboratório científico independente. O fato de o idealizador perguntar por alternativas significava acrescentar variáveis e compreender o processo, não defender uma linguagem ou pedir concordância automática. [S-CONV]

Rust oferece ownership como mecanismo de segurança de memória sem coletor obrigatório; isso não garante que o programa seja pequeno, correto, livre de deadlocks ou numericamente fiel. Dependências e trechos `unsafe`/FFI também precisam de revisão. Bibliotecas, algoritmos e representações podem pesar mais que a linguagem. [E-RUST]

## 8.2 Os dez critérios fechados

A tabela abaixo preserva a avaliação escolhida na conversa. **As notas são julgamento arquitetural, não medição empírica nem probabilidade de sucesso.** A agregação foi recalculada nesta edição e coincide com os totais publicados. Critérios parcialmente correlacionados e incerteza nas notas limitam a leitura dos décimos; 91,6 não significa desempenho 6,6% superior a 85,0.

| Critério | Peso | Rust | C++ | C# | Java |
|---|---:|---:|---:|---:|---:|
| Baixo consumo de RAM e CPU | 15% | 5 | 5 | 4 | 3 |
| Inicialização e distribuição multiplataforma | 12% | 5 | 5 | 4 | 3 |
| Velocidade de indexação e processamento | 12% | 5 | 5 | 4 | 4 |
| Concorrência, persistência e resiliência | 10% | 5 | 3 | 5 | 5 |
| Grafos e álgebra linear esparsa | 12% | 4 | 5 | 4 | 4 |
| Parsing e integração com analisadores | 7% | 5 | 4 | 4 | 4 |
| Segurança de memória e contratos de tipos | 12% | 5 | 3 | 4 | 4 |
| Inferência local de embeddings | 5% | 4 | 5 | 4 | 4 |
| MCP, CLI e servidores | 5% | 4 | 3 | 5 | 5 |
| Produtividade e manutenção assistida por IA | 10% | 3 | 2 | 5 | 4 |
| **Total ponderado / 100** | **100%** | **91,6** | **81,8** | **85,0** | **77,6** |

A regra de agregação foi $\mathrm{nota}=\sum_i w_i n_i/5$, com pesos em pontos percentuais e notas de 1 a 5. A classificação é Rust, C#, C++ e Java. Ela documenta por que escolhemos; não transforma preferências de engenharia em leis universais.

## 8.3 Alternativas debatidas, sem caricatura

C++ foi considerado especialmente forte para computação numérica e bibliotecas científicas, com maior responsabilidade de manutenção e segurança de memória. C# foi a principal alternativa por produtividade, tipos e ecossistema; Native AOT pode gerar aplicações autossuficientes, com restrições de compatibilidade. Java foi considerado viável para serviços e estruturas de grafos; não foi descartado por ser incapaz de desempenho. Python e Julia foram reconhecidos como atraentes para pesquisa, e Go como opção de ferramentas/serviços. [S-CONV; E-AOT]

O argumento final para Rust é o equilíbrio do **produto inteiro**, não a tentativa de vencer todas as linguagens em cada microbenchmark. Não há evidência de que agentes escrevem código cientificamente correto com maior frequência só porque uma linguagem recebeu nota alta.

## 8.4 O que a escolha não fixa

| Área | Estado da escolha |
|---|---|
| Motor e CLI | Rust, decidido. |
| Oráculos, cálculos de referência e estatística | Python/NumPy/SciPy, decidido como laboratório. |
| Parsing incremental | Tree-sitter é o primeiro candidato a avaliar. |
| Persistência | SQLite é candidata principal; contrato transacional vem antes. |
| Grafo | Estrutura tipada/esparsa; biblioteca não fixada. `petgraph` foi mencionada como candidata. |
| Álgebra linear | Biblioteca verificada por conformidade; `faer` foi mencionada e tem estruturas esparsas documentadas. |
| MCP | Integração posterior, via adaptador do mesmo núcleo; `rmcp` é candidato oficial disponível. |
| Embeddings | Backend substituível; `fastembed` foi citado, sem modelo/versionamento homologados. |
| GUI e serviço multiusuário | Fora do primeiro marco; não possuem stack escolhida. |

Tree-sitter é um gerador/parser incremental de árvore sintática concreta; sua disponibilidade não prova resolução semântica completa de chamadas entre arquivos. O SDK MCP não determina a arquitetura interna do motor. SciPy fornece operações de sistemas lineares e ação da exponencial, mas oráculos ainda precisam de entradas, pressupostos e tolerâncias corretos. [E-PARSER; E-MCP; E-FAER; E-CG; E-EXPM]

## 8.5 Encerramento e condições de revisão

Não haverá outra disputa de linguagens como pré-requisito automático da implementação. Nas primeiras entregas, verificaremos distribuição, dependências, fidelidade numérica e viabilidade de recursos. Falha de uma biblioteca pede primeiro revisão da biblioteca; um bloqueio estrutural comprovado ou mudança real de requisitos pode reabrir ADR-001.

A reconstrução não importará um segundo motor Python. A comparação científica pode inicialmente usar arquivos de entrada/saída e fixtures versionadas, sem FFI ou novo servidor. Usar linguagens distintas ajuda a separar implementações, mas **não garante independência se ambas reproduzem a mesma premissa errada**.

# 9. Fundamentos e contratos matemáticos discutidos

## 9.1 Matemática estabelecida versus modelo do produto

Esta seção conserva as formulações estudadas, com suas condições e alcance. **Não escolhe retroativamente o algoritmo da reconstrução.** Provar uma propriedade de um operador não prova que a propriedade representa relevância para engenharia de software. Implementar uma fórmula também não prova que ela mede o envelhecimento útil do RAG.

Cada futuro contrato deverá identificar entradas, unidades, domínio, convenções, casos degenerados, resultado esperado, tolerâncias e evidências de referência. A implementação não pode substituir a fórmula por outra apenas porque é mais conveniente ou está disponível em uma biblioteca.

## 9.2 D0: da média ponderada ao sensor textual legado

Uma formulação geral discutida desde a origem foi:

$$
D_0=\frac{\sum_{f\in U}m_f\,\delta_f}{\sum_{f\in U}m_f}.
$$

$U$ é o universo comparado, $m_f$ é o peso de cada unidade e $\delta_f$ é a divergência local. Se os pesos são não negativos, a soma dos pesos é positiva e cada divergência está em $[0,1]$, o resultado também está nesse intervalo. Essas condições precisam ser garantidas; o caso de soma zero exige definição própria. [S-HIST; S-CAD, cap. 8]

No TypeScript legado, o inventário localiza, para arquivos não simultaneamente vazios:

$$
\delta_f=\min\left(1,\frac{a_f+r_f}{n_{0,f}+n_{t,f}}\right),
\qquad
m_f=\log\left(1+\max(n_{0,f},n_{t,f})\right).
$$

$a_f$ e $r_f$ são linhas adicionadas e removidas pelo diff, e $n_{0,f},n_{t,f}$ são contagens de linhas nos dois estados. A agregação percorre a união dos caminhos. Dois arquivos vazios recebem divergência local zero antes da divisão. Essa é **uma proxy textual histórica**. Os pesos logarítmicos não estão demonstrados como os melhores preditores de perda de qualidade do contexto. [S-R00, §3; S-CAD, pp. 40–43]

| Exemplo didático por arquivo | $a_f,r_f$ | Denominador | Divergência |
|---|---:|---:|---:|
| Dez linhas sem alteração | 0, 0 | 20 | 0 |
| Substituir uma linha entre dez | 1, 1 | 20 | 0,10 |
| Adicionar uma linha a dez | 1, 0 | 21 | $1/21$ |
| Remover arquivo não vazio de dez linhas | 0, 10 | 10 | 1 |
| Restaurar exatamente o baseline | 0, 0 no diff dos extremos | Depende do arquivo | 0 |

Esses resultados pressupõem o diff e as convenções enunciadas. Uma renomeação pode ser vista como remoção mais adição se não houver política própria de identidade. Um arquivo vazio criado pode alterar o manifesto sem alterar um sensor que só conta linhas. Mudanças de comentário e de operador crítico podem ter o mesmo D0. Portanto, **D0 não substitui identidade de arquivos, invalidação ou análise de impacto**.

## 9.3 Novelty, SDS e pontos cegos

As discussões iniciais exploraram um Semantic Drift Score/SDS com parcelas de conteúdo, estrutura, novidade e outros sinais, com pesos heurísticos. Posteriormente, a conversa separou uma divergência agregada do repositório de métricas auxiliares de novelty/ponto cego. A formulação completa de todos os rótulos iniciais não está recuperada em fonte primária nesta edição; não se publicará uma equação “oficial” preenchendo essa lacuna. [S-HIST]

A distinção operacional preservada é clara: um arquivo novo não vazio já entra na comparação da união dos estados. Somar uma parcela de novidade com o mesmo significado pode duplicar seu efeito. Por outro lado, **informação ausente do índice** e **conteúdo novo no repositório** não são necessariamente a mesma coisa. A primeira pode justificar uma medida separada de cobertura/atraso de representação.

O caderno não encontrou uma fórmula autônoma de novelty como terceiro sensor global na base examinada. Encontrou uso da palavra relacionado à novidade de candidatos. Assim, novelty de recuperação e novelty de manutenção permanecem conceitos distintos; não devem receber um único campo sem contrato. [S-CAD, §8.7]

## 9.4 DG: relações e resumo espectral

O DG TypeScript legado combina distância relacional e componente espectral quando ambas estão disponíveis:

$$
D_G=0{,}6\,d_{\mathrm{rel}}+0{,}4\,d_{\mathrm{spec}}.
$$

O relatório descreve relações incidentes, pesos por linhas e origens consumidoras; o espectro usa componentes e $\lambda_2$ de uma projeção por arquivos/imports, não o espectro integral. Sem a componente espectral, a implementação retorna informação relacional com estado parcial. O servidor Python contém outro DG, por Jaccard global de arestas. A equação acima não deve ser aplicada a ambos. [S-R00, §§3 e 3.1]

Os pesos 0,6/0,4 são históricos e não constantes científicas. A reconstrução precisa definir universo, identidades, direção, comparação de nós e tratamento de indisponibilidade antes de adotar qualquer composição.

Arestas e espectro fornecem informações diferentes. Comparar identidades de relações preserva informação local que pode desaparecer em um resumo espectral. Grafos distintos podem ter o mesmo espectro; conectividade algébrica não é uma identidade completa da arquitetura. Uma pequena mudança estrutural pode ser importante para uma tarefa, e um grande deslocamento espectral pode não alterar a evidência de outra.

A associação intuitiva anterior de D0 a deformação/homotopia e de DG a ruptura deve permanecer como **motivação exploratória**, não como classificação topológica formal já demonstrada. Nenhuma métrica textual passa a medir homotopia apenas por receber esse nome. [S-HIST; S-CONV]

## 9.5 O grafo factual e a projeção matemática

Um grafo factual registra relações extraídas do código com seus tipos, direções e evidências. Uma projeção matemática escolhe unidades, arestas elegíveis, pesos, direção ou simetrização para aplicar determinado algoritmo. Não são o mesmo objeto.

No legado v3, a projeção térmica descrita admite `call` e `jsx_render`, associa extremos a unidades recuperáveis, remove laços, deduplica pares, simetriza e usa pesos unitários. Outros perfis usam arquivos/imports. Uma chamada factual dirigida pode alimentar uma projeção simétrica para exploração; isso **não inverte a verdade da chamada** e deve continuar explícito na proveniência. [S-R00, §3]

Na reconstrução, tipos/direções/confiança de extração e relação com a consulta são dimensões candidatas. Não se confunde “relação extraída com alta confiança” com “relação muito relevante para esta pergunta”. Nenhum grau de confiança deve ser chamado de probabilidade calibrada sem uma calibração correspondente.

## 9.6 Laplaciano e energia de Dirichlet

Para um grafo não dirigido com adjacência simétrica $W$, pesos $w_{ij}\ge0$ e matriz diagonal dos graus ponderados $D$, o laplaciano combinatorial é:

$$
L=D-W.
$$

Dado um vetor de escores $h$, a convenção usada nesta seção é:

$$
E(h)=h^T Lh
=\frac12\sum_{i,j}w_{ij}(h_i-h_j)^2
=\sum_{\{i,j\}\in E}w_{ij}(h_i-h_j)^2.
$$

Na última soma, cada aresta não dirigida aparece uma vez. Algumas apresentações chamam $\tfrac12h^TLh$ de energia; o fator deve ser declarado para não errar derivadas. Essa energia penaliza diferenças entre vizinhos. Não mede diretamente relevância semântica nem veracidade do conteúdo. [S-P23, §7; S-CAD, caps. 6–7]

“Energia de Dirichlet” não é uma etapa de transporte do código e não se confunde com “condições de contorno de Dirichlet”, nas quais valores são fixados na fronteira. A escolha de condições de contorno é uma decisão adicional quando se recorta um domínio.

Para grafos dirigidos ou normalizações diferentes, não se transplantam automaticamente simetria, conservação de massa ou garantias do solver. O laplaciano normalizado simétrico foi discutido como alternativa histórica; requer convenções próprias, sobretudo em nós isolados e na interpretação do equilíbrio.

## 9.7 Calor livre: uma evolução no tempo matemático

No modelo linear de calor livre, com $L$ fixo, simétrico e semidefinido positivo:

$$
\frac{dh}{d\tau}=-Lh,\qquad h(\tau)=e^{-\tau L}s.
$$

$s$ é o sinal inicial. Na escrita original com difusividade $\alpha$, $\tau=\alpha t$ reúne a escala do operador e do tempo. Tempo matemático não é tempo de CPU, número de iterações nem acompanhamento contínuo do repositório.

Sob essas condições, a energia satisfaz:

$$
E'(\tau)=-2\,h^TL^2h=-2\|Lh\|_2^2\le0,
$$

$$
E''(\tau)=4\,h^TL^3h\ge0.
$$

Logo, a proposta histórica de procurar uma mudança de sinal de $E''$ como inflexão útil foi rejeitada **nesse modelo**. Uma curva de energia decrescente e convexa pode ter mudanças de escala, mas não aquela inflexão. Não se usa um gráfico plausível para revogar essa propriedade. [S-P23, §7]

O plano antigo discutiu $u=d_{\max}\tau$, $x=\tau_{cut}$, $\tau_{warm}=x+y$ e uma definição operacional de proximidade ao equilíbrio por variação total. Também separou energia global, fronteira local e pertinência semântica. Esses observáveis e seus defaults estão preservados no apêndice B como históricos; o controlador antigo não é automaticamente o controlador novo.

## 9.8 Reinjeção persistente: outro problema

Uma dinâmica com fonte persistente é:

$$
\frac{dh}{dt}=-Lh+\beta(s-h),\qquad \beta>0.
$$

Seu equilíbrio satisfaz:

$$
(L+\beta I)h^*=\beta s.
$$

Ele também minimiza:

$$
\frac12h^TLh+\frac\beta2\|h-s\|_2^2.
$$

O primeiro termo favorece suavidade estrutural; o segundo mantém influência das sementes. O legado v3 resolve diretamente o sistema, com PCG e precondicionamento diagonal. As iterações tentam aproximar uma solução fixa; não são passos físicos de exploração no grafo. [S-CAD, pp. 37–38; S-R00, §3]

Em um modo espectral de autovalor $\lambda$, o calor livre multiplica por $e^{-\tau\lambda}$, e a reinjeção por $\beta/(\beta+\lambda)$. São filtros relacionados, mas não iguais. O caderno registra a relação por média ponderada de kernels temporais; isso não transforma um benchmark do equilíbrio em benchmark de um tempo particular de calor livre.

O valor legado $\beta=1$ é configuração observada, não decisão herdada. O contrato da reconstrução deverá declarar qual objeto calcula e por que escolhe aquela parametrização.

## 9.9 Resíduo, erro e ordem certificada

Para $M=L+\beta I$ e aproximação $\hat h$, o resíduo é $r=\beta s-M\hat h$. No modelo simétrico especificado, $\lambda_{\min}(M)\ge\beta$, portanto:

$$
\|h^*-\hat h\|_2\le\frac{\|r\|_2}{\beta}.
$$

Esse é um limite matemático em termos do resíduo correto. Cálculo em ponto flutuante acrescenta erros de medição e arredondamento. A margem estimada do legado não constitui aritmética intervalar formal. Se o vetor for projetado, truncado ou renormalizado depois do solver, a certificação deve corresponder ao vetor efetivamente devolvido. [S-CAD, pp. 38 e 80; S-R00, §3]

Uma separação entre duas coordenadas maior que duas vezes uma cota de erro por coordenada permite certificar sua ordem. Isso prova uma propriedade numérica; não prova que a coordenada maior seja a evidência mais importante. `CONVERGED`, `INCOMPLETE`, `CANCELLED` e falhas numéricas não devem ser confundidos com sucesso semântico.

## 9.10 Difusão discreta, PPR e orientação de matrizes

O relatório registra uma recorrência anterior com reinício:

$$
h^{(k+1)}=(1-\gamma)s+\gamma A D^{-1}h^{(k)}.
$$

O comentário dizia $D^{-1}A$, mas o laço dividia pelo grau do vizinho emissor, correspondendo a $AD^{-1}$ na convenção de vetor coluna descrita. Filtros, corte e tratamento de isolados impedem presumir conservação universal apenas pelo nome. [S-R00, §3]

Personalized PageRank foi proposto como baseline de propagação com reinício. O novo experimento deverá declarar orientação, normalização, sementes, tratamento de nós sem saída e parâmetro de reinício. Parentesco ou coincidência em casos especiais não autoriza chamar todos esses operadores de equivalentes, nem tratar nomes diferentes como métodos necessariamente distintos.

# 10. O papel da difusão e a hipótese dos materiais

## 10.1 Ordenação contínua, seleção discreta

A proposta discutida é que busca tradicional encontre pontos de entrada, o grafo registre relações e a difusão produza escores graduais para apoiar ordenação, expansão e seleção. Ela não substitui lexical, vetorial ou relações resolvidas.

“Contínua” significa que os escores não são apenas sim/não ou distância inteira por saltos. A escolha final de trechos é discreta, e um cálculo estacionário por consulta não é um campo sendo atualizado em tempo real no processo. A autonomia do watcher e o tempo de difusão são assuntos separados. [S-CONV]

BFS por distância pode empatar candidatos a um salto; múltiplas sementes e caminhos podem diferenciá-los por propagação. Mas uma BFS acompanhada por bons critérios de desempate não precisa ser uma expansão ingênua que inclui tudo. Para demonstrar valor incremental, o baseline deve ser competitivo, não artificialmente fraco.

## 10.2 O que a topologia não sabe sozinha

Um utilitário genérico pode ser bem conectado e irrelevante à tarefa. Um símbolo necessário pode estar isolado ou conectado por um tipo de relação ausente da projeção. Uma semente equivocada pode aquecer uma comunidade perfeitamente coerente, mas errada. Calor zero não certifica irrelevância; uma região homogênea não certifica suficiência.

Por isso, a separação entre recuperação, difusão e seleção é essencial. O score estrutural não substitui a pergunta, a leitura da evidência nem o gabarito externo de avaliação.

## 10.3 A analogia do material

O idealizador observou que cobre e estanho não transmitem calor da mesma maneira. A correspondência investigada é: **relações diferentes entre elementos do código podem transmitir relevância com intensidades diferentes**. A analogia motivou condutâncias de aresta e, secundariamente, capacidades por nó. [S-CONV]

A precisão conceitual necessária é que mudar material costuma alterar coeficientes e propriedades do modelo, não obrigatoriamente toda a forma da equação. No Compass, a atribuição de pesos é uma escolha de modelagem. A física não determina que uma chamada, uma importação ou uma composição JSX deva ter um valor específico.

## 10.4 Laplaciano ponderado e exemplo conferível

Para pesos não negativos e simétricos, usar $L_W=D_W-W$ preserva a formulação:

$$
(L_W+\beta I)h=\beta s.
$$

Considere três nós, A ligado a B e C, semente $s=(1,0,0)$ e $\beta=1$. Com pesos iguais a 1, a solução é $(1/2,1/4,1/4)$. Se A–B tem peso 1 e A–C peso $1/10$, a solução é $(22/35,11/35,2/35)$.

| Nó | Pesos iguais | A–B forte; A–C fraca |
|---|---:|---:|
| A | 0,500000 | 0,628571 |
| B | 0,250000 | 0,314286 |
| C | 0,250000 | 0,057143 |

O cálculo demonstra que pesos relativos alteram a distribuição. **Não demonstra que B é mais relevante que C.** Os valores foram conferidos algebricamente nesta edição como exemplo didático; não são métricas do aplicativo.

Multiplicar todos os pesos pelo mesmo $c>0$ equivale, após dividir a equação por $c$, a substituir $\beta$ por $\beta/c$. Assim, uma melhora com pesos globalmente maiores pode vir apenas de mudança de escala. O experimento precisa distinguir escala global e informação nos pesos relativos.

## 10.5 Fontes candidatas de condutância

Foram discutidos tipo de relação, confiança da extração, especificidade da ligação e dependência da consulta, $w_{ij}(q)$. Cada alternativa tem riscos. Confiança de extração e utilidade contextual são diferentes. Penalizar hubs pode remover dependências legítimas. Pesos condicionais à consulta aumentam flexibilidade e oportunidade de sobreajuste.

**HIPÓTESE COND-001:** condutâncias informativas melhoram a priorização comparadas a pesos uniformes, sob insumos e orçamento controlados. A decisão vigente é preservá-la para investigação, sem alterar o motor legado e sem adotá-la como default novo.

## 10.6 Capacidade térmica: por que não basta acrescentar uma matriz

Uma dinâmica ilustrativa sobre rede pode usar uma matriz diagonal positiva $C$ de capacidades:

$$
C\frac{dh}{dt}=-L_Wh+\beta(s-h).
$$

Nessa formulação específica, no equilíbrio a derivada desaparece e resta o mesmo sistema $(L_W+\beta I)h=\beta s$. A capacidade muda a trajetória temporal, não esse equilíbrio. Se forem alterados termos de fonte, ancoragem ou unidades, pode surgir outro modelo; isso exige contrato próprio.

A conversa decidiu priorizar a hipótese de condutâncias, não implementar capacidades como um detalhe cosmético. Não se acrescentará complexidade apenas para tornar a metáfora mais física.

## 10.7 Fronteiras e recortes

Recortar um subgrafo e remover suas conexões externas pode criar retenção artificial. A observação histórica de fronteira propunha medir fluxos mantendo o exterior pertinente. Uma otimização local futura precisa declarar condições de contorno e erro, em vez de assumir que resolver o recorte é equivalente a resolver o grafo inteiro. [S-P23, §§8–9 e 21]

Região, conjunto de candidatos e pacote entregue são três objetos. Mesmo uma região estruturalmente bem delimitada pode fornecer apenas parte do contexto para uma tarefa multimódulo.

# 11. Registro de hipóteses e ideias não incorporadas

## 11.1 Hipóteses principais

Os IDs abaixo preservam a organização estabelecida na conversa. Não renomeiam retroativamente H0/H1 de campanhas anteriores.

| ID | Pergunta investigada | Estado científico preservado |
|---|---|---|
| **DRIFT-001** | D0/DG ajudam a decidir atualização com melhor qualidade/custo que políticas simples? | Utilidade operacional não demonstrada adequadamente. |
| **RETR-001** | Relações estruturais acrescentam evidências úteis à recuperação? | Indícios exploratórios; contribuição precisa de isolamento melhor. |
| **HEAT-001** | Propagação/difusão melhora ranking e seleção frente a alternativas competitivas? | Inconclusiva; operador precisa ser identificado. |
| **COND-001** | Pesos informativos nas relações superam projeção uniforme? | Proposta nova no diálogo, sem teste confirmatório. |
| **AGENT-001** | Compass melhora correção de tarefas ou eficiência do agente? | Não demonstrada com generalização e controle suficientes. |

Falhar em COND-001 não refuta automaticamente HEAT-001. Falhar em HEAT-001 não refuta DRIFT-001. Uma limitação do recuperador pode impedir medir bem o benefício operacional de manutenção, mas não invalida a matemática de uma comparação de snapshots.

## 11.2 Hipótese inicial de predição de perda

O contexto recuperado das primeiras discussões registra uma proposta de comparar o indicador de drift com perda de qualidade $L=Q_{fresh}-Q_{stale}$, utilizando correlação de Spearman, hipótese nula $\rho_s(D,L)\le0$, alternativa positiva, nível $\alpha=0,05$ e Recall@5 como métrica inicialmente cogitada. O ensaio seria retrospectivo em históricos de commits. [S-HIST]

Esse protocolo é **histórico e recuperado de conversa**, não uma campanha comprovadamente executada. Predição de perda e decisão ótima de atualização são perguntas diferentes. Um sensor correlacionado pode custar caro demais ou ser inferior a uma política simples. O novo protocolo precisará congelar definição de $Q$, idade do índice, unidade experimental e baselines.

## 11.3 Langevin e evolução estocástica

Foi levantada a ideia de usar Langevin para acompanhar a evolução do sistema, interpretando diffs como forças determinísticas e flutuações ligadas ao raciocínio da IA como ruído. Trata-se de uma intuição registrada, não de um modelo operacional aprovado. [S-CONV; S-HIST]

Faltam variável de estado, observáveis, lei de ruído, escala temporal, mecanismo de estimação e previsão verificável. Não se pode declarar que “raciocínio da IA é ruído branco” sem evidência. A edição não inventa uma equação estocástica do Compass para preencher a lacuna. Essa frente permanece fora da implementação inicial.

## 11.4 Outras alternativas históricas preservadas

O plano de 23/09 discutiu laplaciano normalizado, PPR, pesos por tipo/direção, sementes proporcionais ao score, cálculo local com fronteiras corretas, cross-encoder, classificação aprendida de intenção, reação-difusão, change-points, joelhos de curva, limiares por família, ganho por token, múltiplas regiões, GNN e aprendizado online. São backlog e alternativas, não compromissos do novo produto. [S-P23, §21]

Do mesmo modo, os papéis de agentes e os 14 entregáveis históricos do motor integral não se tornam roadmap vinculante da reconstrução apenas por terem sido enumerados. A obrigação é não perder sua motivação; não é reproduzir sua complexidade.

## 11.5 Condições de decisão científica

Continuar uma linha exige evidência suficientemente útil e reproduzível no escopo anunciado. Mudar de abordagem é adequado quando um componente mais simples atende melhor ao objetivo. Interromper uma linha é defensável quando testes válidos e implementações conformes não mostram benefício suficiente para seu custo.

Um resultado inconclusivo não é vitória e não é refutação universal. Também não autoriza uma sequência ilimitada de novas exceções até a hipótese vencer. Orçamento, pergunta, efeito mínimo útil e decisão possível devem estar definidos antes do experimento confirmatório.

# 12. Verificação, experimentos e critérios de evidência

## 12.1 A cadeia de conformidade

O percurso obrigatório é: **problema → hipótese → contrato matemático/computacional → implementação → execução identificada → medição → conclusão delimitada**. Cada seta pode falhar. A auditoria deve localizar a falha sem reescrever a pergunta depois de conhecer a resposta.

Há cinco verificações distintas: a formulação corresponde à intenção; o algoritmo corresponde à formulação; o código realiza o algoritmo; a rota pública realmente o chama com aqueles parâmetros; o experimento mede a vantagem pretendida. Um teste unitário cobre apenas os vínculos que efetivamente examina.

## 12.2 Oráculos e independência

Resultados esperados podem vir de soluções analíticas pequenas, aritmética exata ou implementações de referência com método distinto. NumPy/SciPy são ferramentas úteis, não fontes mágicas de correção. O oráculo deve compartilhar o problema especificado, mas não simplesmente copiar o algoritmo e suas suposições equivocadas. [E-CG; E-EXPM]

O implementador não altera unilateralmente fixtures ou rubricas para acomodar sua saída. Uma contestação legítima do gabarito gera revisão explícita, com motivo e versão. Testes antigos podem orientar investigação, mas não serão copiados sem avaliação para se tornarem autoridade da nova implementação.

## 12.3 Testes de propriedades, negativos e mutações

Exemplos discutidos incluem retorno ao baseline, estados iguais, arquivos adicionados/removidos, vazio, denominador zero, valores finitos, limites numéricos e simetrias realmente previstas. Para grafos, verificar casos analíticos, nós isolados, desconexão, pesos extremos, invariância a renomeação de IDs quando aplicável e comportamento em falha de convergência.

A ideia de mutation testing é introduzir erros controlados no ambiente de teste: retornar sempre zero, trocar denominador, omitir uma aresta, inverter orientação ou esconder indisponibilidade. Se os testes aceitam um erro que deveriam proibir, o instrumento precisa de revisão. Não se promete correção universal nem se define nesta edição um percentual arbitrário de mutantes como prova científica.

## 12.4 Teste da ordenação: isolar antes de integrar

O primeiro ensaio de ranking deve congelar consulta, snapshot, grafo, revisões, candidatos, sementes e orçamento. O gabarito fica fora do alcance dos recuperadores. A intervenção é a regra de ordenação/seleção; o empacotador e os diagnósticos entregues devem ser controlados.

Foram propostos: recuperação direta como controle; expansão por distância/BFS com desempate explícito; difusão/reinjeção sob contrato; Personalized PageRank como alternativa competitiva. A comparação pode ocorrer em duas etapas: **ranking em universo fixo** e, depois, **aquisição com expansão**. Uma busca “sem expansão” não recebe artificialmente o mesmo comportamento de expansão só para caber na tabela; cada contraste declara o que foi fixado e o que pode mudar.

Quando métodos acessam conjuntos diferentes, o ganho pode vir de descoberta, ordenação, seleção ou apresentação. O relatório precisa separar essas causas. Diagnósticos volumosos e nomes de braços enviados ao modelo podem alterar seu comportamento; não são automaticamente efeitos da relevância calculada.

## 12.5 Teste de condutância

Comparar pesos unitários com pesos por tipo/especificidade ou outra hipótese explicitamente escolhida. Fixar as condições anteriores, controlar escala global em relação a $\beta$ e calibrar somente no desenvolvimento. Avaliar tarefas desfavoráveis: hubs, ponte de baixo cosseno, componente isolado, semente errada e relações verdadeiras porém irrelevantes.

Um resultado melhor em pesos escolhidos a partir do gabarito não valida a ideia em dados novos. A dependência da consulta, se adotada, deve ter método identificado e avaliação separada do peso fixo.

## 12.6 Teste de manutenção

Reconstruir sequências identificadas de commits/alterações e comparar políticas como atualização integral, periódica, incremental por eventos e orientada por D0/DG. As alternativas devem compartilhar garantias mínimas de validade; não se atribui ganho de um algoritmo que simplesmente deixa de fazer trabalho obrigatório.

Medir custo de observação, custo do sensor, invalidação, recomputação, atraso de disponibilidade e qualidade da recuperação. Uma análise de índice deliberadamente obsoleto pode ser útil no laboratório, mas não justifica servi-lo como atual na produção. Mudanças pequenas e críticas, renomeações, reversões, exclusões e eventos perdidos são cenários necessários para investigar limites. [S-CONV]

## 12.7 Teste ponta a ponta com agentes

Só depois de haver instrumento confiável, medir diagnósticos, patches, testes funcionais, regressões, tempo e custo. Ambiente Compass-only e ambiente de uso livre respondem a perguntas diferentes. No primeiro, a restrição de ferramentas precisa existir de fato; um prompt pedindo para não usar shell não prova isolamento. No segundo, as ferramentas convencionais devem estar igualmente disponíveis aos braços. [S-P23, §16]

Preservar todas as sessões previstas, resultados parciais, cancelamentos, falhas e custos. Um erro de infraestrutura pode justificar interrupção e nova versão do experimento; uma resposta ruim é um resultado, não uma autorização de retry seletivo até vencer.

## 12.8 Métricas de recuperação e produto

| Métrica | Uso legítimo | O que não representa |
|---|---|---|
| Precisão | Fração de itens entregues relevantes segundo referência | Suficiência de todo o contexto. |
| Recall/cobertura | Fração de evidências de referência recuperadas | Entendimento ou correção da resposta do agente. |
| Cobertura sob orçamento $R@B$ | Fatos indispensáveis entregues dentro de $B$ tokens | Uma definição universal sem rubrica e tokenizer. |
| Qualidade do ranking | Posição de evidências relevantes | Atualidade e autenticidade. |
| Sobrevivência no packing | Evidência encontrada que chegou ao envelope | Evidência efetivamente usada pelo modelo. |
| Atualidade declarada/verificada | Correspondência à revisão e ao instante contratado | Garantia de que o disco nunca mudou depois. |
| Sucesso funcional | Obrigações/testes externos da tarefa satisfeitos | Generalização para qualquer repositório. |

A fórmula didática proposta é:

$$
R@B=\frac{\text{fatos indispensáveis entregues dentro do orçamento}}{\text{total de fatos indispensáveis da rubrica}}.
$$

O denominador, evidências alternativas aceitáveis, granularidade e critérios de completude de trechos precisam ser congelados. Não exigir um conjunto de linhas arbitrário como única verdade quando há outra evidência válida para o mesmo fato; não flexibilizar bordas pós-hoc sem identificar a análise de sensibilidade.

## 12.9 Estatística e tamanho de campanha

A unidade de análise não é cada token, nó, checkpoint ou requisição. O protocolo declara tarefa, snapshot, repetição e agrupamento por repositório. Repetições ajudam a estimar variabilidade, mas não criam novos problemas independentes.

Para um contraste de benefício, definir antes da avaliação o efeito mínimo útil $\delta$ e o sentido da comparação. Uma formulação candidata é $H_0:\mathbb E[\Delta]\le\delta$ contra $H_1:\mathbb E[\Delta]>\delta$. Isso é **um molde para o novo protocolo**, não uma revisão retroativa dos H0/H1 antigos.

Intervalo incluindo zero não demonstra superioridade nem equivalência. Intervalo positivo, sozinho, tampouco estabelece utilidade econômica. O método de inferência deve respeitar dependências e a alegação de generalização. Não confundir milhares de reamostragens com milhares de tarefas.

Na conversa foram sugeridos pilotos de aproximadamente 20–30 perguntas e, antes, campanhas maiores de 200–300 tarefas em 8–10 repositórios. **Esses números foram propostas de planejamento, não cotas científicas aprovadas.** O tamanho confirmatório depende de variabilidade, agrupamento, efeito útil e orçamento. Os testes T01/T07 e projetos já estudados servem ao desenvolvimento; não se tornam inéditos por renomeação.

## 12.10 Critério de avanço, sem bloqueio circular

Integração funcional pode avançar quando os contratos pertinentes passam, mesmo antes de provar superioridade do produto completo. Caso contrário, exigiríamos evidência de um sistema que ainda não existe para permitir construí-lo. A exigência de eficácia deve governar adoção como default e alegações de benefício, não impedir todos os protótipos necessários para medir.

Por outro lado, uma falha de instrumento não permite concluir que a hipótese funciona ou não funciona. O objetivo é executar poucos experimentos bem definidos, capazes de produzir uma decisão, em vez de crescer indefinidamente a suíte e a documentação.

# 13. Recursos, desempenho e operação local/servidor

## 13.1 Leveza é requisito, não consequência do nome Rust

A linguagem influencia overhead, inicialização e controle de memória, mas representação dos dados, cache, inferência e algoritmos podem dominar o consumo. Um projeto Rust que carrega tudo em RAM pode ser mais pesado que uma arquitetura gerenciada que carrega apenas o necessário. Não há benchmark das quatro linguagens implementando o Compass novo. [S-CONV]

O produto deve medir repouso, indexação inicial, atualização incremental, consulta fria/quente e recuperação após interrupção separadamente. “CPU baixa em repouso” não implica “indexação barata”; latência baixa depois de carregar o modelo não representa o custo de uma primeira consulta.

## 13.2 O exemplo dos embeddings

Para $N$ vetores de dimensão $d$ e $b$ bytes por coordenada:

$$
M_{vetores}=Ndb.
$$

Com 100.000 vetores, 384 dimensões e float32, o total bruto é 153.600.000 bytes: **153,6 MB decimais, aproximadamente 146,48 MiB**. Esse cálculo não inclui pesos do modelo, índices auxiliares, metadados, grafo, buffers, runtime e cache. Não pressupõe que tudo precise ficar residente em RAM.

Quantidade de chunks/janelas, quantização, estruturas contíguas e carregamento parcial alteram a conta. Uma otimização de representação deve preservar a qualidade e ter medição própria; não é automaticamente gratuita em precisão.

## 13.3 Orçamento completo de recursos

| Dimensão | O que medir/contratar |
|---|---|
| Memória | Residente em repouso, pico durante trabalho, modelos/cache e soma dos processos. |
| CPU | Tempo de processamento, concorrência e interferência na atividade do usuário. |
| Latência | Inicialização, primeira indexação, atualização, query e percentis sob carga. |
| Disco e I/O | Índices, modelos, snapshots, logs, leitura/escrita e retenção. |
| Trabalho numérico | Nós/arestas visitados, iterações, resíduo e encerramento. |
| Contexto | Tokens de conteúdo, metadados e envelope completo. |
| Recuperação | Retomada após falha, consistência de publicação e trabalho perdido. |

Metas numéricas finais ainda não foram definidas. Serão estabelecidas com máquinas e cargas de referência, não herdadas dos limites do antigo controlador térmico.

## 13.4 Estratégias debatidas

Manter observação leve, adiar trabalho caro não urgente, reutilizar derivados válidos, limitar concorrência, evitar matrizes densas desnecessárias, compartilhar estruturas somente quando seguro e descarregar recursos ociosos são diretrizes de eficiência. A implementação específica permanece sujeita a contrato e perfil de desempenho.

Em servidor, pode ser vantajoso compartilhar inferência de embeddings entre repositórios, mas isolamento, filas e controle de recursos precisam ser mantidos. Isso não determina um serviço central novo já na Foundation. O custo que importa é o conjunto de processos, não somente o executável da CLI.

**Regra vigente:** pressão de recursos pode reduzir ou adiar trabalho opcional; não pode justificar contexto conhecido como obsoleto apresentado silenciosamente como atual.

# 14. Segurança, privacidade e limites de confiança

## 14.1 Confidencialidade como direção

A pesquisa e o código são proprietários até decisão explícita em contrário. O histórico registra preocupação com propriedade intelectual e proteção contra exposição. Não há autorização geral para publicar repositórios, enviar dados a provedores, instalar plugins, expor serviços ou executar campanhas pagas. [S-CONV; S-HIST]

A visão local-first aceita uso do motor no ambiente do usuário. Uma sugestão histórica de manter todo o núcleo apenas no servidor como caixa fechada não é a direção atualmente escolhida; a tensão entre distribuição local e proteção intelectual deve ser tratada explicitamente, não escondida atrás da linguagem.

## 14.2 Fronteiras técnicas a especificar

**PROPOSTA para os contratos de segurança:** definir raiz autorizada do workspace, comportamento de links e caminhos externos, arquivos excluídos/segredos, conteúdo não confiável, permissões de leitura/escrita, alcance de logs e identidade do cliente. O contexto recuperado é dado do repositório, não uma instrução capaz de alterar o protocolo do agente ou os critérios do experimento.

Uma leitura autêntica pode conter código malicioso ou instrução de prompt injection. Autenticidade de bytes não é autorização de execução. O Compass não deve executar código de repositórios por conveniência de indexação sem contrato e autorização próprios.

No servidor, autenticação, separação de repositórios/usuários, quotas e política de retenção são necessidades a detalhar antes dessa entrega. Não se afirma que já estão resolvidas pela escolha de SQLite, Rust ou MCP.

## 14.3 Alegações de proteção

Nenhuma linguagem torna um binário impossível de analisar. A publicação de detalhes científicos e a proteção por patente foram debatidas como questões estratégicas. Esta edição não é parecer jurídico, busca de anterioridade nem confirmação de patenteabilidade. Antes de divulgação técnica sensível, a avaliação especializada e a decisão do idealizador permanecem necessárias.

Não se reapresentam aqui prazos legais de países como orientação vigente. O registro histórico dessa preocupação não substitui verificação jurídica atual.

# 15. Governança e trabalho com agentes

## 15.1 O papel do idealizador

Emanuel conduz produto, perguntas de pesquisa e decisões de escopo. O projeto não exige que domine antecipadamente toda a matemática, compiladores, sistemas, recuperação e estatística. A ajuda de IA é parte legítima da estratégia. A compensação para lacunas específicas deve ser verificação independente e explicação compreensível, não confiança irrestrita no agente mais eloquente. [S-CONV]

Há uma preferência expressa que precisa acompanhar os handoffs: **questionar Rust, C++, C# ou Java não significou defender outra linguagem**. O idealizador acrescenta variáveis para compreender o processo. O parceiro técnico deve responder aos argumentos, manter ou revisar a conclusão por razões verificáveis e não interpretar toda pergunta como pedido para concordar ou trocar de direção.

## 15.2 Papéis e fronteiras de autoridade

| Papel | Entrega | Limite |
|---|---|---|
| Direção do projeto | Pergunta, prioridade, aprovação de escopo e decisões | Não precisa escrever sozinho toda a fundamentação. |
| Pesquisador | Hipótese, pressupostos, modelo e proposta experimental | Não transforma plausibilidade em desempenho comprovado. |
| Auditor matemático | Derivações, exemplos de referência, condições e limites | Não ajusta o esperado ao resultado do implementador. |
| Implementador | Módulo e integração conformes ao contrato ativo | Não altera unilateralmente fórmula, rubrica, orçamento ou escopo. |
| Auditor experimental | Instrumento, recibos, métricas e conclusões | Não elimina perdas nem modifica retrospectivamente o protocolo. |
| Editor/consolidador | Clareza, rastreabilidade e estado documental | Não converte reconstrução editorial em decisão aprovada. |

Astra, Sol, Gemini e outros modelos foram usados/discutidos em papéis diferentes. A arquitetura de governança não deve depender de um nome comercial ou de uma suposta infalibilidade. Um agente diferente pode repetir a mesma premissa errada. A independência precisa estar nos métodos, fontes, critérios e dados, e a revisão humana especializada é particularmente valiosa nas alegações científicas e de segurança.

## 15.3 Uma tarefa, um contrato e uma entrega observável

Cada tarefa futura deve declarar capacidade concreta, contrato/versão, entradas e saídas, arquivos autorizados, limites, testes pertinentes, critério de aceite e o que não fará. O agente apresenta a entrega e as evidências, não apenas uma narrativa de implementação.

Novas ideias surgidas durante a execução são registradas como propostas. Não viram alterações silenciosas de modelo. Um resultado inesperado deve primeiro ser classificado: implementação, integração, experimento, hipótese ou causa ainda desconhecida. Não se responde automaticamente com uma refatoração ampla.

Não executar campanhas externas, instalar dependências que transmitam dados, aplicar migrations em bases reais ou publicar código sem autorização adequada. Documentação de arquitetura não é autorização financeira ou operacional ilimitada.

## 15.4 Controle da árvore de trabalho

O novo repositório deve manter identidade explícita do que foi alterado, integrado, testado e publicado. Uma branch/commit identifica somente o conteúdo que efetivamente contém. Alterações não commitadas relevantes devem ser declaradas nos registros de execução; resultados não podem ser atribuídos apenas a HEAD quando o binário vem de outro estado.

A política detalhada de branches, worktrees, revisões e integração será definida na fundação operacional. A direção é impedir múltiplos agentes escrevendo de modo descoordenado nas mesmas fronteiras. Não há obrigação de criar um sistema sofisticado de orquestração antes do primeiro componente.

## 15.5 Auditoria proporcional ao risco

A reação à bagunça não será substituir implementação por infinitas auditorias. A Missão 00 já cumpriu seu papel de inventário; o arquivamento é uma operação delimitada. Testes e revisões novos devem responder ao risco da mudança ou à hipótese ativa.

Uma fórmula alterada exige conferência matemática. Uma troca de serialização exige testes de contrato. Uma mudança de índice pode exigir integração. Nem toda alteração requer repetir campanhas pagas ou redigir outra apostila. A entrega precisa progredir de modo visível, preservando incertezas que não bloqueiam legitimamente o próximo passo. [S-CONV; S-P23, §19]

# 16. Documentação que não recria a confusão

## 16.1 Uma autoridade por assunto

| Artefato | Responsabilidade | O que não deve fazer |
|---|---|---|
| **Documento principal** | Visão, decisões vigentes, fundamentos, limites e referências | Fingir que descreve o código atual a cada commit. |
| **`COMPASS_STATE.md`** | Estado operacional curto: commit, capacidades verificadas, tarefa ativa, bloqueios e links | Repetir a teoria inteira ou revogar uma decisão científica sem registro. |
| **Contratos versionados** | Definição executável de um componente ou experimento | Derivar automaticamente o esperado da implementação. |
| **Artefatos experimentais** | Configuração, dados, recibos, scripts e resultados imutáveis | Ser sobrescritos para refletir uma conclusão posterior. |
| **Caderno histórico** | Estudo e fotografia do legado | Servir como instrução automática para o novo código. |
| **`AGENTS.md` novo e mínimo** | Indicar onde estão as autoridades e limites da tarefa | Importar roadmaps antigos ou autorizar trabalho não solicitado. |

Esta organização reconcilia duas necessidades da conversa: um documento principal completo e um estado operacional pequeno. Não são dois planos mestres concorrentes. O primeiro descreve a direção; o segundo aponta o que já existe e o trabalho autorizado, com referências ao contrato ativo.

## 16.2 Markdown e PDF são a mesma edição

A fonte principal desta entrega é Markdown, apropriada para versionamento e uso no novo repositório. O PDF é a representação de leitura/impressão da mesma edição. Correções futuras devem ocorrer na fonte e gerar nova edição; não editar os dois independentemente.

A versão impressa permanece uma fotografia. As anotações manuais serão tratadas como observações ou propostas até uma decisão registrada incorporá-las. Não se apagam os resultados antigos nem se altera sua origem para combinar com a interpretação atual.

## 16.3 Contrato pedagógico preservado

O leitor é o idealizador, com boa compreensão técnica e sem pressuposição de formação avançada em todas as áreas. A explicação deve partir do problema e da intuição, introduzir símbolos e pré-requisitos, desenvolver um exemplo, relacionar à implementação e discutir validação. Não infantilizar; não ocultar etapas usando “trivialmente”.

O pedido de um único documento completo foi mantido. Uma produção modular interna não precisa virar dezenas de entregas desconectadas. A sugestão histórica de parar após um capítulo piloto foi substituída por uma obra integral com revisão interna; essa decisão editorial não autoriza eliminar revisão humana do conteúdo científico. [S-CONV]

Este documento principal não pretende substituir todos os exercícios das 138 páginas do caderno. Preserva o conteúdo necessário para decidir e implementar, e identifica onde estão as derivações e discussões históricas adicionais. A aprendizagem e a auditoria continuam possíveis sem que o usuário precise aceitar a opinião do agente como oráculo.

## 16.4 Mudanças e novos debates

Uma alteração deve informar decisão anterior, motivo, evidência, novo alcance e o que permanece igual. O contexto original não deve ser apagado. Enquanto estiver em debate, a alternativa aparece como hipótese/proposta, não como “direção atual”.

**Norma de não contaminação:** fontes históricas são lidas somente quando a tarefa pede uma investigação delimitada. Seus comandos imperativos, caminhos, defaults e critérios de rollout não governam automaticamente o novo repositório.

# 17. Fundação e progressão de implementação

## 17.1 Primeiro marco aprovado: Compass Foundation

**Objetivo:** observar e comparar estados de um repositório com identidade, rastreabilidade e resultados verificáveis.

A entrega inicial será um núcleo Rust e uma CLI. Precisará representar arquivos/revisões/snapshots conforme contrato, registrar o escopo de observação, comparar estados deterministicamente e produzir relatório estruturado. O cálculo D0 entra depois que se define exatamente o que os estados representam e qual divergência será medida. [S-CONV]

**Fora desse marco:** grafo, embeddings, difusão, MCP, GUI, servidor multiusuário, aprendizado online e migração do legado.

A estrutura mínima discutida foi:

```text
compass-d0/
  Cargo.toml
  Cargo.lock
  crates/
    compass-core/
    compass-cli/
  research/
    contracts/
    hypotheses/
  verification/
    reference/
    fixtures/
  README.md
```

O documento principal e o futuro `COMPASS_STATE.md` terão localização explícita no novo repositório. Os nomes de módulos acima são organização inicial proposta; não representam diretórios já criados por esta entrega. Não se importam as árvores do projeto antigo.

## 17.2 Critérios para fechar o contrato dos estados

Antes da implementação do comparador, o contrato precisa definir o universo elegível, identidade de caminho e conteúdo, tratamento de ausência/inacessibilidade, consistência da observação, normalizações e formato do relatório. Precisa também estabelecer a independência dos exemplos esperados.

Os cenários de aceite a detalhar são: estados iguais; criação e exclusão reais; arquivo vazio; conteúdo inacessível; observação incompleta; alteração durante leitura; restauração do baseline; renomeação conforme política; links conforme limite do workspace; saída determinística; erro explícito; fonte e snapshot identificáveis. Não se antecipam resultados para cenários cujo contrato ainda não existe.

Um comando ilustrativo discutido foi:

```text
compass compare --baseline snapshot-a --current snapshot-b --json
```

Ele exemplifica a capacidade pretendida, não uma API implementada nem um parser de CLI já homologado.

## 17.3 Progressão posterior: capacidade antes de sofisticação

A direção conversada estabelece Foundation primeiro e expansão por componentes verificáveis. A sequência abaixo é uma **proposta de planejamento**, não cronograma fechado nem autorização de construir todos os módulos de uma vez.

| Etapa proposta | Incremento observável | Pergunta de controle |
|---|---|---|
| Foundation | Snapshots, comparação e CLI rastreáveis | Sabemos quais estados foram comparados e por quais convenções? |
| Sensor D0 | Divergência explícita e oráculo independente | O código realiza a operação especificada? |
| Recuperação direta | Busca e entrega de trechos da revisão declarada | A evidência encontrada chega ao agente e permanece legível? |
| Manutenção | Invalidação, atualização incremental e retomada | A política preserva validade e reduz trabalho desnecessário? |
| Estrutura | Relações com evidência e projeções identificadas | A representação factual é fiel no escopo suportado? |
| Ranking experimental | Comparação de estratégias, incluindo difusão | A complexidade acrescenta evidência útil sob orçamento? |
| Integração MCP | Adaptador do mesmo serviço de domínio | A ferramenta pública chama o mecanismo testado? |
| Validação de produto | Tarefas reais e qualidade/custo | O benefício alcança o usuário e generaliza no escopo anunciado? |

MCP pode ser planejado após o núcleo direto utilizável; a tabela não estabelece que toda pesquisa térmica deve terminar antes de oferecer uma interface. Ensaios de manutenção que dependam de qualidade de recuperação serão integrados quando esse instrumento existir, sem bloquear a conformidade inicial de D0.

## 17.4 O que significa uma entrega pronta

Uma entrega pronta identifica a versão, a capacidade, o caminho público ou CLI pertinente, os testes realmente executados, as limitações e o próximo problema. Não basta entregar uma spec intermediária sem comportamento observável. Também não se exige prova comercial do sistema inteiro para aprovar uma biblioteca de snapshots.

Um algoritmo experimental pode ser disponibilizado explicitamente como tal depois de conformidade e integração. Torná-lo padrão ou anunciar superioridade requer evidência adequada. A separação entre “utilizável” e “eficaz comparativamente” é intencional.

# 18. Valor comercial, investimento e originalidade

## 18.1 Valor que pretendemos demonstrar

O objetivo é reduzir exploração improdutiva, fornecer contexto necessário, manter evidências atuais e melhorar a relação qualidade/custo do trabalho com IA. A hipótese comercial discutida é poder anunciar economia mensurada de tokens ou custo preservando qualidade, e não simplesmente divulgar uma matemática sofisticada. [S-HIST; S-CONV]

A visão não pressupõe que todo ganho venha da difusão. Pode vir de sementes melhores, indexação, leitura atual, relações, seleção ou atualização. O produto é bem-sucedido quando entrega valor verificável, mesmo que uma hipótese específica seja abandonada.

## 18.2 Contabilidade que sustenta uma alegação

Custo por tarefa correta pode ser expresso como custo total do conjunto dividido pelo número de tarefas corretas, preservando no numerador o gasto com falhas. Com zero tarefas corretas, não se deve fabricar um custo favorável por divisão ou omissão.

O vetor de custo precisa separar entrada, entrada em cache, saída, raciocínio quando disponível, ferramentas, processamento local, indexação, manutenção e latência. Os contratos do provedor determinam quais campos são subconjuntos; não somar cache ou raciocínio duas vezes. Tokens do pacote podem já estar incluídos na entrada observada. Preços de API não são automaticamente o custo de uma assinatura ou cota. [S-P23, §15]

O resultado deve dizer exatamente o que diminuiu. Menos tokens não prova menos ruído. Menos chamadas não prova melhor raciocínio. Uma média de custo menor com regressão importante de qualidade não é vitória automática.

## 18.3 Investimento e divulgação

A conversa distinguiu iniciar relações exploratórias de captar sustentado em desempenho comprovado. Os resultados disponíveis justificam investigação, mas não alegações gerais de superioridade nem apresentação do primeiro whitepaper como validação integral. A reconstrução deve produzir evidência que suporte um piloto externo e comparação útil antes de ampliar a narrativa comercial.

Não foram escolhidos modelo de negócio, licença, preço, calendário de captação ou estrutura societária para o novo Compass. Essas escolhas não serão inventadas neste documento.

## 18.4 Originalidade e trabalhos relacionados

Foram citados historicamente Aider Repo Map, GraphCoder, RepoGraph, RepoHyper, Graph Diffusion Convolution, recuperação por heat kernel, MMR/RRF, RepoBench e SWE-bench como referências ou comparadores. O registro dessa discussão é preservado; **esta edição não é um levantamento atualizado e exaustivo de anterioridade**. [S-CONV; S-P23, §25]

Grafos, laplacianos, difusão e solvers conhecidos não devem ser apresentados isoladamente como descoberta original. A eventual contribuição do Compass precisa ser delimitada na representação, integração, política ou resultado demonstrado. Referências inspiram hipóteses; não substituem experimento próprio nem autorizam copiar implementações sem análise pertinente.

# 19. Registro consolidado de decisões e debates

## 19.1 Decisões vigentes

A numeração abaixo organiza esta edição. **ADR-001** preserva o ID já usado na escolha da linguagem. Os IDs DEC são referências editoriais para localizar as decisões, não novos atos de aprovação.

| ID | Decisão preservada | Seções |
|---|---|---|
| DEC-01 | Reconstruir o software integralmente em repositório independente. | 1, 3, 17 |
| DEC-02 | Preservar o legado como história; não condicioná-lo a uma nova refatoração. | 3 |
| DEC-03 | Não importar código, instruções, skills ou roadmaps legados automaticamente. | 3, 16 |
| DEC-04 | Produto é infraestrutura de contexto, não apenas localizador de arquivos. | 1, 5 |
| DEC-05 | Avaliar relevância, suficiência, atualidade e eficiência separadamente. | 5, 12 |
| DEC-06 | Proveniência e limites explícitos fazem parte do produto. | 5–7 |
| DEC-07 | Distinguir manutenção, recuperação e consistência. | 6 |
| DEC-08 | Drift baixo não legitima evidência sabidamente obsoleta. | 6–7, 13 |
| DEC-09 | Operação local e preocupação com uso em servidor fazem parte da visão. | 5, 13 |
| DEC-10 | Leveza/controle de recursos é requisito de primeira classe. | 13 |
| DEC-11 | Modelos matemáticos são meios; nenhuma sofisticação é finalidade do produto. | 9–12 |
| DEC-12 | Difusão é complementar à recuperação tradicional, não sua substituição obrigatória. | 6, 10 |
| DEC-13 | Condutâncias são hipótese, sem adoção automática. | 10–11 |
| DEC-14 | Separar fidelidade matemática e eficácia empírica. | 9, 12 |
| DEC-15 | Implementação não define seu próprio gabarito. | 12, 15 |
| DEC-16 | Critérios experimentais não mudam silenciosamente após observar resultados. | 12 |
| DEC-17 | Resultados históricos são preservados com alcance, não descartados nem universalizados. | 3–4 |
| DEC-18 | Rust é a linguagem do motor, conforme ADR-001. | 8 |
| DEC-19 | Python é laboratório independente, não outro motor de produção. | 8, 12 |
| DEC-20 | Bibliotecas/modelos precisam satisfazer contratos; não estão todos escolhidos. | 8 |
| DEC-21 | Foundation começa por estados/revisões/snapshots e comparação. | 7, 17 |
| DEC-22 | CLI primeiro; MCP depois, chamando o mesmo núcleo. | 8, 17 |
| DEC-23 | Documento principal completo e estado operacional curto têm responsabilidades distintas. | 16 |
| DEC-24 | Material de estudo deve ser autossuficiente, progressivo e revisável em papel. | 16 |
| DEC-25 | Perguntas comparativas do idealizador acrescentam variáveis; não indicam preferência presumida. | 8, 15 |
| DEC-26 | Alegações de superioridade e economia exigem evidência compatível. | 12, 18 |

## 19.2 Debates encerrados ou reformulados

| Debate | Trajetória preservada | Situação atual |
|---|---|---|
| Refatorar ou reconstruir | Houve resistência inicial, reconstrução limitada proposta e cobrança de coerência | Reconstrução integral decidida; não reabrir por inércia do legado. |
| Teoria “já sólida” | Fundamentação matemática conhecida foi confundida com validação do produto | Operadores podem ser sólidos; utilidade e integração seguem testáveis. |
| “O motor nunca foi testado” | Crítica à correspondência teoria–execução | Componentes foram testados; validação integral da intenção continua insuficiente. |
| D0 depende de recuperar tudo | Dependência prática motivou a expansão de escopo | Cálculo textual independente; avaliação de impacto exige recuperação. |
| Difusão versus busca | A formulação parecia substituir ferramentas tradicionais | Relevância estrutural complementa descoberta e seleção. |
| BFS inevitavelmente imprecisa | Distância simples pode empatar candidatos distintos | Comparar com expansão competitiva; não presumir vitória térmica. |
| “Material” do calor | Pesos uniformes podem apagar diferenças de relações | Condutância é hipótese controlável; capacidade dinâmica não é automaticamente útil ao equilíbrio. |
| Rust versus alternativas | Comparação genérica evoluiu para requisitos do produto | Critérios fechados e Rust eleita; notas não são benchmark. |
| Apostila completa ou piloto | Piloto proposto para calibrar didática | Obra única aceita, com modularidade interna e revisão. |
| Mais documentos para organizar | Planos novos repetiam decisões antigas | Uma autoridade por assunto e matriz de cobertura. |
| Corrigir os defeitos antigos antes de seguir | Auditoria ameaçava reiniciar o ciclo | Arquivar, não sanear; novo código nasce independente. |

## 19.3 O que permanece aberto

| Pendência | Por que importa | Momento de resolução |
|---|---|---|
| Snapshot integral do legado executado? | O relatório apenas propôs a preservação. | Antes de apagar/mudar o objeto antigo. |
| Contrato exato de estados e observação | Base de D0, invalidação e autenticidade. | Foundation. |
| Arquivos/linguagens elegíveis e política de links | Evita omissões e leitura indevida. | Foundation e primeiro indexador. |
| Fórmula D0 da nova versão | Proxy escolhida deve corresponder ao objetivo. | Após estados; antes do sensor. |
| Especificação DG e falha parcial | Não herdar duas fórmulas incompatíveis. | Antes de integrar DG. |
| Consistência durante query/update | Repositório pode mudar enquanto é consultado. | Antes de publicação/consulta incremental. |
| Modelo, tokenizer e backend vetorial | Afetam qualidade, recursos e invalidação. | Antes da busca vetorial. |
| Grafo factual e projeções | Granularidade, direção, pesos e evidências. | Antes da recuperação estrutural. |
| Operador de ranking e parâmetros | Não escolher por nomenclatura “calor”. | Contrato/ensaio de ranking. |
| Condutância contextual | Método de pesos ainda não definido. | Experimento separado de COND-001. |
| Limites de CPU/RAM/latência/disco | Não há máquina/carga de referência fechada. | Perfil de desempenho antes de release. |
| SDK, versões e targets de distribuição | Existência da biblioteca não prova integração. | Contrato técnico de cada adaptador. |
| Gabaritos independentes e holdout | Evitar reciclar tarefas conhecidas como inéditas. | Antes da campanha confirmatória. |
| Critérios de benefício e não inferioridade | Precisam refletir utilidade e variabilidade. | Pré-registro do experimento. |
| Modelo comercial e proteção intelectual | Não foram decididos nesta reconstrução. | Frente estratégica própria. |

Nenhuma dessas pendências muda a linguagem por si só. Elas indicam quais contratos faltam, não uma autorização para o agente inventar valores e declarar o projeto fechado.

# 20. Orientação para o primeiro trabalho após esta edição

O próximo trabalho é **redigir e conferir o contrato de estados e revisões do Compass Foundation**, usando as decisões acima. Não é implementar todo o motor, importar a apostila como spec executável ou lançar outra auditoria geral do legado.

O contrato deve permitir responder: o que foi observado; como os arquivos e revisões são identificados; qual a abrangência e a consistência da observação; quais diferenças existem; como erros e leituras incompletas são representados; quais casos independentes verificam essas regras.

Com esse contrato aprovado, o agente pode construir o núcleo e a CLI dentro do escopo autorizado. A entrega deve apresentar um comportamento demonstrável e sua identidade de versão. Não há necessidade de esperar prova de utilidade da difusão para comparar dois snapshots.

**Ponto de partida consolidado:** preservar conhecimento sem preservar a confusão; reconstruir código sem reiniciar do zero o raciocínio; implementar contratos sem ajustar a ciência à saída; medir benefício sem fabricar um vencedor.

# Apêndice A. Resultados históricos preservados

## A.1 Cobertura PostSpark por tarefa

Valores em porcentagem; diferenças expressas em pontos percentuais. Esta tabela reproduz a cobertura relatada no whitepaper e confirmada no caderno, não uma nova campanha. [S-WP, §5; S-CAD, pp. 84–85]

| Tarefa | B | C0 | C1 | C1−B |
|---|---:|---:|---:|---:|
| T01 — snap e âncoras | 0,0 | 66,7 | 100,0 | +100,0 |
| T02 — histórico | 0,0 | 0,0 | 0,0 | 0,0 |
| T03 — teclado | 0,0 | 100,0 | 100,0 | +100,0 |
| T04 — coordenadas | 66,7 | 0,0 | 0,0 | −66,7 |
| T05 — limites geométricos | 33,3 | 0,0 | 0,0 | −33,3 |
| T06 — store transitória | 33,3 | 66,7 | 66,7 | +33,3 |
| T07 — grade | 66,7 | 0,0 | 0,0 | −66,7 |
| T08 — bloco arrastável | 0,0 | 0,0 | 0,0 | 0,0 |
| T09 — tipografia | 0,0 | 100,0 | 100,0 | +100,0 |
| T10 — fontes dinâmicas | 0,0 | 100,0 | 100,0 | +100,0 |
| T11 — composição IME | 0,0 | 100,0 | 100,0 | +100,0 |
| T12 — menu móvel | 66,7 | 0,0 | 0,0 | −66,7 |
| **Média** | **22,2** | **44,4** | **47,2** | **+25,0** |

O ganho C1−C0 é 1/3 em T01, dividido por 12 tarefas: $1/36=2,777\ldots$ pontos percentuais. Na rubrica histórica, equivale a um critério adicional adquirido naquele contraste agregado. A melhora relativa sobre 44,44% é 6,25%, não 2,78%. A chance de uma reamostra bootstrap de 12 tarefas não incluir T01 é $(11/12)^{12}\approx0,352$; isso ajuda a interpretar o limite inferior zero, sem garantir ausência de perdas futuras.

## A.2 Resultados distintos que não devem ser somados

| Registro | Objeto/contagem | Uso admissível |
|---|---|---|
| Setembro, 12 sessões | Seis tarefas pareadas; uso livre de ferramentas | Observação histórica de custo/qualidade naquele protocolo. |
| C1–C6 | Grafos, solves, visitas e pares de fluxos de naturezas diferentes | Testes de mecanismos e limites, não uma amostra única. |
| Grafo v2 de 10/10 | Primeiro log: 34 pass, 2 fail; regressão posterior: 7 pass | Sequência histórica de correção de contratos; não transformar o primeiro log em 36 pass. |
| Recorte real grafo v2 | 30 ocorrências; 84 nós/86 pares no recorte relatado | Integração delimitada; 30 ocorrências não são 30 arestas únicas. |
| Auditoria de 09/10 | 25 pass, dos quais 10 atuais e 15 históricos | Não contar cópias históricas como cobertura adicional do produto atual. |
| Já Comprei | 36 sessões; 165 requisições com loop acumulativo | Evidência daquele loop, com limitações de independência. |
| PostSpark | 72 sessões; 12 tarefas; 144 requisições | Aquisição sob duas rodadas; não 144 tarefas independentes. |
| Missão 00 | Leitura estática e hashes | Inventário; nenhuma nova homologação funcional. |

Fontes: [S-P23, §2; S-CAD, caps. 16–17; S-R00, §§6.1–6.2].

## A.3 Classificação dos números de tokens

A redução de tokens de ferramenta de C1 contra C0 foi aproximadamente 24,65%; contra B, C1 entregou mais tokens de ferramenta. As contagens publicadas de entrada/saída não são mantidas como corretas por apresentarem deltas parecidos com os recalculados. O arquivo histórico é preservado; a correção aparece explicitamente nesta edição, seguindo o caderno.

Não há inferência de que todos os tokens retirados eram supérfluos, de que os nove outputs parciais não continham nada útil ou de que o preço da campanha representa o custo total de rodar o Compass em uma máquina/servidor.

# Apêndice B. Formas matemáticas e parâmetros históricos

## B.1 Catálogo dos mecanismos encontrados, não adotados por padrão

| Família | Operação descrita | Lugar histórico |
|---|---|---|
| D0 textual | Média ponderada de divergências de linhas | TypeScript e forma parecida no Python; significado “semântico oficial” excedia o observado. |
| DG composto | Relações + resumo espectral, com modo parcial | TypeScript. |
| DG Jaccard | Diferença global de conjuntos de arestas | Python paralelo. |
| Reinício discreto | $(1-\gamma)s+\gamma AD^{-1}h$ na convenção descrita | Módulo anterior de difusão. |
| Heat kernel | $e^{-\tau L}s$ aproximado | Perfis/diagnósticos de calor livre. |
| Reinjeção estacionária | $(L+\beta I)h=\beta s$ | `compass_query` v3 quando calor habilitado. |
| Backend Python | `expm_multiply`, parada por diferença de energia, $H_0+\alpha H$ | Servidor alternativo; uso atual não verificado. |

Fonte: [S-R00, §§3–4]. Este catálogo impede uma migração silenciosa de nomenclatura; não obriga implementar todas as famílias em Rust.

## B.2 Parâmetros v3 e de manutenção observados

| Parâmetro/regra | Valor histórico | Advertência |
|---|---|---|
| Calor v3 | Opt-in por `enable_heat` | Omissão não significa solver ativo. |
| $\beta$ e tolerância | 1 e $10^{-6}$ | Escolhas observadas, não defaults científicos universais. |
| `seed_limit` | Completa até 3 após explícitos | O relatório diz todos os explícitos; não é teto universal de três sementes. |
| Expansão | Componentes alcançadas; solve no grafo inteiro | Não há subgrafo ativo otimizado demonstrado na missão. |
| Orçamento v3 | 3.000 tokens quando omitido | Não é o padrão de todas as interfaces. |
| Defaults anteriores | `9007199254740991` em configuração | Não traduzir esse valor em orçamento recomendado do novo produto. |
| Python | 2.000 tokens por default | Outra interface e outro contrato. |
| Cursor v3 | Memória; TTL 15 min | Não garante persistência após reinício. |
| Manutenção | 0,25 / 0,15 / 0,60; saída 0,9 | Histerese/política histórica, sem calibração universal. |
| Embeddings TS | multilingual-e5-small q8 CPU; 384; janela 512 | Disponibilidade atual não foi testada pela Missão 00. |
| Embeddings Python | MiniLM multilíngue e Chroma | Backend diferente, não segunda fonte de verdade. |

Fonte: [S-R00, §§3–4]. Modelos, bibliotecas e limites novos continuam sujeitos a decisão própria.

## B.3 Controlador adaptativo proposto em setembro

O plano consultado de 23/09 escolhia âncoras por caminhos/símbolos/lexical/vetorial, até quatro canais térmicos independentes, grafo de imports unitário, seleção da primeira janela admissível, reranking e entrega progressiva. A finalidade era separar exploração estrutural de contexto realmente entregue. **Sua orientação de refatoração e seus defaults foram superados como instrução de implementação**, embora as ideias permaneçam registradas. [S-P23, §§1–13]

Os cinco critérios de janela eram coerência interna, separação externa, retenção, estabilidade e pertinência. Tratava-se de conjunção, não média que compensasse ausência de pertinência por alta homogeneidade. O plano também proibia fabricar estabilidade repetindo checkpoints quase iguais ou fabricar retenção deixando o conjunto de nós crescer.

| Parâmetro/regra do plano | Default proposto à época |
|---|---|
| Candidatos diretos por lista | 100 |
| RRF | $k=60$ |
| Âncoras diretas/canais térmicos | 8 / 4 |
| Sweep | Prefixos 2, 4, 8, 16, 32, 64, 128, limitados pelo componente |
| Coerência | Pelo menos 2 nós; $CV^2\le1$ e condições de validade |
| Separação | Contraste de fronteira $\ge1/3$ |
| Retenção | Massa $\ge0,50$; perda máxima 10% na janela |
| Estabilidade | Jaccard $\ge0,80$ por três checkpoints separados |
| Pertinência | Ganho não-semente $>0,02$, com contribuição não redundante |
| Passo inicial/intervalo de confirmação em $u$ | 0,125 |
| Crescimento do passo/máximo | 1,5 / 8 |
| Horizonte/checkpoints por canal | $u_{max}=128$ / 64 |
| Piso de contraste | $E_{crit}=10^{-4}E(0)$ |
| Passo mínimo | $10^{-4}$ |
| Critério “morno” | $TV\le0,10$ no componente |
| Trabalho/tempo/memória adicionais | 50 milhões de unidades / 2 s / 64 MiB |
| Tolerância do sinal | $10^{-6}$ em L2, sem certificação intervalar formal |
| Primeira página/adicional | 4.096 / 8.192 tokens |
| Máximo por resposta/configurável | 16.384 / 65.536 tokens |
| Limite de transporte | 1 MiB |

O objetivo de seleção proposto era:

$$
G(c\mid P)=0{,}60\,rel(c)+0{,}25\,\Delta coverage(c,P)
+0{,}15\,bridge(c,P)-0{,}35\,redundancy(c,P).
$$

Esses pesos e termos eram heurísticos de ordenação, não probabilidades de verdade. Calor não era somado diretamente ao cosseno bruto. O plano discutia novidade de candidatos, pontes, acessibilidade de itens adiados, cursores, planos-filhos e separação entre selecionado, entregue e usado. Preservamos o motivo de cada mecanismo sem adotá-lo automaticamente. [S-P23, §§11–15]

A matriz offline histórica cruzava expansão Off/fixa/adaptativa com seleção antiga/nova, e previa testes separados de âncoras e packing. Não é a matriz confirmatória obrigatória do novo motor. Não será reinstalada para repetir uma arquitetura já superada.

## B.4 Backlog completo identificado no plano consultado

Preservam-se como alternativas históricas: laplaciano normalizado simétrico; PPR/reinício; pesos por tipo/direção; massa da semente proporcional ao score; grafo local com fronteira correta; cross-encoder; classificador de intenção; reação-difusão; detector de change-point; curvatura/joelho; limiar por família; ganho por token; múltiplas regiões por canal; GNN; agente julgando candidatos; aprendizado online de pesos; otimização conjunta de preço/modelo/recuperação. A busca de troca de sinal de $E''$ foi rejeitada no modelo simétrico linear fixo. [S-P23, §21]

Registrar esse backlog impede perder uma intuição. Não cria obrigação de implementá-lo nem razão para postergar a Foundation.


## B.5 Observáveis de fronteira e tempo do plano histórico

Para preservar a discussão sem tornar seus critérios normativos, registram-se também as expressões principais. Num conjunto fixo de nós $S$, o plano definia $M_S=\sum_{i\in S}H_i$ e:

$$
\frac{dM_S}{d\tau}=-\sum_{i\in S,j\notin S}w_{ij}(H_i-H_j)=F_{in}-F_{out}.
$$

A saída positiva soma $\max(H_i-H_j,0)$ na fronteira; a entrada soma o sentido oposto. Coerência era acompanhada por $CV^2=\mathrm{Var}(H_S)/\mathrm{mean}(H_S)^2$, com variância pelo tamanho da região e denominador afastado de zero. A separação usava a razão entre soma das diferenças e soma dos valores dos extremos na fronteira. A condutância do corte era $\mathrm{cut}(S,\bar S)/\min(\mathrm{vol}(S),\mathrm{vol}(\bar S))$, quando os volumes permitiam essa divisão. [S-P23, §8]

No componente conectado combinatorial, com massa 1, a proximidade ao equilíbrio era:

$$
TV(\tau)=\frac12\sum_i\left|H_i(\tau)-\frac1n\right|.
$$

A relação histórica $x=\tau_{cut}$ e $y=\tau_{warm}-\tau_{cut}$ distinguia ponto de seleção e tempo restante até um critério operacional. Se o processo não observasse esse último tempo, $y$ deveria permanecer desconhecido/censurado. Uma previsão por energia, como $(E-E_{crit})/(-E')$, não determina por si a região espacial nem o horizonte até a TV escolhida. [S-P23, §§7–9]

O plano também usava a taxa relativa $r=-E'/E$ quando $E>0$, com derivada não positiva sob o modelo espectral fixo. Essas propriedades ajudam a descrever regimes; não descobrem uma fronteira semântica sozinhas. A computação proposta evitava formar $L^3$ densamente: aplicava o operador em passes esparsos. A reconstrução pode estudar esses observáveis, mas não assume que uma janela térmica válida existe ou é útil em toda tarefa.

# Apêndice C. Matriz de cobertura da consolidação

A matriz abaixo foi usada para conferir os tópicos identificados. “Incluído” não quer dizer validado ou implementado. Onde uma fonte está incompleta, a própria seção registra o limite.

| Tema/decisão da conversa e fontes | Local nesta edição | Tratamento |
|---|---|---|
| Origem por diferença entre massas/estados | 2.1, 7, 9.2 | Conceito e limites incluídos. |
| $S_0$, $S_t$, $\Delta$, baseline e reversão | 2, 7, 9 | Separados de atividade acumulada. |
| RAG como bússola; código vivo como conferência | 2.1 | Motivação histórica incluída. |
| Necessidade de recuperação e dependência não absoluta | 2.2, 6 | Debate preservado. |
| SDS, novelty, dupla contagem e pontos cegos | 9.3 | Formulação primária incompleta sinalizada. |
| Representação dual, vetores e grafos | 2.3, 6, 9.5 | Responsabilidades separadas. |
| D0 textual e pesos logarítmicos | 9.2 | Fórmula histórica, não escolha nova. |
| DG relacional/espectral e diferenças Python/TS | 3, 9.4, B | Incompatibilidade preservada. |
| Homotopia/deformação versus ruptura | 9.4 | Intuição, não propriedade certificada. |
| Langevin, diffs e flutuações | 11.3 | Hipótese sem modelo operacional aprovado. |
| Laplaciano, Dirichlet e normalização | 9.5–9.10 | Pressupostos explícitos. |
| Calor livre versus reinjeção | 4.5, 9.7–9.9 | Divergência não apagada. |
| Tempo físico/matemático versus PCG | 9, 10 | Distinção incluída. |
| Ordem certificada, resíduo e precisão | 9.9, 12 | Não confundidos com relevância. |
| Difusão não substitui lexical/vetorial | 6, 10 | Decisão vigente. |
| Ordenação contínua e seleção discreta | 10.1 | Ambiguidade resolvida. |
| BFS e PPR como baselines | 9.10, 12.4 | Comparação futura, não vitória assumida. |
| Hubs, isolados e sementes erradas | 10.2, 12.5 | Limites e negativos incluídos. |
| Material, condutância e capacidade | 10 | Hipótese e exemplo conferidos. |
| Escala dos pesos versus $\beta$ | 10.4 | Controle experimental incluído. |
| Fronteiras, cortes, regiões e packing | 10.7, B.3 | Sem equivalência ingênua de subgrafos. |
| Derivadas de energia e inflexão impossível | 9.7, B | Rejeição sob hipóteses explícitas. |
| C1–C6 e heterogeneidade experimental | 4.2, A | Contagens não agregadas como réplica. |
| Setembro, cache e custo | 4.3, 18.2 | Custo observado separado de tokens. |
| Já Comprei e contexto acumulado | 4.4, A | Limitação não transferida a outra campanha. |
| PostSpark, tabela por tarefa e IC | 4.5, A | Dados e incerteza preservados. |
| Tokens publicados versus recalculados | 4.5, A.3 | Dois registros explícitos. |
| Nove outputs incompletos | 4.5, A | Não excluídos retroativamente. |
| Baseline em uma rodada e C0 ambíguo | 4.5, 12.4 | Atribuição causal limitada. |
| Missão 00, hashes e snapshot pendente | 3 | Levantamento não virou homologação. |
| Múltiplas rotas/perfis/defaults | 3, B | Não são uma implementação única. |
| Supervisor, lease, interpretação e telemetria | 3, D | Achados com escopo preservado. |
| Código não commitado e aliases históricos | 3, 15.4 | Risco de identidade/contexto incluído. |
| Reconstrução do zero, não refatoração | 1, 3, 19 | Direção encerrada preservada. |
| Teoria versus código versus instrumento | 3, 12 | Cadeia de conformidade incluída. |
| Exemplo $1+1=9$ e oráculo independente | 1.4, 12 | Regra fundamental. |
| Mutação, testes negativos e propriedades | 12.3 | Estratégias registradas. |
| Hipóteses DRIFT/RETR/HEAT/COND/AGENT | 11 | Estados científicos separados. |
| H0/H1 inicial de correlação e perda | 11.2 | Contexto recuperado, não protocolo ativo. |
| Piloto, holdout e critérios antes dos resultados | 12 | Tamanhos sugeridos não viraram quotas. |
| Blueprint de produto e quatro capacidades | 1, 5 | Visão vigente. |
| Precisão, recall e suficiência | 5, 12.8 | Não equivalentes. |
| Invalidação imediata versus manutenção inteligente | 6–7 | D0 não é autoridade de atualidade. |
| Sob demanda/contínuo/local/servidor | 5, 13 | Direção e detalhes pendentes. |
| Recursos, memória vetorial e overhead | 13 | Cálculo ilustrativo e métricas. |
| Comparação Rust/C++/C#/Java/Python/Go/Julia | 8 | Razões e limites preservados. |
| Dez critérios e score da linguagem | 8.2 | Pesos e totais conferidos. |
| Rust vencedor; Python só verificação | 8 | Decisão vigente. |
| Bibliotecas candidatas não fixadas | 8.4 | Sem dependência inventada. |
| Papel humano, agentes e perguntas sem preferência | 15 | Preferência do idealizador preservada. |
| Documento canônico e estado operacional curto | 16 | Responsabilidades reconciliadas explicitamente. |
| Apostila didática/única/revisão em papel | 16.3 | Decisão editorial preservada. |
| Foundation e CLI antes de MCP | 17 | Primeiro escopo fechado. |
| Segurança, privacidade e propriedade intelectual | 14, 18 | Sem promessa jurídica/técnica indevida. |
| Investimento e alegações comerciais | 18 | Evidência antes da divulgação de eficácia. |
| Trabalhos relacionados | 18.4, fontes | Registro histórico, não busca exaustiva. |
| Pendências e decisões superadas | 19 | Visíveis, não preenchidas silenciosamente. |

# Apêndice D. Achados adicionais e correções de interpretação

## D.1 Fronteira do interpretador

A Missão 00 localizou um perfil opcional em que o interpretador recebia catálogo local de módulos/declarações e snippets, enquanto o blueprint dizia limitar-se ao pedido/contexto explícito. O achado não foi atribuído a `compass_query`, que não chamava esse interpretador. Essa é uma divergência arquitetural localizada, não prova genérica de vazamento de rubrica nem de dependência de LLM em toda consulta. [S-R00, §6]

## D.2 Recibos que não significam o que o nome sugere

`completed`, `hybrid`, `converged`, “semântico”, “oficial”, “grafo estático”, “holdout” e “snapshot” apareceram com escopos que precisaram ser corrigidos ou delimitados. A lição é definir cada termo por condição verificável. Nenhum rótulo permite concluir mais que seu contrato.

Na Missão 00, abrir o workspace poderia criar diretórios, configurar WAL ou aplicar migrations. Por isso, não se usaram interfaces do próprio produto como se fossem leitores neutros do estado. Uma próxima auditoria de runtime deve preservar o objeto e explicitar efeitos colaterais. [S-R00, §§3 e 6]

## D.3 Igualdade de hashes não resolve toda a proveniência

O relatório descreve 28 coincidências e quatro diferenças em um manifesto de 32 entradas do pacote de grafo de 10/10, além de coincidência no conjunto posterior de 611 arquivos. Esses conjuntos têm escopos diferentes; não se conclui que um contradiz o outro. Hash identifica bytes e deve acompanhar o perímetro e a data. [S-R00, §2.1]

Há também uma distinção documental verificada nesta entrega: o plano de 23/09 recuperado da Biblioteca tem **96.901 bytes**, enquanto a Missão 00 lista uma versão local de mesmo nome com **118.135 bytes**. Não são a mesma revisão. Esta consolidação trata o plano recuperado como fonte histórica própria, e o relato sobre a versão local como informação da Missão 00; não inventa uma comparação de conteúdo entre os dois.

## D.4 Critérios científicos não são contratos jurídicos nem garantias comerciais

A palavra “validado” só deve aparecer com objeto, versão, método e escopo. A crítica ao whitepaper não autoriza concluir fraude, má intenção, impossibilidade do Compass ou fracasso de todas as hipóteses. Também não autoriza investir com base em um resultado que ainda não sustenta a alegação. A decisão continua sendo produzir experimentos capazes de confirmar e contrariar o mecanismo.

# Apêndice E. Glossário e convenções

| Termo | Uso nesta edição |
|---|---|
| Acervo/repositório | Conteúdo observado sob política; pode incluir alterações fora de commits. |
| Snapshot | Representação identificada de uma observação; sua consistência precisa de contrato. |
| Revisão | Identidade de um conteúdo/estado específico, não apenas nome de arquivo. |
| Geração de índice | Conjunto publicado de derivados e suas dependências/versionamentos. |
| Derivado | Artefato como vetor, índice, relação ou projeção produzido de uma fonte. |
| RAG | Recuperação de conteúdo para compor contexto de uma geração. |
| Embedding | Representação vetorial produzida por modelo/configuração identificados. |
| AST/CST | Estruturas sintáticas; não garantem resolução semântica completa. |
| Grafo factual | Relações extraídas com significado, direção e evidência. |
| Projeção | Grafo/operador preparado para determinado algoritmo. |
| Semente | Ponto inicial de relevância; não é verdade fornecida pelo gabarito. |
| Laplaciano | Operador de diferenças no grafo sob convenção declarada. |
| Dirichlet | Energia de diferenças entre nós; não medida de verdade semântica. |
| PCG | Método numérico de gradientes conjugados precondicionados. |
| Resíduo | Diferença entre lados da equação usando a solução aproximada. |
| Condutância de aresta | Peso que modula acoplamento; não confundir com condutância de corte. |
| Drift | Divergência entre estados segundo uma representação/regra. |
| Invalidação | Retirada de uma alegação de validade de um derivado afetado. |
| Reconciliação | Conferência do estado observado contra o estado conhecido. |
| Overlay | Representação de alterações recentes; mecanismo histórico/candidato, não schema novo fechado. |
| Packer/empacotador | Seleciona/serializa evidências sob orçamento e contrato. |
| Proveniência | Fontes, revisões e transformações que explicam a origem do resultado. |
| Oráculo | Referência de resultado esperado, independente da implementação testada. |
| Ablação | Experimento que altera uma contribuição identificada, controlando as demais. |
| Holdout | Dados efetivamente reservados do desenvolvimento no escopo declarado. |
| Histerese | Condições distintas de entrada/saída para reduzir oscilação; não garantia universal de segurança. |
| CLI / MCP | Interfaces de acesso ao motor; não motores independentes. |
| MB / MiB | Milhões de bytes / unidades de $2^{20}$ bytes. |
| $t$, $\tau$ e $\beta$ | Tempos/escala e parâmetro do modelo; não tempo de CPU ou contagem de tokens. |
| $D_0$, $D_G$ | Indicadores sob contrato; não probabilidades de erro nem certificados de atualidade. |

# Apêndice F. Fontes, proveniência e verificações desta edição

## F.1 Fontes internas e conversacionais

**[S-CONV] Conversa de direção do Compass D0 disponível até 14/10/2026.** Fonte das decisões sobre reconstrução, visão do produto, critérios, Rust, papéis dos agentes, documentação, recursos e próximos passos. É registro de intenção/decisão e debate; não substitui código, experimento ou auditoria. Datas exatas não visíveis de cada passagem não foram inventadas.

**[S-HIST] Contexto recuperado das discussões iniciais.** Foram recuperadas referências à conversa de 14/08/2026 sobre arquitetura híbrida RAG, média ponderada e hipótese de perda; às discussões de setembro sobre Evidence/Drift Engines e valor comercial; e às preocupações de proteção intelectual. Esse material é resumo de conversas anteriores, não exportação integral. Definições incompletas de SDS e a modelagem de Langevin foram sinalizadas como lacunas, não reconstruídas como contratos.

**[S-P23] `PLANO-MESTRE-REFATORACAO-COMPASS-D0.md`, versão recuperada da Biblioteca.** Data interna 23/09/2026; 1.052 linhas no texto disponibilizado. Arquitetura proposta `compass-adaptive-v2`, com fundamentos, parâmetros, avaliação e backlog. Sua orientação de refatoração não rege o novo produto. Tamanho conferido: 96.901 bytes. SHA-256:

```text
fa8720a8fdc58a91f2cb1de474e1f9bdeec533ecbda343dbce185e4bea6c55de
```

**[S-WP] `WHITEPAPER.md`, 11/10/2026.** Título: *Compass D0: Avaliação Empírica de Recuperação Topológica de Contexto com Difusão Térmica em Bases de Código Reais*. Campanha `campanha-postspark-whitepaper-20261011T045639Z`. É o documento original, com alegações posteriormente restringidas pelo caderno. Tamanho conferido: 23.004 bytes. SHA-256:

```text
7a9ab4979a6d7ba889d26d61b4d595e0e717c9316251e2ff7344353bc919eb68
```

**[S-CAD] `CADERNO_COMPASS_D0.pdf`, edição de 11/10/2026.** 138 páginas. Fonte didática e auditoria secundária do legado, com recálculos documentados e limites explícitos. Foram utilizados especialmente capítulos 1–2, 6–9 e 14–20, além das páginas visuais 37–38 e 87–88. Tamanho conferido: 684.944 bytes. SHA-256:

```text
7543190da1cfcc5523ff35913628375ca1596b857e3d03f8bbd7a45ff9e74a9b
```

**[S-R00] `RELATORIO.md`, Missão 00, 14/10/2026.** Inventário operacional provisório, 4.321 linhas no texto disponibilizado, com apêndices de Git e hashes. Não executou validação funcional. O SHA-256 abaixo identifica o relatório recebido, não a árvore inteira do projeto. Tamanho conferido: 463.329 bytes.

```text
a72a0405596c50e117288d005ecd52d90ccc7a2217f8f6a4b5959846cada263d
```

As fontes locais foram preservadas como entradas de leitura. Não foram incorporadas como instruções executáveis do novo repositório nem incluídas como código de produção nesta entrega.

## F.2 Referências técnicas externas conferidas

As referências abaixo foram consultadas em 14/10/2026 apenas para conferir fatos técnicos pontuais citados na consolidação. **Não ampliam o escopo aprovado, não escolhem versões de dependências e não substituem avaliação do Compass.**

**[E-RUST] The Rust Programming Language — Understanding Ownership.** Base para a afirmação de segurança de memória por ownership sem coletor obrigatório. Não certifica a correção do motor.  
<https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>

**[E-MCP] Model Context Protocol — Rust SDK / `rmcp`.** Referência oficial do SDK Rust mencionado como candidato de integração posterior.  
<https://github.com/modelcontextprotocol/rust-sdk>

**[E-PARSER] Tree-sitter — Introduction / Using Parsers.** Parsing incremental e árvores sintáticas concretas; não promessa de resolução semântica universal.  
<https://tree-sitter.github.io/tree-sitter/>  
<https://tree-sitter.github.io/tree-sitter/using-parsers/index.html>

**[E-FAER] `faer::sparse` — documentação da biblioteca.** Estruturas esparsas por linhas/colunas e operações associadas. A biblioteca continua candidata, sujeita ao contrato numérico.  
<https://docs.rs/faer/latest/faer/sparse/index.html>

**[E-CG] SciPy — `scipy.sparse.linalg.cg`.** Condições do método, resíduo/tolerâncias e estado de saída.  
<https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.cg.html>

**[E-EXPM] SciPy — `scipy.sparse.linalg.expm_multiply`.** Ação da exponencial matricial, sem necessidade de formar uma exponencial densa na interface documentada.  
<https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.expm_multiply.html>

**[E-AOT] Microsoft Learn — Native AOT deployment.** Confirma a alternativa de C#/.NET autossuficiente e as limitações que impedem comparações simplistas. A referência não reabre ADR-001.  
<https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/>

## F.3 Referências históricas de pesquisa preservadas

O plano e a conversa citam Kloster/Gleich e `hkgrow`, MMR, RRF, *Lost in the Middle*, trabalhos de oversmoothing, Aider Repo Map, GraphCoder, RepoGraph, RepoHyper, Graph Diffusion Convolution, RepoBench e SWE-bench. A finalidade aqui é registrar as linhas de comparação levantadas, não afirmar que cada trabalho foi novamente revisado nesta edição. O mapeamento de anterioridade e a escolha do benchmark externo permanecem tarefas próprias. [S-P23, §25; S-CONV]

## F.4 Verificações efetivamente realizadas nesta consolidação

Foram conferidos os hashes e tamanhos dos quatro arquivos-fonte montados/recuperados; a agregação das notas de linguagem; as contas didáticas de D0 por arquivo; a solução do exemplo de condutância; o cálculo de armazenamento dos embeddings; e as contagens aritméticas derivadas da tabela histórica de cobertura.

Essas verificações não são execução do Compass, réplica de campanha, auditoria do código-fonte original ou comprovação de benefício. A geração do PDF inclui conferência de texto e inspeção visual de diagramação. Os resultados técnicos de geração constam do manifesto da entrega.

**Limites não resolvidos:** código/runtime atual do projeto não foi acessado nesta consolidação; os recibos primários de todas as campanhas não foram reprocessados; o backup integral não está confirmado; a transcrição completa de todas as conversas iniciais não está disponível; a revisão local do plano citada na Missão 00 não é o mesmo arquivo da Biblioteca; contratos novos ainda precisam de aprovação.

# Apêndice G. Folha de revisão do idealizador

Use esta folha para registrar mudanças sem transformar uma anotação em decisão automática. O ID pode ser uma seção, DEC, ADR ou hipótese. A decisão aprovada deve indicar a versão do documento/contrato que muda.

| Campo | Registro |
|---|---|
| Seção / ID examinado | ______________________________________________ |
| O que compreendi | ______________________________________________ |
| Possível inconsistência | ______________________________________________ |
| Fonte a conferir | ______________________________________________ |
| É erro de texto, hipótese, código ou experimento? | ______________________________________________ |
| O que mudaria minha decisão? | ______________________________________________ |
| Decisão proposta / ainda pendente | ______________________________________________ |
| Aprovação e data | ______________________________________________ |

\vspace{1em}

**Anotações**

\vspace{1em}\hrule\vspace{2.1em}\hrule\vspace{2.1em}\hrule\vspace{2.1em}\hrule\vspace{2.1em}\hrule

\vspace{1em}

> **Regra de encerramento:** esta edição consolida o que foi decidido e deixa visível o que ainda precisa ser decidido. Seu objetivo não é provar que o Compass estava certo, mas permitir que a próxima implementação seja compreensível, fiel e capaz de produzir evidência confiável — favorável ou contrária às hipóteses.
