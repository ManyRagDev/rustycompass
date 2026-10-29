# Laboratório independente D0 — OS-D0-V1 v2

O contrato foi ativado em 19/10/2026 em `docs/entregas/d0/ATIVACAO.md`.
Este laboratório consome o conjunto C2-A1 congelado por leitura. Não importa
Rust, o checker C2 ou funções de produção; não compila nem substitui binários.

## Reprodução offline

Na raiz de `compass-foundation`:

```sh
python3 verification/d0/acceptance.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
python3 verification/d0/mutations.py --self-test --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Python 3 e jsonschema com Draft 2020-12 são necessários. O cache é opcional,
local à máquina: omita a flag se jsonschema e suas dependências já estiverem
no Python. Nenhum pacote é instalado. As cinco definições de schema e as
referências Foundation são resolvidas em arquivos locais permitidos; sem rede.

O self-test precisa criar symlinks, FIFO, socket Unix e hardlinks em `/tmp`.
Um sandbox que proíba `bind()` em sockets Unix impede a montagem: o processo
falha e registra a limitação; isso não vira teste pulado/aprovado. Nesta entrega,
os self-tests finais foram executados com permissão técnica para esse socket
local fora do sandbox. Não há conexão de rede nem leitura de acervos reais.

`results/self-test.json` registra os IDs e testes efetivos, falhas, modalidade,
integridade, hashes dos gabaritos e pendências de runtime. `mutations.json`
identifica 11 mutações, controles positivos, aprovação no schema e asserção
semântica responsável pela rejeição. Falha genérica/import/timeout não conta
como mutação rejeitada. Ambos retornam 1 em falha, 0 em sucesso.

## O que está efetivamente conferido offline

- Hash autorizado do índice, 81 artefatos e 34 arquivos protegidos; adulteração
  em cópia temporária detectada sem modificar os originais.
- 44 cálculos literais e 23 agregados congelados; LCS por programação dinâmica,
  conferida por enumeração de subsequências nos exemplos pequenos e em 961
  pares binários. Igualdade grande usa `LCS(X,X)=|X|`, sem DP quadrático.
- Normalização dos bytes, SHA-256, arredondamento inteiro com empate para par,
  cinco schemas, dez envelopes e dez relatórios congelados com seus inputs,
  canonical bytes, binding, comparação, checks, recursos e disponibilidade.
- Montagem física de 116 cenários. Para cenários que retornam relatório,
  o modelo estático e os literais são conferidos e o relatório esperado passa
  no schema. Cenários de erro têm montagem e envelope esperado preparados;
  essa preparação não afirma que o sensor emitiu o erro.
- Negativos schema-válidos: falso zero sem fonte, agregado sem unchanged,
  igualdade por contagem, débito quadrático na igualdade, política incoerente,
  subconjunto integral, overflow u64, path inválido, falso binding, memória
  falsa e ordem nativa errada. Cada controle válido passa antes da mutação.

Os esperados vêm dos E01–E55 congelados, de literais adicionais em
`fixtures.json`/receitas de `cases.py` e das equações independentes do contrato.
Não são capturados de Rust. `coverage.json` associa todos os 55 IDs a testes,
camadas, cenários concretos e responsável pela prova restante. `coverage-map`
confere referências e completude da matriz a cada self-test.

## Campanha pela CLI: preparada, não executada nesta entrega

Somente quando a coordenação fornecer um release D0 estável:

```sh
python3 verification/d0/acceptance.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
python3 verification/d0/mutations.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
```

A campanha usa exclusivamente a interface pública `compass d0 --baseline ...
--current ... [--baseline-source ...] [--current-source ...] [--limits ...]
[--deadline-ms ...] --json`. O Foundation anterior não possui D0 e não foi
usado como substituto. Um binário sem interface D0 produz falha explícita.

O harness registra SHA-256 do binário antes/depois, exige estabilidade, monta
inputs próprios, compara o relatório com o modelo e os literais, confere exit,
JCS sem newline, repetibilidade em duas invocações e inventário dos inputs
(manifestos, descritores, objetos, links e tipos especiais). Identificadores
matemáticos nunca são substituídos por resultados da CLI. O produtor emitido
é registrado junto ao hash do artefato; o hash não prova sozinho sua origem.

Timeouts de guarda são 30 s por invocação e 10 s para o preflight da interface;
retornam falha. Não medem nem demonstram o deadline do produto. O caso público
`deadline-public` exige o envelope deadline_exceeded com exit 1. O smoke de
SIGINT informa NAO_VERIFICADO se o processo terminar antes do sinal/handler;
exit 130 só demonstra o cancelamento público observado, sem provar o ponto
interno exato. Essas provas pendentes permanecem explícitas mesmo quando um
smoke for observado.

`results/acceptance.json` só é gerado ao invocar o modo `--binary`. Não existe
resultado de execução contra um sensor D0 nesta entrega. O modo `--binary`
também repete os controles offline antes de iniciar testes do sensor.

## Limites da prova e transferência para I1/R1

O modelo Python recompõe entradas/estados efetivos, comparação, normalização,
LCS, admissões lógicas, reservas e saída esperada de árvores estáticas do
laboratório. Suas consultas a metadados não implementam abertura segura de
produção e não demonstram openat2, TOCTOU, estabilidade concorrente ou atomicidade.

I1/R1 devem executar provas com barreiras de substituição após abertura,
metadados/nome antes/depois, sem retry; SIGINT após p/antes de q e prioridade
sobre deadline/erro; checkpoints em chunks e a cada no máximo 4096 operações;
preflight estrutural antes de construir o domínio. Essas provas estão na
matriz e em `fixtures.json`. Não há flags públicas fictícias para sincronizá-las.
Repetir uma corrida probabilística não conclui essas provas.

E49 confere vetores exatos do formatador offline; os extremos exigem testes
diretos de aritmética Rust em I1/R1, sem alegar montagens pela CLI acima dos
limites default. O ledger lógico não é RSS nem garantia contra OOM. Medição
RSS, regressão Foundation, fmt/Clippy/build e homologação Rust/CLI pertencem a R1.

A implementação da V1 pode estar concluída com self-test aprovado e campanha
entregue; isso não homologa o sensor. Resultados gerados são ignorados pelo Git
somente em `results/`; o relatório durável é `RESULTADO-D0-V1.md`.

## Correção C01 / BUG-001 — implementada, aguardando auditoria

`regression:BUG-001` aceita os paths `1111` e `111111111111111111111111`
na validação nativa, snapshot, modelo e relatório: completos, `0/2`, massa dois,
zero células. Os cenários `BUG-001-path-4` e `BUG-001-path-24` estão preparados
para runtime e vinculados a E24/E50. O self-test passou com 129 verificações e
118 cenários após C01. Contadores/limites são selecionados por campo e papel;
`metric_policy.unknown` é texto, enquanto `comparison.counts.unknown` é u64.
A regressão confere todos os 49 campos numéricos encontrados nos schemas,
aceitando u64::MAX e rejeitando overflow como `u64`, sem limitar identificadores
pela aparência numérica. Esta conferência não executa Rust/CLI D0.

## Correção C02 / BUG-002 — implementada, aguardando auditoria

`regression:BUG-002` confronta objetos regulares válidos, ausência genuína e
links intermediários existentes/pendentes com motivos literais. O relatório
correto para link pendente passa; a variante schema-válida `object_missing`
é rejeitada por `report_units`. No par `abc` bilateral, os casos bloqueados
reservam 8 bytes, leem 3 no lado current, preservam current verified e não
inventam revisão na baseline. `E43-objects-dangling-link` está na campanha e
matriz E43, junto aos controles existentes de link direto/FIFO/socket/diretório.
Self-test pós-C02: 131 verificações, 119 cenários, zero falhas; 11 mutações
continuam aprovadas. A inspeção por lstat só modela árvores estáticas; segurança
concorrente/openat2 e checkpoints permanecem pendentes para I1/R1.

## Correção C03 / BUG-003 — implementada, aguardando auditoria

O cenário E52 consome os hexadecimais congelados: baseline `610d0a` e current
`610a`. `assert_e52_stimulus` verifica objetos físicos, revisão/hash/tamanho e
binding dos descritores antes de conceder cobertura, tanto no prepare quanto
no modo runtime. A comparação Foundation deve ser modified; os checks devem
ser verified e as linhas normalizadas iguais, com `0/2`, massa dois, decimal
`0.000000` e zero células. A prova `regression:BUG-003` preserva no JSON os
bytes/hashes observados e rejeita a receita defeituosa reconstruída somente de
linhas normalizadas por `bug003_stimulus_bytes`, embora seu schema e D0 zero
sejam válidos. Retirado o crédito indevido de `byte-hash-schema-vectors` em E52;
a matriz aponta a prova física correta. `bytes-E16a` e E48 permanecem aprovados.

Estado acumulado após C01 → C02 → C03: 132 verificações, 119 cenários preparados,
11 mutações específicas aprovadas; runtime D0 não executado. As correções foram
implementadas/testadas pelo executor e aguardam auditoria focada dos três BUGs;
a conclusão histórica da entrega original não equivale ao aceite final de V1.
