# OS-D0-I1 — implementar o sensor D0 em Rust e na CLI

```yaml
id: OS-D0-I1
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: critico
origem: plano D0; contrato C2-A1 ativado; V1 corrigida e auditada CONFORME
objetivo: executar compass d0 sobre manifestos e pacotes explícitos, com cálculo exato e disponibilidade verificável
dependencias: P1–P4 aprovadas; V1 aceita; auditoria desta OS antes da execução
execucao: sequencial; um escritor; GPT-6.1 Sol / Alto recomendado
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.
Liberação pela coordenação após [auditoria de contrato CONFORME](../../../research/audits/AUDITORIA-CONTRATO-OS-D0-I1.md), sem achado obrigatório aberto. Esta liberação altera somente o estado da ordem, sem mudar escopo, comportamento ou aceites auditados.
Esta ordem autoriza a implementação completa I1-a → I1-b → I1-c, sem novas aprovações entre os marcos. R1 continua sendo o aceite independente posterior. A leitura de conteúdo externo e a contenção de caminhos justificam o porte crítico e a auditoria prévia do contrato; não há nova decisão de produto.

## Baseline e referências

Branch `main`, **sem commits/HEAD**. Inspecionar Git, diff, arquivos não rastreados e hashes antes de escrever. Salvar inventário/cópias da árvore inicial em `/tmp`; registrar no resultado os hashes relevantes antes/depois e as alterações autorizadas. Não usar `git reset`, `clean` ou atribuir resultados a commit inexistente.

| Referência congelada | SHA-256 |
|---|---|
| `research/proposals/d0-c2/SHA256SUMS`, 81 entradas | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| `research/proposals/d0-c2/CONTRATO_CANDIDATO.md` | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| `research/audits/AUDITORIA-OS-D0-V1-C01-C03.md` | `71e21f87fcdeb9d2e51bdca42d0e5beac3209396917e9a9520101c36ef3711c1` |
| `verification/d0/RESULTADO-D0-V1.md` | `4e04373959ebec7c1237cd888bb9aa67ab1038847fc5a97e5b5d6a01cea2c186` |

Foundation verificada antes de I1: `producer_build=sha256:94866a9d405c283d1913c13e53ff0d5413823f3ce1a654e8bbc68619d7ddd4b0`. Esse identificador não será reutilizado para o novo build. Política D0 default: `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Ler, nesta ordem:

1. `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md) e esta OS.
2. `research/contracts/FOUNDATION.md`; contrato C2-A1 completo, seus cinco schemas e `fixtures/metric-policy.json`. A ativação prevalece sobre os rótulos históricos de candidato inativo.
3. `verification/d0/ACCEPTANCE.md`, `coverage.json`, `fixtures.json`, resultado V1 e auditoria das correções. Usar casos E01–E55 e relatórios congelados pertinentes aos testes; não reproduzir a lógica Python como implementação.
4. Rust existente: `model.rs`, `codec.rs`, `compare.rs`, `observe.rs`, `storage.rs`, `lib.rs`, `build.rs`, CLI e testes. Reaproveitar APIs públicas puras de validação/comparação/JCS; preservar sua semântica.

O contrato e os schemas ativados são a autoridade de significado; esta OS delimita implementação, testes e integração. Contradição comprovada deve ser reportada, sem o executor modificar critérios ou escolher outro significado.

## Escrita autorizada — lista fechada

Existentes, apenas para integrar D0:

- `crates/compass-core/src/lib.rs`: exportar o novo módulo.
- `crates/compass-core/build.rs`: incluir explicitamente os inputs normativos D0 descritos abaixo na identidade do build.
- `crates/compass-cli/src/main.rs`: comando D0, encaminhamento ao núcleo, erros/saída/sinal próprios; preservar comandos anteriores.
- `README.md`: uso real do comando, pacotes externos, indisponibilidade e limites comprovados.

Novos; criar apenas os necessários:

- `crates/compass-core/src/d0/mod.rs`: fronteira pública pequena e orquestração.
- `crates/compass-core/src/d0/types.rs`: transporte, política e erros D0.
- `crates/compass-core/src/d0/input.rs`: leitura delimitada, preflight e binding dos inputs.
- `crates/compass-core/src/d0/source.rs`: raízes/handles, objetos, estabilidade e evidência de leitura.
- `crates/compass-core/src/d0/resources.rs`: admissões, reservas, deadline e checkpoints.
- `crates/compass-core/src/d0/metric.rs`: universo, estados efetivos, normalização, LCS e aritmética.
- `crates/compass-core/src/d0/report.rs`: disponibilidade, agregado, relatório e validações cruzadas.
- `crates/compass-core/tests/d0.rs`: testes de contrato do núcleo.
- `crates/compass-cli/tests/d0.rs`: testes da interface pública.
- `verification/d0-i1/run-v1.py`: adaptação de ambiente para executar a V1 **intacta**, conforme seção seguinte; não é outro oráculo.
- `verification/d0-i1/baseline-foundation.tar` e `baseline.json`: arquivo de baseline e inventário verificável, preparados antes de alterar os quatro arquivos existentes acima.
- `verification/d0-i1/acceptance.json` e `mutations.json`: cópias exatas dos resultados produzidos pelo laboratório contra o novo binário.
- `verification/d0-i1/RESULTADO-D0-I1.md`: retorno durável, ACs, comandos, hashes e pendências.

Testes internos e hooks privados `#[cfg(test)]` podem ficar nos novos módulos autorizados. Não adicionar flags públicas de teste. Artefatos Cargo em `target/` e temporários em `/tmp` são saídas técnicas permitidas. Não instalar dependências; Cargo.toml, Cargo.lock e toolchain ficam preservados. Necessidade demonstrada de outro arquivo/dependência exige `BLOQUEIO_DE_FRONTEIRA`, com motivo concreto.

**Protegidos:** todos os demais arquivos, em especial `verification/d0/` inteira, contratos/schemas/fixtures C2, auditorias anteriores, documentos de ativação/OS/plano/coordenação/estado, arquivos originais do documento principal, verificadores e evidências Foundation. Não alterar `compare.rs`, `model.rs`, `codec.rs`, `observe.rs`, `storage.rs` ou o erro global Foundation para acomodar D0. O núcleo D0 pode reproduzir localmente a resolução de estados efetivos conforme contrato, com testes de equivalência; isso evita refatorar a Foundation nesta entrega.

Os 34 hashes de `evidence/preservacao.json` são a preservação **histórica de C2/V1**. I1 altera legitimamente somente quatro desses arquivos: `lib.rs`, `build.rs`, CLI e README. Os outros 30 continuam idênticos; os quatro recebem inventário antes/depois e regressão comportamental. Nunca regravar o inventário histórico para fazê-lo coincidir com I1.

## Executar V1 sem invalidar sua baseline histórica

`verification/d0/validate.py:132–149` confere também os 34 fontes/documentos anteriores. Portanto executar o laboratório diretamente na árvore I1 mudada produziria `protected_hash`, mesmo com sensor correto. A solução desta OS é preservar o laboratório e separar a árvore histórica de teste da origem do binário.

`run-v1.py --prepare-baseline`, **antes das edições Rust/README**, deve conferir os 34 hashes e os 81 artefatos e arquivar exatamente os 34 arquivos em `baseline-foundation.tar`. Usar membros relativos sob `compass-foundation/` e, para o único documento externo listado, sob `compass-documento-principal/`; nenhum membro absoluto, `..`, link ou tipo especial. `baseline.json` registra SHA-256 do arquivo tar, membros/hashes, hashes dos arquivos V1 existentes e desta OS. Não incluir acervos, credenciais, `target/` ou `.git`. Se o arquivo já existir, conferir e reutilizar; nunca substituir silenciosamente uma baseline divergente.

`run-v1.py --binary <binário> [--cached-dependencies <cache>]` deve:

1. Conferir baseline e membros contra o inventário histórico, a integridade dos originais congelados e a identidade dos fontes V1 aceitos. Rejeitar membro extra/proibido ou hash divergente antes de montar o ambiente; copiar conteúdo validado sem extração arbitrária de tar.
2. Montar árvore temporária em `/tmp` com os 34 arquivos históricos, cópias exatas de `verification/d0/` sem resultados/cache, C2/parecer C2, contrato Foundation e auditoria C2-A1 nas posições relativas esperadas. Os caminhos do pacote/harness permanecem temporários.
3. Invocar os dois comandos V1 `--binary` dessa cópia, com **caminho absoluto do novo binário da árvore I1 real**, sem compilação concorrente, monkeypatch, filtro de falhas ou alteração de esperado. O wrapper não interpreta a matemática nem transforma falha em sucesso; propaga falha de qualquer comando.
4. Registrar o hash do binário antes/depois; copiar byte a byte os resultados gerados para os dois JSONs autorizados em `verification/d0-i1/`. Informar nos logs/retorno a origem do laboratório, a árvore I1 efetiva e os códigos. O produtor/build real precisa ser conferido separadamente contra os fontes I1.

Testar o adaptador, incluindo adulteração de membro/fixture em cópia, preservação dos fontes V1 e propagação de retorno não zero. Não criar um sensor simulado para declarar campanha real aprovada. A baseline arquivada permite repetir a campanha em R1 sem exigir que os fontes I1 voltem a ser Foundation. Passar os hashes históricos dentro desse ambiente **não prova** preservação da árvore I1: a conferência dos 30 intactos e quatro deltas autorizados continua separada.

## Implementação sequencial

**I1-a — inputs, fonte e recursos.** Criar tipos/erro D0 separados do erro Foundation. Leitura delimitada de arquivos regulares e JSON estrito; profundidade/contagem/S_manifest antes de construir/validar o domínio. Validar baseline/current e compatibilidade antes dos descritores. Parse e SHA-256 bruto usam o mesmo buffer; aceitar serialização JSON não canônica válida. Não usar `storage::load_snapshot` sem teto como leitor D0.

Fixar raízes/handles e resolver componentes por `openat2` com as restrições contratadas, inclusive referências dos manifestos/descritores e a raiz do pacote, sem fallback `stat`+`open`. Tipo antes de leitura, NONBLOCK/NOCTTY, nome↔handle e metadados antes/depois; sem retries ou cache entre unidades. Link intermediário pendente é `source_invalid_kind`, não ausência regular. Descritor presente deve ser validado mesmo com universo vazio. Omissão de fonte, pacote inválido, binding divergente e falha local de objeto mantêm seus diferentes motivos/envelopes/exits.

Implementar S/Wb/W, reservas irreversíveis `s+1`, EOF/prefixo, chunks ≤65536, bytes lidos distintos de reservados e compartilhamento restrito à mesma raiz fixada/descritor/objeto na unidade atual. Recusa sem débito permite candidato menor posterior. Memória é ledger lógico, sem promessa de RSS/OOM.

**I1-b — sensor e relatório.** União de observed e changes unknown, incluindo unchanged, ordenada por bytes nativos; estado efetivo separado de entrada exata, unknown anterior a exclusões/transições. Conferir ambos os lados admitidos antes de non_text; UTF-8 antes de NUL. Normalizar somente após bytes verificados; aplicar integralmente §§3–6 do contrato.

Depois de fonte/texto/linhas/memória admitidos, igualdade **exata das sequências normalizadas** obrigatoriamente custa zero células, inclusive E36/E51–E55; mesma contagem não prova igualdade. Demais pares usam LCS exata e reserva `n0*nt`, com memória linear compatível com W. Aritmética checada u128, saída u64 string, frações não reduzidas, seis casas com empate para par. Agregar unchanged/vazios, distinguir integral e conhecido, null e zero conforme §5. Validar invariantes de inputs/política/cálculo/relatório no núcleo; schema sozinho e o verificador Python não substituem essas validações.

**I1-c — CLI, proveniência e uso.** Implementar exatamente:

```text
compass d0 --baseline <json> --current <json>
  [--baseline-source <dir>] [--current-source <dir>]
  [--limits <limits.json>] [--deadline-ms <1..3600000>] [--json]
```

`--limits` é perfil estático completo de reduções positivas; prazo somente pela flag. D0 JSON é um objeto JCS **sem newline**, inclusive envelope com `message=code`. Preservar saída/erros dos comandos Foundation. Tratar também erros de argumentos D0, antes do dispatch normal, com os envelopes/exits contratados. Saída humana distingue integral/conhecido/indisponível, sem expor conteúdo. Não acrescentar pacotes automáticos, retenção, captura de acervos, hunks, GUI, grafo, busca ou MCP.

Prazo monotônico começa após argumentos válidos e inclui preparação/saída; SIGINT vence deadline e erro no checkpoint. Verificações entre leituras/chunks/unidades e a cada ≤4096 operações LCS. Preparar/limitar saída antes de stdout; aborto não publica relatório final nem subconjunto. Falha de pipe não recebe promessa de atomicidade. Não mudar a semântica cooperativa para alegar prazo rígido.

`producer_version/build` identificam o emissor real, sem strings de fixture. Estender em `build.rs` a coleta existente com **sete arquivos explícitos**, todos sob `research/proposals/d0-c2/`: `CONTRATO_CANDIDATO.md`, `schemas/report.schema.json`, `schemas/source.schema.json`, `schemas/metric-policy.schema.json`, `schemas/limits.schema.json`, `schemas/error.schema.json` e `fixtures/metric-policy.json`. Manter domínio `compass-build-v1\0`, ordenação e codificação de caminhos/bytes existentes; nenhum outro arquivo de propostas/evidências entra implicitamente. Essa extensão inclui o contrato D0 ativo, que está fora de `research/contracts/`. Recomputar independentemente a identidade com a lista efetiva completa e registrar SHA-256 do executável após a última mudança de input do build.

## Aceites

| ID | Resultado observável e prova mínima |
|---|---|
| I1-AC-01 | Inventário inicial/final comprova somente escritas autorizadas; 81 artefatos, V1 e 30 arquivos históricos não liberados ficam idênticos. Quatro arquivos liberados têm deltas pertinentes. Adaptador preserva a baseline, rejeita adulteração e propaga falhas; não oculta uma falha do sensor. |
| I1-AC-02 | Inputs JSON estritos não canônicos passam com binding bruto correto; duplicatas/BOM/trailing/UTF-8 inválido, profundidade/registros/tetos, incompatibilidades e perfil inválido recebem envelope/exit exatos. Hook privado comprova recusa de S_manifest antes do domínio. Fonte presente inválida falha também no universo vazio. |
| I1-AC-03 | Testes positivos/negativos demonstram contenção sem leitura de alvo externo, links de raiz/intermediário/objeto, tipos especiais, ausência regular e pacote omitido/inválido. Barreiras determinísticas trocam objeto, nome e componente após abertura; instabilidade detectada bloqueia cálculo, inclusive hash coincidente. Handles/metadados/nome não são substituídos por lstat estático. |
| I1-AC-04 | Hash/tamanho vêm dos mesmos bytes, inclusive vazios; EOF curto e excesso s+1 produzem flags/revisões corretas. Reserva não é devolvida em missing/erro; leitura real compartilhada só no caso autorizado; raízes distintas/hardlinks/mesmo hash e unidade posterior não emprestam prova. |
| I1-AC-05 | E01–E55 aplicáveis ao núcleo/CLI, sobretudo E36/E51–E55, reproduzem literais independentes. Incluir controle de mesma contagem diferente, fontes ausentes/adulteradas, CRLF/LF brutos distintos, BOM, UTF-8/NUL, unchanged no agregado, unknown ancestral e exclusões. Não gerar esperado de Rust. |
| I1-AC-06 | Testes de fronteira comprovam ordem de admissões, reservas byte/célula e peak S/Wb/W, sem débito na recusa; candidato menor posterior passa. Igualdade verificada preserva saldo zero; falha de fonte não vira zero. Casos diretos de aritmética cobrem u64::MAX, excesso e todos os empates/extremos E49 sem precisar montar arquivos enormes. |
| I1-AC-07 | Relatórios/exits/status/reasons/subconjunto/zeros e ordem nativa satisfazem os cinco schemas e as validações semânticas. Native paths de aparência numérica passam, inclusive regressões BUG-001/002/003. Nenhum conteúdo/hunk/raiz nova/timestamp de execução entra no relatório. |
| I1-AC-08 | `--help`, argumentos, limits/deadline, modo humano e JSON sem newline funcionam na interface pública. Testes sincronizados provam SIGINT após uma unidade/antes da seguinte, prioridade SIGINT>deadline>erro, checkpoints em leitura/LCS e aborto antes da publicação. Guard timeout e smoke probabilístico não substituem essas provas. |
| I1-AC-09 | Campanhas V1 acceptance e mutations executam contra o release D0 estável pelo adaptador, com esperados intactos, saídas reais e hash estável. Cada negativa atinge sua asserção específica. `runtime_pending` remanescente no JSON V1 é reconciliado por IDs dos testes Rust, sem editar o JSON gerado para alegar cobertura. |
| I1-AC-10 | Testes Rust workspace, regressão CLI Foundation, três mutações Foundation, fmt, Clippy e release passam na mesma árvore final; falha/skip/permissão não é aprovação. Nova identidade de build é recomputada e coincide com o produtor emitido e o executável testado. |
| I1-AC-11 | README e resultado distinguem implementação testada, auditoria/aceite R1 ainda pendentes, ledger lógico e RSS não medido. Mapear cada prova transferida pela V1 ao teste executado ou à limitação real. Não declarar homologação completa apenas por self-test/build. |

Cada teste negativo deve ter controle válido e atingir motivo/asserção específica. Corridas usam barreiras/hooks de teste; não aceitar repetição probabilística como prova. Testar conteúdo e permissões apenas em árvores artificiais autorizadas em `/tmp`.

## Comandos e registro

Os comandos Cargo/Foundation/V1 já existem. O adaptador abaixo é **novo, a entregar nesta OS**, não uma ferramenta já disponível. Na raiz do repositório:

```sh
# Depois de criar o adaptador e antes de mudar Rust/README:
python3 verification/d0-i1/run-v1.py --prepare-baseline

# Focar os testes dos módulos durante o trabalho; fechamento na árvore final:
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
python3 verification/reference/acceptance.py --binary target/release/compass
python3 verification/reference/mutations.py --binary target/release/compass
python3 verification/d0-i1/run-v1.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

O adaptador executa, na cópia, `acceptance.py --binary <absoluto>` e `mutations.py --binary <absoluto>` de `verification/d0/`, repassando o cache quando informado. Não compilar enquanto testa o artefato estável. Formatador em escrita apenas nos novos fontes e nos quatro arquivos autorizados; não aplicar formatação em massa. Após qualquer alteração posterior de input do build, renovar build/hash e verificações afetadas antes do registro final.

Sockets Unix locais podem exigir permissão técnica fora do sandbox. Solicitar essa permissão pelo mecanismo da ferramenta ou registrar a restrição; não é operação de rede nem autorização para acervos reais. Comandos indisponíveis, falhas e skips devem constar explicitamente. A campanha não passa com zero cenários executados.

Retorno em `verification/d0-i1/RESULTADO-D0-I1.md`: estado `IMPLEMENTADA_PENDENTE_AUDITORIA` somente com ACs demonstrados; tabela AC→teste/comando/evidência, diff/hashes, lista real de testes/cenários/falhas, producer_build/binário, origem e resultados do laboratório, restrições e provas de R1. Não alterar documentos centrais: a coordenação confere e atualiza o estado.

## Bloqueios, reversão e sequência

Contrato/esperado legitimamente divergente, arquivo fora da lista ou regressão não resolvida deve ser reportado com AC e reprodução, sem enfraquecer teste nem expandir a ordem. Duas falhas da mesma abordagem exigem reinvestigação antes de terceiro patch. P1–P4 não são bloqueio de produto aberto.

Reversão limita-se aos novos arquivos e hunks desta OS, conferidos contra cópias iniciais; preservar alterações anteriores, baseline e evidências úteis. Nenhum deploy, migration, envio a outro chat, acesso remoto ou coleta de acervos está autorizado.

Após o retorno: auditoria independente da I1 e, conforme o plano, R1 para integração/aceite e medição de recursos reais. Não iniciar R1 automaticamente por concluir esta OS.
