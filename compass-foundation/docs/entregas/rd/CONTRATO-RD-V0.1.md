# Recuperação direta RD v0.1 — busca literal verificável

Versão 1, 19/10/2026. Contrato da [OS-RD-I1](OS-RD-I1.md). Origem: documento principal §§6.3, 7, 13 e 17.3; Foundation e D0/R1 aceitas. Esta primeira fatia procura texto literal e entrega linhas verificadas da revisão declarada. Não mede relevância semântica, suficiência do contexto ou benefício para agentes.

## 1. Fronteira e preservação

Produção Rust, Linux com openat2; CLI sobre o mesmo núcleo. A entrada é **um snapshot Foundation e um pacote externo explicitamente fornecido**, no formato já ativo em D0: `source.json` e `objects/<sha256>`. Não há varredura da árvore viva, fallback, captura automática, índice persistente, retenção nova, embeddings, parsing, ranking, grafo, D0 como gate ou manutenção incremental. O resultado refere-se somente ao snapshot informado, nunca implicitamente à árvore atual.

Reutilizar validação Foundation, JSON estrito/JCS, binding bruto e leitor seguro D0. A ampliação de visibilidade interna de input/resources/source é permitida; a leitura/stability, fórmulas, defaults e contratos D0 permanecem iguais. Não duplicar o leitor por caminhos, enfraquecer openat2, canonicalizar antecipadamente referências com `..` ou adicionar cache/retry. O caso de ancestral substituído continua exigindo detecção durante a janela de leitura. Não se promete fotografia atômica global nem validade eterna depois do EOF verificado.

## 2. Interface

Interface **nova a implementar**:

```sh
compass search --snapshot <manifesto.json> --source <pacote> --query <texto> --max-matches 20 --json
```

`snapshot`, `source` e `query` obrigatórios. `--max-matches`: inteiro decimal 1..1000, default 20. `--limits <arquivo.json>` opcional reduz os limites da seção 5; `--deadline-ms` opcional 1..3600000. Sem inferir pacote a partir da raiz registrada no snapshot. A biblioteca oferece Request/Limits, resultado estruturado/bytes/exit e função com cancelamento; nomes internos ficam a cargo do implementador. CLI apenas adapta argumentos, sinal e apresentação.

Query: UTF-8, 1..256 **bytes**, sem NUL/CR/LF. Outros caracteres, inclusive tab, espaços, pontuação e metacaracteres, são literais; não aparar espaços, executar shell, interpretar regex, case-fold ou normalizar Unicode/BOM. Valores além de u64, negativos, floats e argumentos inválidos são invalid_arguments/exit 2. Valores de query que coincidam com nomes de comandos não alteram o roteamento.

## 3. Busca e trechos

Enumerar somente entries `observed`, ordenadas pelos bytes nativos decodificados de path_key. Não ler excluded ou inventar descendentes de diretórios desconhecidos. Usar os hashes/tamanhos dessas entradas para ler `objects/<hash>`, nunca usar path_key como caminho de abertura do conteúdo.

Para cada arquivo: admitir limites/memória/reserva, ler até tamanho declarado+1 por chunks de no máximo 65536 bytes, provar EOF/hash/tamanho e estabilidade por handles/componentes. Bytes não verificados **não entram na busca ou nos trechos**. Arquivos distintos com mesmo hash são unidades distintas, com leitura/reserva própria nesta versão; sem cache entre caminhos.

Depois da verificação, UTF-8 inválido é non_text/invalid_utf8; UTF-8 válido com NUL é non_text/nul. Essa ordem decide o caso com ambos. Os demais são textuais. Dividir somente em LF: linha é o intervalo até e incluindo LF, ou o sufixo final não vazio sem LF. Vazio tem zero linhas; LF terminal não cria outra linha. CR, BOM e Unicode originais são preservados. Preflight de **todas** as linhas do arquivo exige comprimento ≤max_line_bytes, incluindo LF/CR/BOM; se falhar, todo o arquivo fica unavailable/line_bytes_limit e nenhum hit dele é retido.

Uma linha contém o query como subsequência contígua de bytes UTF-8, sensível a caixa. **Um hit por linha**, mesmo com repetições/sobreposições na linha. Retornar linha inteira: `text` contém exatamente os bytes UTF-8 daquele intervalo, inclusive terminador quando existente. `line` é 1-based; `start_byte` é 0-based inclusivo e `end_byte` exclusivo no objeto **bruto**. Exigir `text.as_bytes()==object[start_byte:end_byte]`, com identidade de conteúdo/tamanho e caminho no hit. Não confundir offsets de bytes com caracteres.

Ordenar hits por caminho nativo e linha crescente. Reter os primeiros max_matches, **continuar a busca em todos os arquivos admitidos** para contar matching_lines_known e detectar falhas tardias. `matches_truncated` significa que a contagem conhecida excede o limite de seleção; não é falha de leitura. Sem cursor/paginação nesta fatia. Não anunciar ranking de relevância: a ordem é determinística.

## 4. Disponibilidade e transporte

Pacote/descritor presente deve ser validado mesmo no universo vazio. Shape estrito com os mesmos seis campos D0 e binding a SHA-256 dos bytes exatos do manifesto, workspace_id, snapshot_id e policy_id. source_ref é SHA-256 dos bytes brutos do descritor. Snapshot com produtor histórico continua aceito; não exigir que seu producer_build seja o executável atual. Fonte/descritor inválido gera envelope, sem relatório de busca.

Cada arquivo observado tem um check independente e estado:

- `searched`: bytes verificados e texto/linhas admitidos; inclusive vazio e ausência de hits.
- `non_text`: bytes verificados, com motivo invalid_utf8 ou nul; inelegibilidade conhecida.
- `unavailable`: não foi possível resolver/verificar/admitir o arquivo. Motivos observados, únicos, ordenados; nenhum hit do arquivo.

Precedência por arquivo: tamanho declarado → memória W → saldo/reserva → abertura/leitura/EOF/estabilidade → UTF-8/NUL → linhas → busca. Motivos de abertura/objeto reutilizam D0: object_missing, source_invalid_kind, source_unreadable, hash_mismatch, size_mismatch, source_unstable. Admissões recusadas usam file_bytes_limit, memory_budget ou total_bytes_limit, sem fingir leitura. Line_bytes_limit conserva o check verified. Não relatar falhas hipotéticas de etapas não executadas.

`scan.status` refere-se à **cobertura da varredura**:

- `complete`/exit 0: snapshot quality=complete e nenhum arquivo unavailable; universo vazio/excluded/non_text também pode ser complete.
- `partial`/exit 3: snapshot parcial ou há unavailable, e ao menos um arquivo foi resolvido como searched/non_text.
- `unavailable`/exit 1: há pendência de snapshot/arquivo e nenhum arquivo foi resolvido.

Motivos globais: snapshot_partial se quality=partial e unresolved_files se há unavailable; únicos em ordem lexical. Snapshot parcial pode entregar linhas de entries observed verificadas, mas jamais vira cobertura completa por isso. matching_lines_known conta somente searched; não é total global exato sob parcialidade. Nenhum hit com cobertura parcial/indisponível significa ausência apenas no subconjunto conhecido. Complete com matches_truncated=true tem cobertura completa, seleção limitada explicitamente; não implica entrega de todos os hits. Não emitir alegação de ausência no acervo vivo.

Com `--json`, stdout é um único objeto JCS UTF-8 **sem newline**, sem progresso/logs. Sem a flag: mesmo valor em JSON legível com newline. Todos os campos são fechados; inteiros de domínio são strings decimais u64 canônicas, sem zeros iniciais. Bytes de stdout preparados antes de publicação; SIGINT/deadline não publicam hits parciais. Falha de I/O de pipe pode deixar prefixo, sem promessa de transação de stdout.

Envelope exato: `{"error":{"code":"<code>","message":"<code>"}}`. invalid_arguments e incompatible_inputs: exit 2; cancelled: 130; input_invalid/source_invalid/input_limit/output_limit/deadline_exceeded/arithmetic_overflow/operational_error: 1. Binding bem formado divergente é incompatible_inputs; shape/domínio inválido é source_invalid. Parse/semântica do snapshot inválida é input_invalid. Erro ao ler/validar o perfil de limits é invalid_arguments. Checkpoint final prioriza cancelamento, depois deadline, depois erro pendente, como D0.

### Valor do relatório — campos fechados

| Campo | Conteúdo |
|---|---|
| schema_version / retrieval_contract_version / report_type | `1` / `0.1.0` / `literal_search` |
| producer_version / producer_build | Versão do executável e build atual, framing Foundation existente |
| scope | `declared_snapshot_only` |
| snapshot | snapshot_id, workspace_id, path_namespace, policy_id, quality, state_digest (nullable), snapshot_file_sha256 bruto |
| source | Seis campos do descritor aceito + source_ref |
| query | text, mode=`literal_utf8_lines`, max_matches string |
| limits | Nove limites efetivos abaixo + max_duration_ms nullable/string |
| scan | status, reasons; strings observed_files, verified_text_files, non_text_files, unavailable_files, excluded_entries, matching_lines_known, matches_returned; bool matches_truncated |
| files | Uma linha por observed, ordem nativa; estrutura descrita abaixo |
| matches | Hits selecionados, ordem declarada; estrutura abaixo |
| resources | Strings object_bytes_read, object_bytes_reserved, logical_memory_peak; sem RSS/células D0 |

`files[]`: path_key, expected_revision `{content_sha256,size_bytes}`, state, reasons e check. `check` contém state (`verified/failed/not_attempted`), source_ref, actual_revision nullable, hash_check/size_check nullable/bool. Actual_revision tem os mesmos dois campos de expected_revision e só existe após EOF real. Recusa de admissão é not_attempted; leitura/abertura falha é failed; texto inelegível/linha longa pode conservar verified. Checagens booleanas preservam fatos EOF mesmo com source_unstable.

`matches[]`: path_key, content_sha256, size_bytes, line, start_byte, end_byte, text. Revisão é a do respectivo arquivo searched; não criar ID/hash adicional de trecho. `observed_files=verified_text_files+non_text_files+unavailable_files`; verified_text_files conta somente searched. matches_returned=len(matches) e matches_truncated=(matching_lines_known>matches_returned). excluded_entries conta entries excluded, sem estimar seus descendentes. Campos não mencionados são proibidos.

## 5. Recursos e consistência da execução

Limites iniciais de engenharia, reduzíveis por perfil completo com **exatamente nove chaves**, strings u64, 1..default. Não são SLO de produto nem extrapolação dos benchmarks anteriores.

| Chave | Default/teto |
|---|---:|
| max_manifest_bytes | 16777216 |
| max_source_descriptor_bytes | 65536 |
| max_records | 10000 |
| max_json_depth | 64 |
| max_file_bytes | 4194304 |
| max_total_object_bytes | 67108864 |
| max_output_bytes | 1048576 |
| max_logical_memory_bytes | 134217728 |
| max_line_bytes | 4096 |

max_records conta N=entries+directory_coverage+issues de **um** snapshot. Perfil tem cap de 65536 bytes/profundidade 64, JSON estrito. Input acima de bytes/depth/N é input_limit; source descriptor acima do cap também é input_limit. Soma/produto/contadores usam aritmética checada, sem float/saturação silenciosa.

Reutilizar o mecanismo de créditos Context, com **fórmula específica desta busca**: antes de construir Snapshot/relatório, admitir `S_manifest=8*b+4096*N+65536+8*max_output_bytes`. O último termo cobre retenção/preparação da resposta limitada; uma busca pode manter hits enquanto lê próximos arquivos. Depois de validar descritor, acrescentar `8*d`; S final usa bytes brutos b/d, permanece debitado até o fim. Transporte delimitado pode ser parseado para contar registros; recusar antes do domínio se o crédito exceder teto.

Por arquivo de tamanho declarado s, admitir `W=2*(s+1)+65536` sobre S antes de abrir/reservar objeto; liberar W ao acabar, sem liberar créditos estruturais/resposta. `logical_memory_peak=max(S_manifest,S,S+W admitido)`; recusa não aumenta peak. Texto/linhas são percorridos por views/iteradores limitados, sem reter conteúdo de todos os arquivos. Não há índices de linha ou cópia normalizada arbitrários fora desses créditos. Fórmula é admissão lógica, **não RSS/controle do alocador ou garantia contra OOM**.

Reserva de objeto é s+1, não devolvida; read é bytes físicos, até prefixo/erro. Recusa por saldo não debita e arquivos menores posteriores podem prosseguir. Vazio precisa reservar 1 para provar EOF. Relatório canônico >max_output_bytes gera output_limit sem relatório; não descartar diagnósticos/hits silenciosamente para caber. Esse cap aplica-se ao relatório, não ao envelope fixo de erro, inclusive com perfil de 1 byte. O default de max_matches limita retenção, não leitura da fonte.

SIGINT/deadline checados entre entradas/chunks e a cada no máximo 65536 bytes percorridos pelos **novos laços RD de validação textual, linhas e busca**, incluindo arquivo sem LF/sem hits. Preservar os checkpoints do leitor D0. Hashing, parse/validação Foundation e JCS compartilhados têm entradas limitadas pelos caps, mas **não têm preempção interna**; checar cancelamento/prazo antes e depois dessas chamadas. O quantificador de 65536 bytes não exige alterar esses primitivos protegidos nem garante latência máxima de interrupção. Testes com barreiras/relógio injetável, sem sleeps probabilísticos. Deadline começa após argumentos/perfil válidos e inclui entradas, busca e preparação de saída. I/O bloqueante também pode atrasar a interrupção. Sem paralelismo de leitura ou compilação durante as medições.

## 6. Gabaritos e controles mínimos independentes

Fixture deve ser escrita/conferida antes da lógica Rust. Calcular offsets/hashes pelo conteúdo literal abaixo, não copiando a produção. A matriz vincula cada RD01–RD24 a testes realmente executáveis e distingue CLI de prova interna.

| ID | Cenário e esperado obrigatório |
|---|---|
| RD01 | `a.txt="alvo\nNada\nalvo alvo\n"`, query alvo: hits linha 1 [0,5) e linha 3 [10,20); 3 ocorrências textuais dão **2 linhas**. |
| RD02 | `aaa\n`, query aa: um hit linha 1 [0,4); ALVO não encontra alvo; regex/pontuação e espaços são literais. |
| RD03 | Query ausente em texto verificado: complete/0, zero hits; scoped ao snapshot. |
| RD04 | Vazio e `\n`: zero/uma linha, nenhum hit com query não vazia, ambos searched e EOF verificado; reserva 1 no vazio. |
| RD05 | `é🙂\r\nneedle\n` (15 bytes): 🙂 retorna linha 1 [0,8); needle linha 2 [8,15). Texto inclui CRLF/LF. CR isolado e U+2028 não separam linhas; o oráculo não pode usar splitlines genérico. |
| RD06 | BOM+`alvo\n` (8 bytes): query alvo retorna [0,8), BOM preservado; é e e+combining não são normalizados. |
| RD07 | Caminhos nativos incluindo `0xff`, `10`, `2`: ordem por bytes, path_key sem perda; nomes nunca viram caminho de objeto. |
| RD08 | UTF-8 inválido e NUL: non_text, razões exatas e checks verified; fora dos hits; precedência invalid_utf8 antes de nul. |
| RD09 | Um arquivo ausente + outro textual válido: partial/3; removendo o válido, unavailable/1. Nenhuma falha vira zero completo. |
| RD10 | Adulteração mesmo tamanho, menor e maior: hash/size/EOF e actual_revision corretos; nenhum trecho adulterado. |
| RD11 | Raiz/objects/objeto symlink, link pendente, FIFO/socket: motivos exatos, sem leitura externa ou bloqueio indefinido; controle regular passa. |
| RD12 | Barreira troca nome/componente/ancestral antes da reconferência, inclusive alias/../package: source_unstable, sem hits desse arquivo; referências estáveis seguem válidas. |
| RD13 | Snapshot parcial com observed verificado + cobertura desconhecida: hit permitido, scan partial; parcial sem resolvido: unavailable. Excluded não é lido. |
| RD14 | Fronteiras de s+1 e saldo não devolvido; recusa de arquivo grande não impede posterior menor; arquivos de mesmo hash debitados por caminho. |
| RD15 | Linha de 4096 bytes admitida; 4097 bloqueia todo o arquivo antes de reter hits. Query de 256 bytes admitida e 257 recusada; fronteiras contam UTF-8 bruto. |
| RD16 | Três linhas matching, max_matches=1: primeiro hit, known=3, truncated=true; falha em arquivo posterior continua aparecendo e reduz cobertura. |
| RD17 | S_manifest/S/W exatamente no teto e abaixo; preflight antes do domínio, W antes de leitura/reserva; créditos/peak literais independentes. |
| RD18 | JSON duplicado, campos extras, profundidade, números/strings u64, negativo/float/overflow e perfil incompleto: envelope/código/exit específicos. |
| RD19 | SIGINT/deadline sincronizados durante leitura/busca e antes de publicar; cancelamento vence deadline/erro, nenhum relatório parcial. |
| RD20 | Fluxo init→snapshot→pacote→search; árvore viva removida, mesmo stdout; sem escrita no acervo/pacote/manifesto. |
| RD21 | Descritor inválido/binding divergente também no universo vazio; snapshot/pacote de revisão diferente não é reconciliado silenciosamente. |
| RD22 | JCS, Unicode, strings u64, newline/escapes, output exato no cap e acima: sem prefixo preparado ou perda silenciosa; query igual a d0/init/search não troca comando. |
| RD23 | S0/S1 mesmo path com conteúdos diferentes: cada busca entrega bytes do seu snapshot/pacote; pacote trocado falha; D0 baixo/zero não autoriza mistura. |
| RD24 | Oráculo/validador: controle válido e mutações de trecho, offsets, revisão, cobertura falsamente complete e truncamento; cada uma deve atingir a asserção específica. |

Medição descritiva: 1.000 arquivos de 4.096 bytes em 100 diretórios, mesma receita ASCII M de R1/S0, query literal `BASE` na linha j=31 **somente** (substituir BASE por NONE nas outras 63 linhas, preservando comprimento). Cada arquivo possui uma linha matching, total 1000; max_matches=20, truncated=true. Um snapshot/pacote, 4.096.000 bytes lidos, 4.097.000 reservados. Três invocações nativas com mesmo stdout, GNU time individual/RSS e wall, ambiente/cache/build/disco/ledger separados, sem SLO ou root/flush. Essa carga é nova de RD; não reutiliza o corpus binário Foundation como se fosse texto.

Fixtures, cobertura, relatórios brutos representativos, binding e resultados ficam na entrega RD. Contratos/fixtures D0 e resultados históricos permanecem preservados. Novos fontes/CLI e inclusão deste contrato no build produzem nova identidade; evidência de regressão do **novo** binário é obrigatória. Aceite final depende da conferência independente posterior da coordenação, não do self-test.
