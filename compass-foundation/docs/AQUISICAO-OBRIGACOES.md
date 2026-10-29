# Aquisição por obrigações

O consumidor pode declarar quais evidências precisa receber no próprio pack. O Compass resolve essas obrigações em uma geração do acervo, verifica os objetos e avalia o conteúdo efetivamente entregue. A interpretação da pergunta e a resposta final permanecem com o agente consumidor.

`completion.status = SUFFICIENT_EVIDENCE_ACQUIRED` significa que todas as obrigações declaradas estão satisfeitas **nesta resposta** e não existem pendências de interpretação. Não significa que o Compass comprovou uma conclusão sobre o comportamento do programa. Por exemplo, entregar a declaração e o corpo completos de uma função permite ao agente analisá-la; o status não afirma que ela realiza uma fusão, valida uma permissão ou está livre de defeitos.

## Pedido

O argumento opcional `contract` de `compass_retrieve` é um objeto JSON. Na CLI, use `--contract /caminho/contrato.json`:

```bash
compass retrieve --store /caminho/acervo --generation UUID \
  --query checkoutSignal --mode literal --budget-bytes 16384 \
  --contract /caminho/contrato.json --json
```

```json
{
  "schema_version": "1",
  "intent": "Obter a implementação completa de checkoutSignal para análise",
  "obligations": [
    {
      "id": "implementation",
      "kind": "symbol_definition",
      "symbol": {"name": "checkoutSignal"}
    }
  ],
  "interpretation_pending": []
}
```

Há de 1 a 16 obrigações, todas obrigatórias. O contrato ocupa no máximo 16 KiB em JSON canônico. `intent` registra a intenção; não é interpretado por um LLM dentro do Compass. `interpretation_pending` registra aspectos ainda não transformados em obrigações e impede o status de suficiência. Tipos desconhecidos permanecem explícitos como não suportados; entradas malformadas são rejeitadas.

Os exemplos [com evidências completas](examples/checkout-contract.json) e [com interpretação pendente](examples/checkout-insufficient-contract.json) podem ser usados na fonte sintética local `compass-demo`. O segundo demonstra insuficiência mesmo quando a definição solicitada está presente.

Seletores de símbolo usam `name` exato, com `path_key` e `scope` opcionais para desambiguar. Outra forma é `{"node_id":"identidade retornada"}`, que exige `generation` explicitamente fornecida no pedido. Um cursor sozinho não substitui esse requisito. Consulte `compass_neighbors` para obter identidades e `compass_files` para obter `path_key`. Chaves representam caminhos capturados; não autorizam leitura de caminhos externos.

## Obrigações suportadas

| Tipo | Campos específicos | Evidência necessária |
|---|---|---|
| `symbol_definition` | `symbol` | Declaração e corpo completos, com wrappers reconhecidos pelo parser |
| `literal_occurrence` | `literal` e um de `path_key` ou `symbol` | Pelo menos uma ocorrência exata e sua linha, dentro do domínio escolhido |
| `resolved_relation` | `source`, `target`, `relation` | Relação direta, direcionada e resolvida estaticamente, com testemunho de origem e identidades dos extremos |

`relation` aceita `calls`, `imports`, `references` e `contains`. Relações com arquivos podem usar suas identidades como extremos. A direção importa. Ambiguidade, parsing que impeça certificar a declaração, vínculo parcial e dependência indisponível não são promovidos a prova. Calor e pontuação de busca não comprovam relações adicionais.

Não há prova geral de ausência, expansão de macros, rastreamento novo de valores ou inferência de comportamento em execução. Uma obrigação não suportada continua pendente mesmo se a busca não encontrar resultados.

## Resposta, orçamento e continuação

Pedidos com contrato retornam pack de schema `2`; os pedidos sem contrato preservam o formato anterior. O resultado inclui `completion.status` e `completion.obligations`, com os estados `satisfied`, `partial`, `unresolved` ou `unsupported`, motivos e referências aos testemunhos contidos em `items`.

As evidências obrigatórias têm prioridade. Quando sua união cabe no orçamento, o objetivo é entregá-las em uma chamada. Se não cabe, o Compass prioriza obrigações completas na ordem declarada e depois trechos parciais. Não acrescenta material apenas para ocupar a sobra. Texto, referências, avaliação das obrigações e demais metadados entram no limite `budget_bytes`; o envelope MCP é contabilizado separadamente. Esse limite mede bytes, não tokens do modelo consumidor.

Neste modo, seletores explícitos resolvem as obrigações diretamente; a consulta livre e o ranking não são necessários para certificar esses vínculos. A resposta identifica essa política em `coverage`. Se for solicitada expansão BFS ou calor, seus diagnósticos são separados da suficiência e a seleção continua restrita às evidências obrigatórias. Para descoberta aberta e contexto complementar, as consultas sem contrato mantêm a recuperação e a expansão anteriores.

`INSUFFICIENT_EVIDENCE` exige examinar os motivos: aumentar o orçamento pode resolver uma omissão, enquanto um nome ambíguo exige desambiguação. Repetir o mesmo pedido não muda sua cobertura. Um orçamento incapaz de comportar os metadados mínimos produz erro explícito.

A continuação conserva geração, contrato, parâmetros e política. Cada página é avaliada separadamente: evidências presentes somente em páginas anteriores não tornam a página atual suficiente. `next_cursor: null` significa ausência de continuação; não é um certificado de suficiência. O consumidor que precisar de todas as evidências juntas deve fazer um pedido cujo orçamento comporte o conjunto.

## Integração com agentes

Envie o mesmo objeto no argumento `contract` da ferramenta `compass_retrieve`, junto de `source`, `generation`, `query` e demais parâmetros. As seis ferramentas existentes permanecem disponíveis. Fontes de manifesto/pacote continuam restritas às capacidades anteriores; a aquisição por obrigações exige acervo preparado.

Ao receber suficiência, o consumidor pode encerrar a aquisição **para aquele contrato** e elaborar sua resposta. Se descobrir que precisa de outro fato, deve formular uma obrigação adicional explícita. A política do loop pertence ao consumidor; este mecanismo não interrompe o agente remotamente nem garante sua economia total de tokens.

Não é necessário reindexar acervos existentes apenas para habilitar essa interface. Reinicie a conexão MCP após atualizar o executável para descobrir o novo argumento da ferramenta.

## O que medir

Os controles independentes ficam em `verification/acquisition/`. A aceitação exige uma resposta suficiente para positivos suportados cuja evidência caiba, zero falsas suficiências nos negativos, testemunhos íntegros dentro do pack, orçamento respeitado e equivalência CLI/MCP. Cobertura na primeira resposta, fragmentação, redundância, bytes de texto/metadados, latência e memória são medidas separadas. Resultado correto de uma resposta final de LLM e superioridade do calor não são condições de conclusão deste mecanismo.
