# RD-I2 — leitura verificável de contexto por linhas

Versão 1, 19/10/2026. Contrato da [OS-RD-I2](OS-RD-I2.md). Origem: documento principal §§6.3/7/17.3 e a busca literal RD aceita após C01. Incremento observável: usar o path_key/linha de um hit para abrir as linhas vizinhas e continuar a leitura, na mesma revisão explicitamente declarada.

O [contrato da busca](CONTRATO-RD-V0.1.md) permanece intacto. Read é operação nova; não muda a saída/default de search ou D0 e não cria índice, banco, retenção, captura automática, cursor opaco, ranking, manutenção ou MCP. Rust/Linux/openat2, pacotes externos explícitos já suportados; sem árvore viva ou fallback.

## 1. Interface e autoridade

Interface **nova**:

```sh
compass read --snapshot <manifesto.json> --source <pacote> --path-key <base64url> --start-line 1 --max-lines 20 --json
```

Snapshot/source/path-key obrigatórios. start-line: u64 ≥1, default 1; max-lines: 1..1000, default 20. Path-key: base64url canônico sem padding, caminho relativo nativo válido e não vazio conforme Foundation; nunca caminho de abertura do conteúdo. Preservar bytes não UTF-8 do nome. Argumentos separados e com `=` são suportados; path-key aceita valor hifenizado. Um valor literal `--json` em path-key continua sendo valor, inclusive se outro argumento falha; por ser key inválida pode receber invalid_arguments, mas não ativa saída JSON implicitamente. Flag real de saída nas posições já suportadas continua selecionando o modo.

`--limits <perfil>` e `--deadline-ms` reutilizam exatamente o loader/defaults/validação da busca RD: nove campos positivos reduzíveis, duração opcional 1..3600000. Não introduzir outro perfil ou policy_id. API pública de leitura com Request, resultado estruturado/bytes/exit e AtomicBool; pode reutilizar Limits/Output/Error já existentes.

Validar argumentos antes de ler fonte. Na operação: manifesto único com transporte/preflight/domínio Foundation; depois descritor/binding bruto; só então lookup exato do path_key em entries observed. Snapshot de produtor histórico é aceito. Descritor inválido/divergente também falha no universo vazio ou com chave não observada.

Chave válida sem entry observed correspondente: envelope **path_not_observed/exit 1**, sem abrir objeto. Inclui ausente do registro, excluded, diretório e região desconhecida: significa somente “não há revisão observed selecionável neste snapshot”, sem alegar inexistência/remoção no acervo vivo. Não recuperar por nome parecido, traversal, descendência de excluded ou raiz do snapshot. Argumento/key malformado é invalid_arguments/2.

## 2. Verificação e elegibilidade

Selecionada a entry observed, usar exclusivamente sua revisão `{content_sha256,size_bytes}` e abrir `objects/<hash>` pelo leitor D0 existente. Não ler outros objetos do pacote: ausência/adulteração em arquivo **não selecionado** não altera o resultado de read. Essa disponibilidade por arquivo não muda a exigência de cobertura global de search/D0.

Antes de buscar linhas: admissão de tamanho/W/reserva, leitura s+1, EOF/hash/tamanho e estabilidade de objeto/componentes/ancestral, sem retry/cache. Checagens e motivos são os mesmos da busca. Nenhum texto, nem prefixo, é publicado de objeto não verificado; conservar fatos EOF mesmo quando há source_unstable. Depois: UTF-8 estrito, NUL, preflight de **todas** as linhas, com as mesmas prioridades, LF-only e max_line_bytes RD. Um arquivo com linha longa tardia bloqueia também uma seleção inicial curta.

Textualidade não normaliza BOM, CRLF, CR isolado, Unicode ou case. Linha inclui LF quando existente, último sufixo não vazio é linha, LF terminal não cria outra, vazio tem zero linhas. max_line_bytes inclui todos os bytes da linha, inclusive terminador. Reutilizar helpers RD para evitar outra definição de linhas/UTF-8.

## 3. Intervalo e continuidade

Após verificação/admissão integral, contar total_lines e selecionar de start_line até max_lines linhas consecutivas, ou EOF. Aritmética checada; não somar start+max sem tratar start=u64::MAX. Campo numérico não vira float, wrap, saturação ou índice truncado.

`text` é concatenação **exata** dos intervalos brutos selecionados; start_byte/end_byte são offsets 0-based inclusivo/exclusivo no objeto. `text.as_bytes()==object[start_byte:end_byte]`. Revisão e path_key ficam no relatório, sem novo ID/hash de trecho. returned_lines é o número real de linhas, sem estimativa de tokens.

Se ainda há linhas após a seleção: next_start_line é a próxima linha, truncated=true. Caso contrário: next_start_line=null, truncated=false. A continuação usa o mesmo snapshot/pacote e o número retornado; não significa árvore atual nem usa sessão/estado persistente. Cada chamada verifica novamente a revisão selecionada.

Start além do EOF ou arquivo vazio: **available/exit 0**, total_lines conhecido, returned_lines=0, text="", start_byte=end_byte=size_bytes, next_start_line=null e truncated=false. Isso só pode ser declarado depois de verificar o objeto; não transformar ausência da fonte em EOF. Seleção vazia não é argumento inválido quando start_line é válido.

## 4. Relatório e erros

Status refere-se ao **arquivo selecionado**, não à cobertura de todo o acervo:

- `available`/exit 0: objeto verificado, texto/linhas admitidos, inclusive seleção vazia. Um snapshot parcial pode fornecer uma revisão observed válida; snapshot.quality continua partial e scope continua declared_snapshot_only.
- `non_text`/exit 1: bytes verificados, motivo invalid_utf8 ou nul, mesma precedência RD.
- `unavailable`/exit 1: falha/limite do objeto selecionado, inclusive line_bytes_limit. Nunca entregar texto parcial como available.

Campos fechados; inteiros de domínio são strings u64 canônicas. `--json`: JCS sem newline; sem flag: mesmo valor em JSON legível com newline. Envelope de erro exato `{error:{code:<code>,message:<code>}}`, com JSON válido; invalid_arguments/incompatible_inputs=2, cancelled=130, path_not_observed e demais códigos operacionais RD=1. Nenhuma progressão/log em stdout. Cap de output refere-se ao relatório canônico; envelope fixo de erro é isento.

| Campo | Conteúdo |
|---|---|
| schema_version / read_contract_version / report_type | `1` / `0.1.0` / `verified_read` |
| producer_version / producer_build | Identidade do executável atual, framing existente |
| scope | `declared_snapshot_only` |
| snapshot | Mesma estrutura resumida RD: snapshot_id, workspace_id, path_namespace, policy_id, quality, state_digest nullable, snapshot_file_sha256 bruto |
| source | Seis campos do descritor aceito + source_ref bruto |
| request | path_key, start_line, max_lines |
| limits | Mesmos nove limites efetivos RD + max_duration_ms nullable/string |
| status / reasons | Conforme acima; motivos locais observados, únicos e em ordem lexical |
| file | path_key, expected_revision `{content_sha256,size_bytes}`, check no formato RD de cinco campos |
| excerpt | total_lines nullable, returned_lines, start_byte/end_byte nullable, text nullable, next_start_line nullable, truncated bool |
| resources | object_bytes_read, object_bytes_reserved, logical_memory_peak, strings; sem RSS/células |

`check`: state=verified/failed/not_attempted, source_ref, actual_revision nullable, hash_check/size_check nullable/bool. Sem revisão real antes de EOF. Recusa de admissão é not_attempted; leitura/abertura falha é failed; inelegibilidade textual/linha longa conserva verified. Os estados não são “corrigidos” para esconder leitura/adulteração.

Em non_text/unavailable, excerpt inteiro segue: total_lines=null, returned_lines="0", start_byte=end_byte=text=next_start_line=null, truncated=false. Em available, total_lines/start_byte/end_byte/text não são null, inclusive EOF vazio. Todos os hits de search e campos antigos permanecem iguais; read não acrescenta chave à resposta search.

## 5. Recursos e cancelamento

Reutilizar **sem mudança** os nove limites e o ledger RD: N=entries+directory_coverage+issues do manifesto único; S_manifest=8*b+4096*N+65536+8*max_output_bytes, antes do domínio; acrescentar 8*d para S; W=2*(s+1)+65536 antes de abrir/reservar o objeto. Peak=max(S_manifest,S,S+W admitido). Reservar s+1, não reembolsar, ler no máximo s+1 em chunks ≤65536. Não cobrar/read outros caminhos. Créditos são admissão lógica, não RSS/segurança contra OOM.

Mesmo no EOF solicitado, objeto completo é verificado. Não é leitura física parcial por range nem permite autenticar hash de um prefixo. Conteúdo/índices de outros arquivos não são retidos. Preparar resposta e modos de apresentação antes da publicação; canonical >max_output_bytes gera output_limit, sem truncar por bytes silenciosamente. truncated só representa max_lines, nunca falha de fonte/limite.

Preservar checkpoints compartilhados, checks antes/depois de hashing/parse/validação/JCS sem prometer preempção interna. Novos laços de contagem/seleção/cópia de linhas checam pelo menos a cada 65536 bytes; a linha admitida é ≤4096 por default. SIGINT vence deadline e erro pendente no checkpoint final; nenhum excerpt parcial sob cancelamento/deadline. I/O bloqueante pode atrasar interrupção.

## 6. Gabaritos e validação independente

Esperados escritos antes de Rust; IDs **RL01–RL16**, com ponte para AC, teste interno/CLI e versão real. Não gerar gabarito do binário nem editar RD/C01 históricas.

| ID | Cenário e esperado |
|---|---|
| RL01 | `a\nb\nc\n` (6 bytes), start=2/max=1: text `b\n`, [2,4), total=3, returned=1, next=3, truncated=true. start=2/max=20: `b\nc\n`, [2,6), returned=2, next=null. |
| RL02 | Vazio: total=0/text=""/[0,0). `\n`: uma linha [0,1), text LF. Start=4 e u64::MAX no arquivo RL01: vazio available, offsets [6,6), sem overflow. |
| RL03 | `é🙂\r\nneedle\n` (15 bytes): linha 1 [0,8), linha 2 [8,15). BOM+alvo+LF mantém os oito bytes. CR isolado/U+2028 não separam linhas; Unicode atravessando janela 65536. |
| RL04 | Nome não UTF-8, keys começando com hífen e formas separada/`=`; lookup exato e bytes preservados, sem usar o nome como caminho físico. |
| RL05 | Key inválida/absoluta/dotdot/vazia: invalid_arguments/2. Key válida sem observed, excluded/diretório/unknown: path_not_observed/1, sem leitura de objeto nem alegação de remoção. |
| RL06 | Snapshot parcial com observed verificado: available e quality=partial preservados; binding inválido no universo vazio ou key não observada continua falhando antes do lookup. |
| RL07 | Pacote contém apenas o objeto selecionado; outros objetos observed ausentes/adulterados não são lidos e não entram no orçamento. Empty selecionado ainda reserva 1/prova EOF. |
| RL08 | Ausente/adulterado mesmo tamanho/menor/maior, symlink em raiz/objects/objeto, FIFO/socket: motivos/checks/exits corretos; excerpt null e zero bytes externos. |
| RL09 | Barreira de substituição antes da reconferência do leitor: source_unstable, sem excerpt, preservando fatos EOF. Controle estável tem mesmo conteúdo/range esperado. |
| RL10 | UTF-8 inválido/NUL e precedência; linha de 4096 admitida/4097 bloqueada, inclusive linha longa depois da seleção; nenhum texto parcial reaproveitado. |
| RL11 | Caps de input/profile/N/profundidade, s+1/W/S, tamanho/total/reserva/peak, igualdade e recusa nas fronteiras, u64/overflow. Mesmas fórmulas RD, read debita só a seleção de arquivo. |
| RL12 | SIGINT/deadline com barreiras nos laços/antes de publicar; prioridade cancelamento e nenhum relatório parcial. Não vender primitivo compartilhado como preemptível. |
| RL13 | JCS/campos/u64/pretty/newline/output exato e cap-1; start=0/max=0/1001/negativo/float/overflow; path-key literal --json com erro de linha não ativa flag, flag real segue válida; BUG-005 search permanece corrigido. |
| RL14 | Fluxo init→snapshot→pacote→search→read de vizinhança e continuação; ausência da árvore viva não muda bytes. S0/S1 mesmo path entregam suas próprias revisões; pacote trocado falha. Inputs intactos. |
| RL15 | Carga/medição abaixo: um único objeto lido, três invocações nativas com stdout idêntico, RSS separado do ledger, sem SLO. |
| RL16 | Controle válido e mutações específicas de text, offsets, revisão, next_start_line/truncated e available indevido sobre falha: rejeições atribuíveis, não erro genérico. |

Carga: mesma receita RD de 1000 arquivos de 4096 bytes/100 diretórios, BASE só em j=31. Search fornece um hit; read seleciona `dir-000/file-0000.txt`, start_line=30/max_lines=5: total=64, returned=5, [1856,2176), 320 bytes de texto, next=35/truncated=true, object_bytes_read=4096/reserved=4097. Guardar três amostras GNU time do filho nativo, wall, ambiente/cache/build/disco/ledger, inputs/outputs representativos completos em base64+SHA; não incluir compilação no tempo nem aplicar SLO.

Regressão da versão nova é obrigatória: RD/C01 intactas (48 controles + campanhas RD/Foundation/D0), testes atuais e novos. Não executar R1 fixada ao release antigo nem retirar pins/gabaritos para caber a identidade nova. Após entrega haverá auditoria; preparar contrato não implementa read.
