# Compass Foundation

Compass prepara acervos locais, conserva revisões por SHA-256, oferece busca literal/lexical/semântica, navega relações estruturais e entrega evidências verificadas com expansão BFS ou difusão de calor opcional. Snapshots, comparação e D0 permanecem disponíveis.

O [plano único e progresso](docs/PLANO-IMPLEMENTACAO.md) orienta a execução. OSs e coordenação antigas são históricos. Motor e CLI em Rust; Python é usado somente na verificação independente.

Suporte inicial: Linux com `openat2` (kernel 5.6 ou posterior). A observação é uma janela de leitura, não um snapshot atômico. Arquivos vazios, binários e nomes não UTF-8 são suportados. O resultado distingue diferença conhecida de observação parcial.

## Compilar

A toolchain está fixada em Rust 1.90.0 e as dependências em `Cargo.lock`. Neste workspace, Rust foi instalado em `../.tooling/`, sem modificar o PATH global. O wrapper também funciona com uma instalação Rust convencional quando essa pasta não existe.

```bash
./scripts/cargo-local.sh build --release --locked
./target/release/compass --help
```

Depois do primeiro download de dependências, builds e testes podem usar `--offline`.

## Usar

```bash
./target/release/compass init --root /caminho/do/acervo --json
./target/release/compass snapshot --root /caminho/do/acervo --json
```

`init` cria `.compass/` com identidade e política; repetir o comando preserva a identidade. `snapshot` devolve o caminho do manifesto. Após alterar o acervo, execute outro snapshot e compare os caminhos devolvidos:

```bash
./target/release/compass compare \
  --baseline /caminho/do/primeiro.json \
  --current /caminho/do/segundo.json \
  --json
```

Sem `--json`, a saída usa JSON identado legível. Com a opção, stdout contém um objeto canônico por execução. Encontrar diferenças é sucesso operacional, código `0`; resultado parcial retorna `3`, erro operacional/registro inválido `1`, configuração ou comparação incompatível `2`, e SIGINT `130`.

Os campos `*_path` dos recibos são auxiliares para exibição. Para automação que aceite qualquer nome Linux, decodifique `*_path_base64`: base64url sem padding dos bytes do caminho absoluto. Os caminhos relativos dentro dos manifestos usam `path_key`, também com preservação de bytes.

## Política de observação

A configuração efetiva está em `.compass/policy.json` e é copiada para cada manifesto. Ela inclui exclusões e limites de arquivos, bytes, entradas e duração. Os limites são strings decimais positivas ou `null` quando não configurados. Configurações inválidas são rejeitadas; alteração de política impede comparar snapshots como se tivessem o mesmo universo.

Duração e cancelamento são cooperativos entre operações. Não interrompem uma chamada de filesystem bloqueada, como pode ocorrer em NFS. Sem limites configurados, o tamanho do manifesto e a memória crescem com o inventário.

Por padrão, diretórios `.git`, `.compass`, `.aws` e `.ssh`, arquivos `.env` e `.env.*`, links e tipos especiais são excluídos. `.gitignore` não é aplicado implicitamente. Exclusões adicionais em `exclude_paths` são caminhos relativos codificados em base64url sem padding. `node_modules` e `target` devem ser excluídos explicitamente quando não fazem parte do acervo desejado.

O conteúdo é identificado por seus bytes, sem normalização. Renomeações aparecem como remoção e adição. A permissão de execução integra o estado; mudanças somente em mtime não alteram a revisão.

## Verificar

```bash
./scripts/cargo-local.sh test --workspace --locked --offline
./scripts/cargo-local.sh fmt --all -- --check
./scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
python3 verification/reference/acceptance.py --help
```

Para repetir a carga de recursos de 1.000 arquivos (requer `/usr/bin/time`):

```bash
python3 verification/reference/profile.py \
  --binary target/release/compass \
  --output verification/results/local-profile.json
```

Os gabaritos manuais e o mapeamento F01–F20 estão em [verificação](verification/ACCEPTANCE.md). O [contrato ativo](research/contracts/FOUNDATION.md) e o [schema de transporte](research/contracts/transport.schema.json) definem o comportamento, inclusive as limitações da leitura concorrente.

## Escopo e proveniência

Snapshots guardam manifestos; os bytes usados por D0, busca e leitura são conservados em pacotes explícitos ou capturados pelo preparo de acervos. Acervos gerenciados incluem índices lexicais, embeddings opcionais e grafos estruturais, conforme o [guia de recuperação](docs/ACERVOS-RECUPERACAO.md). A [aquisição por obrigações](docs/AQUISICAO-OBRIGACOES.md) acrescenta avaliação das evidências entregues no pack. Interface gráfica permanece fora do escopo atual.

O build inclui `producer_build`, um SHA-256 dos fontes, manifests Cargo, lockfile, toolchain e contratos, incluindo alterações locais. Esse identificador não depende de existir um commit e não equivale a uma assinatura de autoria.

A [edição documental original](../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md) é a autoridade de produto. A [análise inicial](../docs/ANALISE_INICIAL.md) explica a decomposição da implementação. Esses materiais permanecem fora deste repositório de código e preservados no diretório pai. O [estado operacional](COMPASS_STATE.md) registra verificações e limitações desta versão.

## Sensor D0: pacotes externos verificados

```sh
./target/release/compass d0 --baseline baseline.json --current current.json \
  --baseline-source pacote-antigo --current-source pacote-atual --json
```

D0 mede a fração exata de massa bilateral de linhas sem correspondência em uma
LCS, incluindo arquivos inalterados. O operador prepara e conserva os bytes das
duas revisões: cada pacote contém `source.json` e `objects/<sha256>`. O descritor
possui exatamente `source_schema_version="1"`, `source_contract_version="0.1.0"`,
`workspace_id`, `snapshot_id`, `policy_id` e `snapshot_file_sha256`, este último
calculado sobre os bytes exatos do manifesto usado. Objetos são os bytes brutos,
inclusive BOM, CRLF e vazios, com hash e tamanho iguais à revisão do manifesto.
Consulte o contrato ativado em `research/proposals/d0-c2/CONTRATO_CANDIDATO.md` e
a ativação em `docs/entregas/d0/ATIVACAO.md`. Não há captura/retenção automática.
Mantenha os pacotes fora do acervo; eles contêm os conteúdos e podem conter dados
sensíveis. Um hash não permite recuperar conteúdo antigo que não foi conservado.

Fontes omitidas, objetos ausentes/adulterados e regiões desconhecidas impedem o
resultado integral; o subconjunto conhecido é informado separadamente. Binários
(UTF-8 inválido ou NUL) e transições de escopo ficam fora do universo textual.
Zero textual não significa igualdade de bytes: remove-se um BOM inicial,
normaliza-se CRLF para LF e despreza-se só o fragmento final de um LF terminal.

`--limits perfil.json` recebe o objeto completo dos onze limites estáticos de
`research/proposals/d0-c2/fixtures/limits-default.json`; só reduções positivas são
aceitas. `--deadline-ms 1..3600000` configura prazo cooperativo, com SIGINT tendo
prioridade. Não há prazo rígido para syscall bloqueada. Igualdade exata das linhas
verificadas reserva zero células; os demais pares usam LCS exata com memória
linear. Bytes têm reserva conservadora irreversível de tamanho declarado + 1.
A memória publicada é um ledger lógico de admissão; não é RSS nem garantia contra
OOM. Esta entrega não mede RSS nem afirma eficácia científica/contextual da métrica.

Com `--json`, stdout é um objeto JCS sem newline: exit 0 completo, 3 parcial,
1 indisponível/falha, 2 argumentos/incompatibilidade, 130 cancelamento. Falhas
usam `{error:{code,message}}`, com `message=code`, sem relatório parcial de aborto.
O modo humano distingue integral e subconjunto conhecido. Os comandos Foundation
`init`, `snapshot` e `compare` preservam sua interface e transporte anteriores.

A implementação e os testes da OS-D0-I1 são registrados em
`verification/d0-i1/RESULTADO-D0-I1.md`; a auditoria independente e o aceite R1
foram concluídos (histórico em `COMPASS_STATE_HISTORICO.md`). Para repetir a V1 intacta contra um binário D0
estável, use `python3 verification/d0-i1/run-v1.py --binary target/release/compass
--cached-dependencies /home/many/.cache/uv/archive-v0`. O adaptador usa os 34
arquivos históricos arquivados antes de I1 e não recompila nem troca o binário.

## Busca literal da revisão declarada

`search` recebe um snapshot e um pacote externo D0 explicitamente vinculado aos
bytes desse manifesto. O pacote contém `source.json` (os seis campos de binding
D0) e `objects/<sha256>` com os bytes observados. Não há retenção automática.
Para um acervo sintético com `a.txt` contendo `alvo\nNada\nalvo alvo\n`,
crie `init` e `snapshot`, copie esses bytes para o objeto SHA-256 e escreva o
binding do pacote ao manifesto recebido. Então execute:

```sh
target/release/compass search --snapshot /tmp/exemplo/snapshot.json \
  --source /tmp/exemplo/package --query alvo --max-matches 20 --json
```

A resposta entrega linhas 1 e 3, intervalos de bytes `[0,5)` e `[10,20)`, com LF
preservado. Matching é literal e sensível a caixa; BOM, Unicode e CRLF não são
normalizados. Um hit representa uma linha, mesmo com várias ocorrências.
Caminhos são identificados por `path_key` em base64url, ordenados por bytes
nativos. Os conteúdos vêm exclusivamente dos objetos verificados, inclusive
quando a árvore viva já foi removida.

`scan.status` indica cobertura: complete/0, partial/3 ou unavailable/1. Falhas de
fonte não viram ausência completa. `matching_lines_known` conta o subconjunto
conhecido; `matches_truncated` indica seleção limitada. O scan continua após o
limite de hits para contar linhas e detectar falhas posteriores. Não há ranking
ou paginação. UTF-8 inválido/NUL são `non_text`; linha acima de 4096 bytes torna
o arquivo indisponível, sem entregar seus hits.

Query admite 1–256 bytes UTF-8 sem NUL/CR/LF; `--max-matches` admite 1–1000,
default 20. `--limits perfil.json` reduz os nove limites completos do
[contrato RD](docs/entregas/rd/CONTRATO-RD-V0.1.md); todos são strings u64
canônicas. `--deadline-ms` admite 1–3600000. Há limites de manifesto, descritor,
registros, profundidade, arquivo, bytes totais, saída, memória lógica e linha.
A admissão lógica não é controle de RSS/alocador nem garantia contra OOM.
Cancelamento/deadline são cooperativos: I/O bloqueante e primitivas compartilhadas
podem atrasá-los. O relatório é preparado integralmente antes da publicação;
falha no pipe pode deixar prefixo. `--json` emite JCS sem newline; sem a flag,
o mesmo objeto aparece como JSON legível com newline. A evidência pertence
somente ao snapshot declarado; não demonstra ausência na árvore viva.

Campanha reproduzível, apenas com árvores sintéticas:

```sh
python3 verification/rd/acceptance.py --self-test
python3 verification/rd/acceptance.py --binary target/release/compass
python3 verification/rd/acceptance.py --regressions --binary target/release/compass \
  --cached-dependencies /home/many/.cache/uv/archive-v0
```

## Leitura e continuação

Use `path_key` e a linha retornados por `search` para abrir o contexto:

```sh
target/release/compass read --snapshot /tmp/exemplo/snapshot.json \
  --source /tmp/exemplo/package --path-key YS50eHQ \
  --start-line 1 --max-lines 20 --json
```

`YS50eHQ` identifica `a.txt`. `start-line` começa em 1; `max-lines` admite 1–1000
e tem default 20. Para continuar, repita os mesmos snapshot/pacote/chave com
`excerpt.next_start_line`, até ele ser null. Cada chamada verifica integralmente
o objeto selecionado e preserva seus bytes (incluindo BOM, CRLF e LF). Offsets
são intervalos de bytes [start_byte,end_byte), não posições de caracteres.

`available`/exit 0 inclui vazio e seleção além do EOF, com texto vazio e offsets
no final. `non_text`/exit 1 indica bytes verificados que não são texto admitido;
`unavailable`/exit 1 não entrega excerpt. Chave válida sem revisão observada é
`path_not_observed`/exit 1, não prova de remoção da árvore viva. Um snapshot parcial
pode fornecer um arquivo disponível; a qualidade parcial continua explícita.

São os mesmos `--limits` e `--deadline-ms` da busca. A chamada lê só o objeto
selecionado; arquivo não selecionado ausente não a invalida. A verificação completa
precede a seleção mesmo se ela estiver além do EOF; não há leitura autenticada
de prefixos. Cancelamento/deadline são cooperativos. JSON compacto não tem LF;
sem `--json`, a resposta é JSON legível com LF.

Verificação sintética, sem regravar evidência anterior por padrão:

```sh
python3 verification/reading/acceptance.py --self-test
python3 verification/reading/acceptance.py --binary target/release/compass
```

## Aquisição espectral experimental

O incremento está implementado e avaliado como experimental no [plano de implementação](docs/PLANO-IMPLEMENTACAO.md#próximo-incremento--aquisição-espectral-adaptativa). Para uma coleção preparada:

```sh
target/debug/compass retrieve --store /caminho/colecao --query nome_da_funcao \
  --mode lexical --acquisition-policy spectral_window_v1 \
  --scope dependencies --budget-bytes 16384 --json
```

`scope` é obrigatório nesse modo: `local`, `dependencies` ou `callers`. As
sementes são descobertas internamente. O pack terminal de schema 3 separa
condição estrutural, motivo de término e cobertura/lacunas. `next_cursor=null`
encerra o procedimento; não declara exaustão do acervo nem suficiência da
resposta. Contrato, cursor e theta explícito são incompatíveis com essa política.
Sem `acquisition_policy`, os caminhos existentes mantêm seu comportamento.
A geração inteira deve ter até 128 documentos e 22.369.621 bytes para
revalidação estrutural; acima disso, o resultado terminal indica limitação sem
evidências. O orçamento de referência é 16 KiB; 3 KiB não comportaram os
metadados completos nos controles.

O cálculo usa Rust/f64 esparso; NumPy serve apenas à verificação offline. A
política permanece experimental e sua seleção não substitui a análise dos
testemunhos entregues. Evidências e medições novas ficam em
[`verification/spectral/`](verification/spectral/README.md).

## MCP local

Para conectar o Multiagentes nesta máquina, a instalação local já está preparada:
consulte [comando, fontes e teste de conexão](docs/MCP-INTEGRACAO.md).

O adaptador `mcp` usa entrada/saída padrão, sem porta de rede, com protocolo
MCP **2025-11-25** explicitamente negociado. Um cliente que não aceite essa
versão deve encerrar a conexão. Ferramentas: `compass_search`, `compass_read`,
`compass_files`, `compass_query`, `compass_neighbors` e `compass_retrieve`.
Todas chamam o mesmo núcleo da CLI e entregam `structuredContent` e conteúdo
textual JSON equivalentes. Texto recuperado é dado não confiável do acervo.

Crie um arquivo de configuração local (os caminhos relativos são resolvidos a
partir da pasta desse arquivo):

```json
{
  "sources": {
    "meu-acervo": {
      "snapshot": "revisao/snapshot.json",
      "source": "revisao/package"
    }
  },
  "deadline_ms": 30000
}
```

```sh
target/release/compass mcp --config /caminho/compass-mcp.json
```

No cliente MCP, use o caminho absoluto do executável como `command` e
`["mcp", "--config", "/caminho/compass-mcp.json"]` como `args`.
A configuração admite de 1 a 128 fontes; nomes usam letras ASCII, números,
`_` e `-`. O objeto opcional `limits` é o perfil completo de nove limites da
busca; só reduções são aceitas. Prazo padrão: 30 segundos, cooperativo.

Exemplos de argumentos das ferramentas:

```json
{"source":"meu-acervo","query":"calcularTotal","max_matches":20}
```

```json
{"source":"meu-acervo","path_key":"YS50eHQ","start_line":"1","max_lines":20}
```

No MCP, `start_line` é string decimal para conservar todos os valores u64.
Use a mesma fonte e chave com `next_start_line` para continuar. As ferramentas
não aceitam caminhos físicos nem permitem escolher outra raiz. Não capturam
arquivos, executam código do acervo ou alteram suas fontes.

Uma consulta de conteúdo é processada por vez; outra chamada simultânea recebe
`Server busy` e pode ser repetida após a primeira terminar. Ping e notificações
de cancelamento continuam responsivos. O cancelamento usa os checkpoints do
núcleo; I/O bloqueado pode atrasar o encerramento. Mensagens de entrada são
limitadas a 64 KiB; os limites do núcleo continuam valendo para os relatórios.
A resposta MCP inclui também o envelope e a representação textual dos dados.

Teste com um cliente stdio real de verificação, sem alterar configurações de
aplicativos:

```sh
python3 verification/mcp/acceptance.py --binary target/release/compass
```

Esse teste demonstra o protocolo e o fluxo pela CLI/cliente; não representa
uma avaliação de benefício com agentes nem configuração automática do seu host.

## Acervos e recuperação local

Preparação automática, índice lexical, símbolos, semântica em CPU e evidence packs com expansão BFS/calor: consulte o [guia de acervos e recuperação](docs/ACERVOS-RECUPERACAO.md). Os comandos anteriores continuam compatíveis.
