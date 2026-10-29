# Contrato de aceite D0-R1 — integração e recursos

Versão 1, 19/10/2026. Contrato operacional da [OS-D0-R1](OS-D0-R1.md), sem alteração de significado, schema, limites ou gabaritos do sensor. Autoridade de produto: [ativação P1–P4](ATIVACAO.md) e conjunto C2-A1 congelado. Produção Rust permanece protegida nesta etapa.

## Baseline apta à integração

I1 e sua correção C01 foram verificadas. [Auditoria C01 CONFORME](../../../research/audits/AUDITORIA-OS-D0-I1-C01.md), SHA-256 `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3`; BUG-004 verificado, sem achado obrigatório remanescente nesse escopo.

- Binário aprovado: SHA-256 `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`.
- Produtor dos 37 inputs efetivos: `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`.
- Política default: `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.
- Índice C2-A1: `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`, 81 artefatos.

R1 identifica tanto os fontes/contratos quanto os bytes do executável. Não existe HEAD/commit. Hash não constitui assinatura de autoria. Binário ausente/divergente ou inputs alterados exigem reconferência da baseline, sem rebuild silencioso ou atribuição de provas antigas à nova versão.

## Integração real pela CLI

Cada carga usa acervo artificial próprio em `TemporaryDirectory`, com pacotes fora dele. Executar `init`, `snapshot` S0, preparar pacote baseline verificando bytes/hash/tamanho das entradas observed, aplicar mudanças, `snapshot` S1 e preparar pacote current. Copiar os dois manifestos brutos para fora do acervo; os descritores vinculam exatamente esses bytes. Não reconstruir snapshots com o modelo de testes nem obter os esperados da resposta D0.

Executar `compare` separado e `d0` com os dois pacotes; conferir comparação, checks, matemática, saída JCS sem newline e exit contra este contrato e a V1 independente por leitura. O empacotador é uma simulação explícita do operador no laboratório, sem nova capacidade de captura/retenção em produção. Não copiar excluded, links ou conteúdo de acervos existentes.

Depois dos controles normais, retirar somente a árvore viva artificial, conservando os manifestos externos e os pacotes. D0 deve repetir os mesmos bytes de relatório, demonstrando independência do acervo vivo. A remoção fica restrita a caminhos criados e registrados pelo próprio runner no temporário.

## Cargas com esperados independentes

Os literais abaixo devem ser transcritos em `verification/d0-r1/fixtures.json` e conferidos pelo self-test **antes de invocar o sensor**. Não ajustar a fixture para copiar resultados da CLI.

### M — muitos arquivos e massa de unchanged

S0: 1.000 arquivos, índices 0..999; caminho `dir-{i // 10:03d}/file-{i:04d}.txt`, 100 diretórios. Cada arquivo contém 64 linhas, índices j=0..63, cada uma com exatamente 64 bytes: `f"F{i:04d}:L{j:02d}:BASE".ljust(63, "x") + "\n"`, ASCII. Total 4.096 bytes por arquivo. Não alterar executabilidade.

S1: nos arquivos 0..9 substituir somente `BASE` por `EDIT` na linha j=31; remover arquivos 10..14; adicionar arquivos 1000..1004 pelo mesmo gerador, no diretório já existente `dir-000`. Nenhum diretório de S0 fica vazio. Os índices novos devem usar explicitamente `dir-000/file-{i:04d}.txt`, sem criar `dir-100`. Arquivos de configuração `.compass` permanecem exclusões Foundation, fora das contagens de arquivos elegíveis.

| Grandeza | Esperado |
|---|---:|
| Arquivos elegíveis S0 / S1 | 1000 / 1000 |
| Bytes elegíveis S0 / S1 | 4096000 / 4096000 |
| Compare unchanged / modified / removed / added | 985 / 10 / 5 / 5 |
| Demais contadores compare | 0 |
| Unidades D0 candidatas / resolved_text_units | 1005 / 1005 |
| C / M, fração não reduzida | 660 / 128000 |
| Decimal | 0.005156 |
| Bytes de objetos lidos / reservados | 8192000 / 8194000 |
| Células reservadas | 40960 |

Todos os arquivos têm 64 linhas únicas em sua sequência. Os dez modificados têm LCS=63; os demais unchanged têm LCS=64. As cinco adições e cinco remoções têm LCS=0, massa 64 cada. Portanto C=10×2+10×64=660 e M=64000+64000=128000. Somente os dez pares diferentes e bilaterais reservam 64×64 células. Pacotes baseline/current são raízes distintas, sem compartilhamento/hardlinks; cada revisão necessária é lida e reserva 4097 bytes. D0 é complete/exit 0, sem unidades bloqueadas ou transições/non_text.

### L — trabalho de LCS admitido

Acervo separado: um arquivo `p.txt` com 2.000 linhas ASCII `f"line-{j:04d}\n"`, j=0..1999, 20.000 bytes. S1 substitui somente a linha j=1000 por `edit-1000\n`, mantendo tamanho. Pacotes em raízes distintas.

| Grandeza | Esperado |
|---|---:|
| Compare modified / demais contadores | 1 / 0 |
| n0 / nt / l / a / r / c / m | 2000 / 2000 / 1999 / 1 / 1 / 2 / 4000 |
| Valor / decimal | 2/4000 / 0.000500 |
| Bytes lidos / reservados | 40000 / 40002 |
| Células reservadas | 4000000 |

A LCS é 1999: todas as outras linhas são únicas, permanecem na mesma ordem, e a linha substituta não ocorre em S0. Essa prova não depende de executar Rust. D0 é complete/exit 0 e cabe nos limites default.

### Controles negativos na carga L

Aplicar um por vez em cópias/restaurações controladas, sem contaminar os pacotes normais:

| Intervenção | Resultado obrigatório |
|---|---|
| Omitir somente `--baseline-source` | Relatório unavailable/exit 1; baseline source_missing, current verified; sem cálculo/integral/conhecido. |
| Retirar objeto baseline do pacote vinculado | Mesmo bloqueio, motivo local object_missing. |
| Adulterar objeto baseline sem mudar seu tamanho | Mesmo bloqueio, hash_mismatch; size_check=true e hash_check=false após EOF; current verified. |
| Alterar somente snapshot_file_sha256 do descritor baseline para 64 zeros | Envelope incompatible_inputs/exit 2, message=code; não é relatório de indisponibilidade. |

No primeiro controle, baseline fica not_attempted; nos dois de objeto fica failed. Nas três indisponibilidades, unidade sem calculation e measurement.value/line_mass/zero_basis/known_subset=null, motivos globais no_text_units/unresolved_units. Não aceitar erro genérico, timeout ou zero como rejeição correta. Restaurar os inputs e comprovar novamente o controle válido.

## Medição de recursos

Medir **três invocações D0 válidas de M e três de L**, sem compilação concorrente. Guardar cada amostra bruta e seu exit/stdout SHA-256; dentro de cada carga, stdout deve repetir byte a byte. Um relatório representativo por carga basta, referenciado pelas três amostras.

Usar `/usr/bin/time` como processo nativo que lança a CLI, por exemplo `-q -f "%U %S %M %e %x" -o <temporário>`, para CPU user/system, max RSS em KiB, elapsed e exit. Medir wall também com perf_counter_ns, declarando que inclui o lançamento do wrapper. Não usar RSS do Python/orquestrador, nem ru_maxrss cumulativo de vários filhos, como RSS individual do sensor. Preservar a saída bruta do medidor e suas unidades/resolução; CPU arredondada a zero é possível e não deve ser fabricada.

Registrar ambiente suficiente: kernel/plataforma/arquitetura, CPU/quantidade lógica, RAM disponível via informação local, Python, comando/versão disponível do GNU time, limites efetivos, receita/digest do corpus, build/binário e cache **não controlado**. Não limpar caches, mudar governor, exigir root, acessar rede ou alegar cold-cache. Medição ausente/inválida é NAO_VERIFICADO, sem substituição pelo ledger.

De cada relatório conferir os recursos lógicos contra as fórmulas S/Wb/W e as reservas, mantendo-os em coluna/campo separado do RSS. Registrar também tamanho lógico de manifestos/descritores/pacotes/relatórios e alocação física quando o SO a disponibilizar, com exclusões/unidades explícitas. Não incluir compiler/cache de dependências no custo dos pacotes.

**As medidas são descritivas.** Não existe limiar aprovado de latência, RSS, superioridade ou escala; não inventar SLO. Guard timeout de 60 s por subprocesso é proteção do laboratório, não deadline do produto nem critério de desempenho. Timeout/falha deve interromper o aceite e ser relatado, não descartado como amostra ruim.

## Consolidação de evidências e condição de aceite

R1 pode herdar provas já auditadas quando conferir o mesmo SHA do binário, os 37 inputs e os hashes dos artefatos. Os 64 testes Rust, 23 Foundation, 253 verificações V1, 14 mutações (3 Foundation + 11 D0) e a barreira pública BUG-004 foram conferidos em C01. Não declarar que foram reexecutados em R1 se apenas herdados. Não é necessário repetir suites inteiras sem alteração ou lacuna concreta.

Criar matriz R1 separada para E01–E55, usando `verification/d0/coverage.json` como referência histórica intacta. Associar IDs a cenários/testes/evidências reais, seus hashes e versão: `EXECUTADO_R1`, `HERDADO_VERIFICADO` ou `NAO_VERIFICADO`. Reconciliar explicitamente E35/E43/E46/E49 e BUG-004: checkpoints/corridas/aritmética já auditados podem ser herdados; RSS deve ter a prova nova de R1. Referenciar testes Rust permanentes e parecer auditado; temporários antigos não são a única sustentação.

As evidências novas preservam manifestos e descritores brutos em base64 com SHA-256 em `inputs.json`; stdout de cada cenário representativo, em base64 com SHA-256, em `reports.json`. Guardar os bytes completos permite conferir JCS/binding depois da limpeza dos temporários. Conteúdo dos objetos não precisa entrar no repositório: é sintético e regenerável pela receita congelada. Não guardar dados de acervos reais.

R1 só é entregue para aceite se: integração e negativos passam, as seis amostras RSS são válidas, matriz não mascara pendências obrigatórias, baseline permanece intacta e a proveniência coincide. O executor termina IMPLEMENTADO_PENDENTE_VERIFICACAO; a coordenação/auditoria posterior confere os artefatos antes de declarar R1 concluída. HIP-I1-01 permanece observação contratual, sem implementação ou obrigação nova. Falha nova recebe diagnóstico/achado rastreável; não se corrige produção nesta ordem.
