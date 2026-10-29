# Análise inicial e plano do Compass Foundation

> Este documento registra o diagnóstico anterior à implementação. As etapas F0–F4 foram implementadas após autorização, e os resultados atuais estão no [estado operacional](../compass-foundation/COMPASS_STATE.md). A edição original analisada permanece preservada.

14 de outubro de 2026. Escopo: os nove arquivos presentes em `compass-documento-principal/`, o estado visível da pasta e a preparação da primeira implementação. As referências a seções abaixo apontam para o [documento principal](../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md).

## 1. Resultado esperado

O Compass será uma infraestrutura local de contexto para agentes: localizar evidências no código, entregar conteúdo com origem e revisão e manter suas representações atualizadas com custo controlado. A primeira entrega será menor e verificável: uma CLI Rust que observa um acervo, registra o que conseguiu ler e compara dois estados.

A mudança é **estrutural**, porque ainda não existe aplicação nesta pasta. A direção de produto está suficientemente definida para começar o contrato Foundation. A especificação computacional dos estados, o ambiente de compilação e o versionamento operacional ainda precisam ser estabelecidos antes da implementação.

## 2. Evidências e estado atual

### Inventário completo

| Arquivo original | Inspeção e conclusão |
|---|---|
| `COMPASS_DOCUMENTO_PRINCIPAL.md` | 132.765 bytes, 1.436 linhas e 18.734 palavras pela contagem por espaços. Lidos integralmente a introdução, os 20 capítulos e os sete apêndices. Contém visão, decisões, limites científicos e primeiro marco; não é schema implementável por si só. |
| `COMPASS_DOCUMENTO_PRINCIPAL.pdf` | 276.276 bytes, 49 páginas A4 e 28 entradas de primeiro nível no outline. Versão de leitura da mesma edição. Todas as páginas renderizadas e examinadas em mosaicos; páginas 15, 20 e 32 ampliadas para tabelas, fórmulas e escopo Foundation. |
| `LEIA-ME.md` | Define corretamente Markdown como fonte canônica e separa o pacote documental do software. Explicita dependências para regeneração e orienta começar pelo contrato dos estados. |
| `MANIFESTO_EDICAO.json` | Oito entregáveis com tamanho e SHA-256, todos coincidentes com os arquivos atuais. Os quatro materiais históricos de entrada são referências do manifesto e não estão distribuídos nesta pasta. |
| `VERIFICACAO_DA_EDICAO.json` | Mistura resultados calculados pelo script com metadados adicionais da edição final, inclusive inspeção de PDF. É um registro histórico válido para os bytes conferidos; nem todos os campos são reproduzidos pelo script incluído. |
| `verificar_edicao.py` | Sintaxe Python válida. Calcula notas, uma solução matricial e estatísticas pequenas; confere presença de capítulos e rótulos. Requer SymPy. As tabelas numéricas estão codificadas no script, sem extração automática do documento. Sobrescreve o JSON da edição. |
| `gerar_documento.py` | Sintaxe Python válida. Verifica Pandoc/XeLaTeX, invoca Pandoc com argumentos estruturados e grava log. Não faz chamadas de rede no código examinado. Gera diretamente no caminho final e não atualiza o manifesto. |
| `estilo.tex` | Cabeçalhos, capa, fontes, espaçamento, tabelas e quebras de página. Apoio editorial; não é documento LaTeX independente nem parte do aplicativo. |
| `layout.lua` | Filtro Pandoc para larguras de tabelas, blocos de código e apêndices. As regras dependem do número de colunas e de títulos específicos desta edição. Adequado a este documento; mudanças na estrutura exigem nova revisão visual. |

Não há `Cargo.toml`, fontes Rust, testes do aplicativo, migrations, banco, configuração de CI ou dependências do produto. Não foi encontrado `AGENTS.md` aplicável nos diretórios ancestrais examinados. Os diretórios reservados `.git`, `.agents`, `.codex` e `.aws` visíveis estavam vazios; sua presença não significa que esses serviços estejam configurados.

### Conferências realizadas

O [registro estruturado](evidence/ANALISE_2026-10-14.json) contém os resultados. Os oito pares tamanho/hash do manifesto coincidem. Isso confirma integridade em relação ao manifesto fornecido, não autoria, backup do legado ou correção de um aplicativo.

As notas ponderadas são Rust 91,6; C++ 81,8; C# 85,0; Java 77,6. A solução ponderada foi recalculada por eliminação racional, em vez da inversão via SymPy do script fornecido: `(22/35, 11/35, 2/35)`. O exemplo de armazenamento resulta em 153.600.000 bytes. São contas didáticas, não benchmarks do motor.

A tabela histórica implica coberturas de `2/9`, `4/9` e `17/36`. Uma distribuição exata de bootstrap pareado, com 12 sorteios das 12 tarefas, reproduz os intervalos percentis de 95% publicados: C1−B de −13,89 a +63,89 pontos percentuais; C1−C0 de 0 a +8,33. Esse recálculo usa a tabela publicada e suas frações subjacentes; não verifica os recibos, a rubrica ou as sessões originais.

No PDF, não apareceram caracteres de substituição nem caixas de palavras fora dos limites da página. Os 189 parágrafos longos de prosa selecionados automaticamente, sem fórmulas, tabelas ou código, correspondem ao texto extraído após normalização de formatação, cabeçalhos e rodapés. As cinco divergências iniciais eram interrupções por paginação e desapareceram ao remover esses elementos. Esse controle não equivale a comparar automaticamente cada fórmula e célula; elas foram examinadas no contexto da fonte e em páginas representativas. A visão geral não mostrou cortes ou sobreposições evidentes.

O verificador fornecido não foi executado: SymPy está ausente e sua execução sobrescreveria o relatório original. Foram feitas conferências independentes. O PDF também não foi regenerado; a versão recebida permaneceu intacta.

### Leitura crítica por área

| Área do documento | Avaliação e consequência |
|---|---|
| Identidade e capítulos 1–3 | A separação do legado está explícita. Não há motivo para reconstruir o software antigo antes de começar o novo. A preservação integral do legado só bloqueia alterações destrutivas naquele acervo, que não integra esta tarefa. |
| Capítulo 4 e apêndice A | Os resultados históricos têm limites e correções identificados. Não sustentam prometer superioridade do produto nem escolher difusão como padrão. |
| Capítulos 5–6 | O produto e suas responsabilidades estão claros. Evidence, Maintenance e proveniência devem começar como módulos de uma biblioteca, não como serviços independentes. |
| Capítulo 7 | É a lacuna imediata: caminho, revisão, elegibilidade, ausência, concorrência e determinismo ainda são perguntas. A proposta Foundation trata essas perguntas diretamente. |
| Capítulo 8 | Rust é uma decisão vigente. A nota arquitetural não precisa ser transformada em uma nova disputa de linguagens. SQLite, Tree-sitter e demais bibliotecas continuam candidatos para problemas posteriores. |
| Capítulos 9–10 e apêndice B | A distinção entre calor livre, reinjeção e propagação discreta é essencial. As fórmulas examinadas mantêm seus pressupostos; não há justificativa para escolher um operador pela metáfora do calor. Pesos de aresta e capacidades de nó têm efeitos diferentes. |
| Capítulos 11–12 | As cinco hipóteses têm perguntas distintas. Conformidade pode ser testada antes de eficácia comercial. Os esperados dos testes devem preceder o algoritmo e incluir falhas que contradigam resultados favoráveis. |
| Capítulo 13 | Recursos são requisito, mas faltam cargas e metas numéricas. Medir leitura, memória, disco e duração desde Foundation; estabelecer limites de release com uma carga identificada. |
| Capítulo 14 | Leitura local e limites de raiz afetam o observador desde o início. Não é necessário antecipar autenticação multiusuário para implementar a CLI. |
| Capítulos 15–16 | Uma autoridade por assunto evita novos planos concorrentes. O estado operacional curto aponta para a fonte e o contrato; este relatório é o diagnóstico inicial, não outra visão de produto. |
| Capítulos 17–20 | O primeiro trabalho está explicitamente delimitado: contrato dos estados e depois núcleo/CLI. A sequência posterior não exige encerrar pesquisa térmica antes de oferecer MCP sobre recuperação útil. |
| Apêndices C–G | Cobertura, glossário, fontes e folha de revisão ajudam a rastrear decisões. Os quatro documentos históricos e recibos primários ausentes limitam auditorias retrospectivas, mas não impedem definir estados novos. |

### Problemas concretos encontrados

1. **O relatório editorial não é reproduzido integralmente pelo verificador.** A execução substitui o JSON e elimina sete campos: `pdf_pages`, `pdf_outline_entries`, `coverage_matrix_topics`, `decision_records`, `rendered_page_coordinate_outliers`, `pdf_replacement_characters` e `pdf_rendering_inspected`. A nova saída também deixa de coincidir com o manifesto da edição. Em uma revisão editorial futura, separar cálculo reproduzível de inspeções manuais e produzir resultados em outro destino.
2. **Os cálculos do verificador podem divergir da fonte sem falhar.** Pesos, notas e coberturas são constantes no Python. Alterar uma tabela no Markdown não necessariamente altera o resultado do script. Isso limita o alcance de `all_source_labels_defined` e dos checks numéricos: presença e aritmética não verificam toda a narrativa. Para uma nova edição, comparar dados extraídos ou manter uma fonte estruturada única para as tabelas.
3. **A regeneração não é transacional como entrega editorial.** O gerador aponta diretamente para o PDF final e não produz novo manifesto. Uma futura alteração deve gerar em destino temporário, verificar o resultado e só então publicar a nova edição. Isso não invalida o PDF atual, cujos bytes coincidem com o manifesto.
4. **Ambiente de produto ainda não preparado.** Git não reconhece a pasta; Rust/Cargo não estão disponíveis nas localizações examinadas. A configuração deve ser resolvida no início da implementação, com verificação de versões e um build mínimo.

Esses achados não exigem reformar o pacote original para iniciar Foundation. A edição recebida permanece como referência imutável desta análise.

## 3. Escopo

Incluído no primeiro marco: identidade do acervo, política de observação, arquivos e revisões, manifestos persistidos, cobertura e erros, comparação determinística, CLI, testes de contrato e demonstração local.

D0, DG, busca, parsing, embeddings, grafos, difusão, MCP, GUI e servidor multiusuário pertencem às entregas seguintes. Uma interface gráfica agora não resolveria a principal incerteza: o significado dos dados que ela mostraria.

A proposta usa Linux como primeiro alvo de validação, acompanhando o ambiente disponível. A visão multiplataforma permanece; Windows/macOS exigirão adaptação explícita de caminhos, links, permissões e publicação de arquivos. Essa ordem é proposta de implementação, não decisão previamente homologada no documento principal.

## 4. Decisões e contratos

O [contrato proposto de estados](../research/contracts/FOUNDATION_ESTADOS_V0.1.md) concentra regras, estados, CLI e exemplos. As escolhas centrais são:

- Conteúdo identificado pelos bytes, sem normalizar codificação ou fins de linha.
- Caminhos e conteúdos têm identidades distintas; renomeação inicial é remoção e adição.
- Observação de árvore viva é uma janela de leitura, sem alegação de snapshot atômico.
- Falta de permissão, instabilidade e limite de recursos produzem desconhecimento explícito.
- Comparação só afirma remoção quando há evidência de cobertura do local correspondente.
- Manifestos JSON persistidos antes de escolher banco de índices. SQLite pode ser avaliado quando houver consultas e transações que o justifiquem.
- Identidade da observação separada do resumo determinístico do estado.
- A primeira etapa guarda hashes e metadados; não guarda blobs de conteúdo. D0 textual futuro precisará de uma nova política de retenção ou de fontes reproduzíveis para obter os bytes dos dois extremos.

A implementação deve proteger o acesso à raiz também entre a checagem e a abertura de um arquivo. Consultar metadados sem seguir links, por si só, não resolve substituições concorrentes; a documentação de Rust descreve essa condição de corrida. O adaptador de sistema de arquivos deverá tratá-la e ter um teste correspondente. [Rust std::fs](https://doc.rust-lang.org/std/fs/index.html)

## 5. Plano executável

Os caminhos de código abaixo são propostos; não existem ainda. A raiz de implementação deve ser independente do legado. Esta pasta pode abrigá-la, desde que o Git seja configurado em um local gravável; a proteção atual de `.git` não deve ser contornada. Se necessário, uma subpasta nova será a raiz Git, com referências explícitas ao pacote documental.

### F0 Contrato e exemplos de referência

- **Resultado:** regras do contrato revisadas e fixtures com respostas esperadas estabelecidas antes do comparador.
- **Áreas:** `research/contracts/`, `verification/fixtures/`, `verification/reference/` e `COMPASS_STATE.md`.
- **Dependência:** revisão das escolhas propostas, em especial alvo Linux, elegibilidade e retenção sem blobs.
- **Aceite:** todos os casos F01–F20 do contrato têm entrada e saída esperadas; nenhum resultado de implementação é usado para inventar o esperado. Congelar schema de transporte e exemplos de serialização como parte do contrato antes de codificar persistência.
- **Verificação:** leitura cruzada das tabelas de estado e dos casos; cálculos por biblioteca padrão Python e exemplos manuais. Esta etapa não precisa de SymPy.
- **Restrição:** D0 permanece sem fórmula nova; fontes históricas não viram testes de conformidade automaticamente.

### F1 Base Rust e identidade do acervo

- **Resultado:** workspace compilável com `compass-core` e `compass-cli`; comando de inicialização cria uma identidade local uma única vez.
- **Áreas:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/compass-core/`, `crates/compass-cli/`, `README.md` e instruções mínimas do novo repositório.
- **Dependência:** F0; toolchain instalada/configurada e raiz Git funcional.
- **Aceite:** segunda inicialização preserva a identidade; falha de permissão retorna erro; diretório de estado não entra no universo observado. Versão da toolchain e dependências ficam fixadas.
- **Verificação:** build da CLI, `--help`, `--version` e integração de `init` em diretório temporário.
- **Restrição:** núcleo não depende de transporte, LLM ou servidor; nenhum diretório do legado é alterado.

### F2 Observação e publicação de snapshots

- **Resultado:** `compass snapshot` produz um manifesto validado com conteúdos identificados, cobertura, exclusões e falhas.
- **Áreas:** módulos `identity`, `policy`, `observe`, `snapshot` e `storage` em `compass-core`; comando correspondente na CLI; testes de filesystem.
- **Dependência:** F1 e schema congelado em F0.
- **Aceite:** vazio e binário são distinguíveis; link não é percorrido; nome não UTF-8 é preservado; leitura instável não recebe hash de revisão aceita; cancelamento não publica JSON truncado; escritores concorrentes não sobrescrevem snapshots.
- **Verificação:** fixtures F01–F12, F15–F18 e F20; falhas de leitura e mudanças concorrentes por injeção controlada; um caso real de permissões sob usuário sem privilégios.
- **Restrição:** nenhuma execução do código observado; nenhuma cópia silenciosa de conteúdo; nenhum envio remoto.

### F3 Comparação e relatório público

- **Resultado:** dois manifestos produzem diferenças reproduzíveis, sem consultar novamente o acervo.
- **Áreas:** `compare`, tipos do relatório e comando `compare`; testes de contrato e CLI.
- **Dependência:** F2.
- **Aceite:** igualdade, adição, remoção, alteração, mudança de executabilidade e desconhecimento conforme contrato; divergência de política rejeitada; retorno ao baseline reconhecido; duas execuções com as mesmas entradas geram JSON idêntico.
- **Verificação:** casos F01–F20 relevantes à comparação; fixture manual `a.txt=abc`, `b.txt=vazio` contra `a.txt=abd`, `c.txt=vazio`; mutações deliberadas que escondem erro ou retornam igualdade constante devem falhar.
- **Restrição:** nenhuma heurística de rename, escore D0 ou inferência semântica inserida nesta fatia.

### F4 Integração, recursos e entrega

- **Resultado:** demonstração reproduzível em árvore temporária e release local identificada.
- **Áreas:** testes de integração, fixtures, README e estado operacional.
- **Dependência:** F3.
- **Aceite:** executar o fluxo da seção 6, registrar duração, arquivos/bytes lidos, pico de memória e disco produzido, junto de máquina, carga e build. Limitações de plataforma e observação aparecem no relatório público.
- **Verificação:** executar os testes do workspace e a demonstração pela CLI compilada. Depois que Cargo e seus arquivos existirem, usar `cargo test --workspace --locked`; esse comando testa os membros do workspace conforme a [documentação oficial do Cargo](https://doc.rust-lang.org/cargo/commands/cargo-test.html). Formatação e análise estática serão configuradas na mesma toolchain.
- **Restrição:** resultados de Foundation não serão apresentados como eficácia de recuperação ou superioridade com agentes.

Não há testes do produto executáveis hoje. Os comandos de CLI e Cargo desta seção são critérios para a implementação futura, não verificações já aprovadas.

## 6. Verificação de ponta a ponta

1. Criar acervo temporário com arquivo de texto, vazio, binário, diretório e link para fora da raiz.
2. Inicializar identidade, observar S0, reiniciar a CLI e carregar o manifesto persistido.
3. Alterar texto, criar um arquivo vazio, remover outro e mudar apenas a permissão de execução de um terceiro.
4. Observar S1 e conferir a lista exata de diferenças por uma fixture manual.
5. Restaurar os bytes, caminhos e executabilidade de S0; observar S2 e verificar igualdade de estado entre S0 e S2, com identidades de observação distintas.
6. Repetir com acesso negado e com alteração durante leitura. Exigir parcialidade/erro e impedir uma remoção ou igualdade falsas.
7. Trocar um componente de caminho por link durante abertura; verificar que nenhum alvo externo foi lido.
8. Interromper publicação e iniciar dois escritores; nenhum manifesto final pode ficar parcial ou sobrescrever outro.
9. Comparar os mesmos manifestos duas vezes; exigir bytes idênticos no relatório JSON e códigos de saída previstos.

O teste de concorrência deve usar sincronização controlada, não depender de uma corrida aleatória passar por acaso.

## 7. Riscos e conclusão

| Risco | Tratamento |
|---|---|
| Chamar uma varredura de snapshot atômico | Publicar `observation_window`, cobertura e limitações; manter a opção de captura imutável para contrato futuro. |
| Confundir falha com exclusão ou igualdade | Modelo explícito de desconhecimento e cobertura por diretório; teste negativo obrigatório. |
| Testes confirmarem a própria implementação | Fixtures estabelecidas antes do código e método de referência separado. |
| Guardar só hashes e depois exigir diff textual histórico | Declarar agora a limitação; definir retenção de bytes antes do sensor D0. |
| Selecionar bibliotecas antes de conhecer as operações | Escolher dependências por fatia; banco, parser e solver não entram em Foundation. |
| Ampliar a pesquisa antes da primeira entrega | Encerrar Foundation com CLI demonstrável e limitações claras. |
| Perder rastreabilidade sem Git/toolchain | Resolver a raiz operacional antes do código e registrar build/versões reais. |

Foundation estará pronta quando uma pessoa puder observar, persistir e comparar estados pela CLI, verificar as diferenças e entender o que não pôde ser observado. A análise desta pasta e a proposta inicial do contrato estão concluídas; o marco de software ainda não foi implementado.
