# Verificação independente do Foundation 0.1

Os resultados esperados de `fixtures/manual_expectations.json` foram escritos
antes da execução da CLI. S0 contém `a.txt=abc` e `b.txt` vazio; S1 contém
`a.txt=abd` e `c.txt` vazio. O gabarito é modificação, remoção e adição, nessa ordem.
O laboratório Python usa somente a biblioteca padrão: altera arquivos pequenos
em diretórios temporários, executa a CLI pública e confere suas respostas e os
manifestos publicados. Não implementa outro comparador nem importa código Rust.
SHA-256 de vazio e `abc` têm vetores fixos; outro conteúdo é conferido com
`hashlib`. A comparação precisa funcionar depois da remoção da árvore original.

```sh
./scripts/cargo-local.sh build --release --workspace --locked --offline
python3 verification/reference/acceptance.py --binary target/release/compass
python3 verification/reference/mutations.py --binary target/release/compass
```

O segundo comando falha se qualquer saída `--json` contiver texto adicional,
chaves duplicadas ou código de saída inesperado. A comparação também deve
preservar byte a byte os dois manifestos recebidos. O terceiro injeta três erros
deliberados na resposta pública e exige rejeição por assertions: sempre igual,
omissão de vazios e transformação de inacessibilidade em remoção. São mutações
do protocolo, sem alegação de cobertura exaustiva das mutações do código fonte.

| Aceite | Verificação independente na CLI | Limite desta camada |
|---|---|---|
| F01 | Acervo vazio, repetição, IDs distintos e digests iguais | — |
| F02–F05 | Criação, remoção, vazios, substituição de mesmo tamanho; gabarito S0/S1 e hashes fixos | — |
| F06 | A → B → A; igualdade dos extremos e baseline preservado | — |
| F07 | Renomeação resulta em remoção e adição | — |
| F08 | `chmod 000` em arquivo produz parcialidade e `unknown` | Pula sob root, que pode ignorar permissões |
| F09 | Diretório sem enumeração preserva região desconhecida e descendente `unknown`; manifesto parcial também sobrepõe uma leitura bem-sucedida | Negação por permissões pula sob root |
| F10 | Hook Rust `modifications_during_both_attempts_never_accept_a_revision` intervém depois de ler bytes nas duas tentativas | Uma corrida externa sem sincronização seria probabilística |
| F11 | Links internos/externos/cíclicos, FIFO, exclusões de política | O destino externo fica fora da árvore testada |
| F12 | Nome não UTF-8, caixa distinta, hardlinks; ordenação por bytes nativos; raiz não UTF-8 com recibos reutilizáveis por seus campos base64 | Linux |
| F13 | CRLF/LF e BOM são mudanças de bytes | — |
| F14 | Acervos e políticas incompatíveis, schema desconhecido | Erro 2 na incompatibilidade; schema ilegível é registro inválido, erro 1 |
| F15 | Limite por arquivo, total de bytes e entradas produz parcialidade sem digest | Duração é exercitada por hook nos testes Rust; não é promessa de interrupção instantânea de I/O bloqueante |
| F16 | Três escritores concorrentes publicam IDs diferentes; init mantém identidade; SIGINT real durante espera de lock retorna 130 e não publica | Hook Rust `cancellation_before_publication_leaves_no_final_or_temporary` verifica cancelamento após sincronização, antes da publicação |
| F17 | Hook Rust `symlink_substitution_before_open_never_reads_external_target` troca o diretório ancestral por link e verifica zero bytes externos lidos | Não se confunde teste de link estático com teste de corrida |
| F18 | JSON inválido, campo repetido, entrada duplicada, digest adulterado, caminho inválido, enum desconhecida | — |
| F19 | Repetição da comparação é idêntica em bytes; mtime não altera digest; execução altera metadata | — |
| F20 | Acervo vazio, arquivo/diretório, arquivo/link e transições reversas | — |

## Registro de execução

Em 14 de outubro de 2026, os dois comandos Python acima foram executados contra
o binário **release**, em Linux, UID 1000. Resultado: **23 testes passaram**, em
0,349 s, e **os três mutantes foram rejeitados**, sem testes pulados. Os arquivos
de trabalho foram criados em temporários e removidos ao fim. Não foi observado
acesso ao conteúdo dos alvos externos dos links nem modificação dos baselines.

A integração também executou **40 testes Rust** (20 unitários e 20 de contrato),
sem falhas ou ignorados, além de `cargo fmt --check` e Clippy com warnings tratados
como erro. Os testes Rust incluem os hooks F10, F16 e F17 descritos acima. A
verificação independente leu os testes e as rotas públicas correspondentes.

Identificação da execução (também em `reference/release-verification.json`):

- Binário: `target/release/compass`, versão 0.1.0, schema 1, contrato 0.1.0.
- SHA-256 do binário: `c95d458792b24a1173b5c04d5d284841482e48e5198ab2e5aca04fc1e932c41a`.
- `producer_build` capturado em snapshot temporário: `sha256:94866a9d405c283d1913c13e53ff0d5413823f3ce1a654e8bbc68619d7ddd4b0`.

Na conferência posterior foram examinados comparação por ancestral, parser de
JSON com chaves duplicadas, coerência das revisões, limites, abertura a partir de
descritores, lock e publicação sem substituição. A revisão encontrou e a
integração corrigiu a classificação de erros: política inválida dentro de um
snapshot é registro corrompido (1); política inválida como configuração é erro
de configuração (2). Ambos os caminhos têm testes na CLI.

A conferência também reproduziu perda de bytes no caminho absoluto dos recibos:
uma raiz contendo `0xff` era inicializada e observada, mas o caminho legível
devolvido não apontava ao arquivo publicado. O teste foi escrito e falhou antes
da correção. Os recibos agora incluem `workspace_path_base64`,
`policy_path_base64` e `snapshot_path_base64`, com os bytes absolutos codificados
em base64url sem padding; os campos legíveis são auxiliares. O teste passou após
a correção, incluindo comparação usando os caminhos decodificados. Os helpers
da suíte também usam essa representação sem perda.

Os hashes fixos da política e do estado da fixture ASCII foram recalculados com
`hashlib` e JSON ordenado da biblioteca padrão e coincidiram com o contrato.
Essa checagem restrita não é uma implementação Python completa de JCS; os testes
Rust também exercitam vetores de números e ordem UTF-16, usando `serde_jcs`.

O aceite confirma conformidade nesses casos de Foundation. Não comprova uma
fotografia atômica de filesystem vivo, ausência de toda corrida possível,
autenticidade de um manifesto, limites rígidos de RAM ou eficácia futura de D0,
recuperação e difusão. Nenhum conteúdo histórico é armazenado nos manifestos.
