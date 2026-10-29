# Acervos, recuperação e difusão local

O Compass prepara revisões independentes da árvore viva. Índices, símbolos, vetores e evidências pertencem a uma geração identificada. CLI e MCP usam o mesmo núcleo Rust; nenhum código do acervo é executado.

## Preparar e atualizar

```bash
compass collection prepare --root /caminho/repositorio --store /caminho/acervo --json
compass collection status --store /caminho/acervo --json
compass collection update --store /caminho/acervo --json
```

O destino é explícito. O teto padrão é 4 GiB, ajustável na criação com `--max-disk-bytes`. O teto contabiliza comprimentos lógicos dos arquivos, inclusive staging, deduplicando hardlinks; não promete equivalência com blocos e metadados ocupados no filesystem. Revisões e objetos publicados não são apagados automaticamente. Staging abandonado é reconciliado; uma publicação concorrente mais nova não pode ser sobrescrita por um escritor atrasado.

A captura usa o mesmo fluxo seguro que calcula o hash. A política inicial das coleções limita arquivos a 4 MiB, leitura total a 256 MiB e 10 mil entradas, além das exclusões de Foundation. Arquivos não capturados e cobertura parcial permanecem explícitos. A observação não é uma transação atômica de toda a árvore nem uma promessa de atualização contínua.

`update` reutiliza objetos, entradas lexicais e vetores válidos. O resolvedor estrutural recompõe seu escopo de acervo para evitar relações antigas após mudanças interarquivos. Gerações anteriores continuam consultáveis com `--generation UUID`.

## Habilitar semântica local

```bash
compass model install --model-dir /caminho/modelo-e5 --json
compass collection prepare --root /caminho/repositorio --store /caminho/acervo-semantico \
  --semantic-model /caminho/modelo-e5 --json
```

A instalação baixa explicitamente os arquivos públicos fixados de `intfloat/multilingual-e5-small` (aproximadamente 488 MB no conjunto) e confere seus SHA256. Inferência Candle CPU em float32; sem serviço externo e sem download durante consulta. Mantenha o diretório do modelo acessível: o acervo registra seu caminho absoluto. Alteração ou ausência dos arquivos produz erro explícito.

Vetores: 384 dimensões, prefixos E5, média com máscara e normalização L2. As unidades respeitam limites sintáticos quando disponíveis e são divididas para caber nos 512 tokens; os bytes originais são preservados. Consultas acima desse limite são recusadas. A busca usa cosseno exato; não há ANN ou GPU. Relevância semântica continua sendo uma hipótese de ranking a ser avaliada, não prova de correção do código.

## Consultar e navegar

```bash
compass files --store /caminho/acervo --query src --json
compass query --store /caminho/acervo --query 'total da compra' --mode hybrid --json
compass neighbors --store /caminho/acervo --query total_amount --json
compass neighbors --store /caminho/acervo --node ID --depth 2 --json
compass retrieve --store /caminho/acervo --query checkoutSignal --mode literal \
  --expansion heat --theta 1 --budget-bytes 16384 --json
```

Modos: `literal`, `lexical`, `semantic`, `hybrid`. A fusão usa RRF60 e informa quais canais participaram. A busca literal antiga mantém sua interface e seus limites. Fontes legadas sem índices continuam disponíveis por `search` e `read`.

Estrutura: Python, Rust, JavaScript, TypeScript, JSX e TSX. Imports locais e referências são resolvidos conservadoramente por caminhos e escopos. Macros, despacho dinâmico, aliases/configurações complexas, dependências externas e ambiguidades podem ficar parciais ou indisponíveis. “Resolvido” significa vínculo sintático identificado no acervo, não execução observada nem análise completa de tipos. Cada relação informa evidência e produtor.

Expansão: `none` por padrão, `bfs` ou `heat`. Calor livre `exp(-tL)s`, projeção simétrica de pertencimento/imports/chamadas resolvidas, peso unitário, pares deduplicados, `L=D-W`. `theta=t*grau_max`, entre 0 e 10. Série de Poisson esparsa, tolerância de cauda `1e-8`; limites de trabalho e erro de ponto flutuante são informados separadamente. O calor ordena expansão; não é probabilidade de relevância.

O pack verifica integralmente os objetos antes de retornar texto. `budget_bytes` limita o payload JSON canônico inteiro, incluindo metadados; `payload_bytes` informa a contagem. O orçamento é em bytes, não tokens do modelo consumidor. Trechos omitidos conservam referências e a continuação conserva geração e parâmetros. Use o mesmo pedido com `--cursor` para continuar; offsets de consultas exigem `--generation`.

Para declarar evidências obrigatórias e verificar sua presença na resposta, acrescente `--contract arquivo.json` na CLI ou `contract` no MCP. A [aquisição por obrigações](AQUISICAO-OBRIGACOES.md) explica os seletores, os limites e o significado de `SUFFICIENT_EVIDENCE_ACQUIRED`. Consultas sem contrato conservam o comportamento descrito acima.

## Conectar pelo MCP

```json
{
  "sources": {
    "meu-acervo": {"collection": "/caminho/acervo"},
    "legado": {"snapshot": "/caminho/snapshot.json", "source": "/caminho/pacote"}
  },
  "deadline_ms": 30000
}
```

Execute `compass mcp --config /caminho/config.json`. Fontes são autorizadas na configuração; os argumentos das ferramentas não aceitam caminhos arbitrários.

| Ferramenta | Uso |
|---|---|
| `compass_search` | Busca literal, inclusive nas fontes antigas |
| `compass_read` | Leitura verificada; em acervos gerenciados exige `generation` |
| `compass_files` | Descoberta de caminhos observados |
| `compass_query` | Busca indexada e combinação de canais |
| `compass_neighbors` | Descoberta de símbolos e relações |
| `compass_retrieve` | Evidence pack com orçamento e expansão opcional |

Ferramentas indexadas são anunciadas quando existe uma fonte `collection`. Preparação, atualização e instalação são operações CLI. Toda consulta é somente leitura. O cliente continua responsável por interpretar a intenção, escolher consultas e administrar seu histórico de mensagens.

MCP entrega `structuredContent` e texto JSON equivalentes: o consumidor deve incorporar uma representação, para não duplicar conteúdo no contexto. `_meta.compass` mede payload e resultado da ferramenta, separadamente do orçamento do pack. Cancelamento é cooperativo; timeout retorna erro, cancelamento solicitado pelo cliente segue o protocolo existente.

## Verificação e limites desta entrega

Controles reproduzíveis ficam em `verification/recovery/`, `verification/semantic/` e nos testes Rust. O laboratório cobre acervos de desenvolvimento e avaliação separados, bytes/linhas/revisões, uso real CLI/MCP, atualização e árvore removida, embeddings contra PyTorch e calor contra solução independente. Os arquivos de resultados identificam os binários efetivamente testados.

Os acervos são pequenos e sintéticos. Medidas de qualidade e recursos desses controles não são estimativa de ganho em qualquer repositório. Watcher, DG, priorização de manutenção por D0/DG, GUI, servidor remoto multiusuário e outras plataformas ficam para evolução posterior.

O campo de configuração MCP `limits` conserva o perfil das operações legadas. `max_output_bytes` e `deadline_ms` também limitam as ferramentas indexadas. Reduções dos demais limites legados fazem as ferramentas indexadas recusarem a chamada explicitamente; não são ignoradas nem reinterpretadas como orçamento da inferência. Use configuração separada para esse perfil restrito. Nesta entrega, derivados têm teto de 128 MiB serializados por geração; a projeção de calor aceita até 100 mil nós/1 milhão de arestas e até20 milhões de visitas. Esses são limites de trabalho/dados, não garantia de RSS; o modelo CPU requer memória adicional, medida nos controles.
