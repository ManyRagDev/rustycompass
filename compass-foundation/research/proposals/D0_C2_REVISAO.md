# D0-C2 — Parecer da conferência independente

14 de outubro de 2026. Skill aplicada: `codex-arquiteto`. **Parecer: pendente para ativação**, por decisões de produto P1–P4 e liberação da coordenação. A revisão técnica C2 está concluída, com candidato/transportes/exemplos verificáveis; não é homologação de sensor implementado. Não há aprovação atribuída a Emanuel.

## 1. Resultado esperado e escopo

Revisar C1 independentemente, resolver lacunas técnicas, preparar material congelável para a coordenação. Mudança futura multicamadas (fonte → núcleo → relatório/CLI), mas entrega atual exclusivamente documental. Artefatos em [d0-c2/INDEX.md](d0-c2/INDEX.md), candidato em [CONTRATO_CANDIDATO.md](d0-c2/CONTRATO_CANDIDATO.md), evidência em [conferencias.json](d0-c2/evidence/conferencias.json). Proposta C1 preservada; Foundation ativa e gabaritos preservados. Nenhum sensor, adaptador, retenção, CLI ou V1/I1 foi iniciado.

A revisão parte dos inputs/esperados literais de C1 e das regras atuais do repositório; nenhum resultado de produção D0 existe ou foi usado. A evidência de C2 recalcula LCS por **enumeração de subsequências**, diferente da DP recomendada para futura produção. O checker desta pasta é uma conferência pequena de documentos/fixtures, não um laboratório D0 completo.

## 2. Evidência relevante inspecionada

| Arquivo/símbolo | Observação conferida |
|---|---|
| `research/proposals/D0_V0.1.md`, §§4–5 | E01–E35 e fórmulas são propostas. Memória/CLI/saldo/transportes ainda remetiam ao futuro. |
| `research/contracts/FOUNDATION.md`, §§2–8, 12 | Hash bruto, exclusões/cobertura, ausência comprovada, snapshots sem conteúdo. Nenhuma alteração autorizada nesta revisão. |
| `model.rs`, `Snapshot`, `Snapshot::validate` | Validação de árvore, política, digest/counters/versões; não valida bytes externos. |
| `compare.rs`, `View::state` (linhas 83–121) | Unknown ancestral primeiro; ausência por primeiro componente faltante; excluded ancestral pode representar descendente sem entry exata. |
| `compare.rs`, `compare` (linhas 123–245) | União de todas entries para comparação; unchanged contado/omitido de changes; unknown_regions preservado mesmo sem arquivos. Classificação não se reconstrói só de before/after. |
| `storage.rs`, `snapshot`, `load_snapshot` | Persistência de manifesto, sem blobs; leitor Vec irrestrito precisa de leitor D0 delimitado próprio futuro. |
| `codec.rs`, `decode`, `canonical` | Rejeição de duplicados, parse não exige wire JCS; saída JCS já existe. |
| `observe.rs`, `open_beneath`, `unchanged` | Handles/openat2 sem fallback, checks de identidade/metadados; orientação para adaptador, sem empréstimo de garantia de snapshot atômico. |
| `compass-cli/src/main.rs`, `Command`, `run`, `main` | Três comandos existentes; códigos e SIGINT cooperativo. d0/limits/deadline não existem. |
| `build.rs` | 21 inputs de código/contratos ativos, propostas fora do build. Identidade recomputada corresponde à evidência Foundation, sem build novo. |
| Documento principal preservado cap.7, §9.2, cap.17 | Indisponível não é zero, A→B→A usa extremos; pesos log são históricos; ativação D0 precede consumidores. |

## 3. Achados, gravidade, decisão e resolução

Gravidade alta = afeta integridade/significado/disponibilidade ou deixa consumidor adivinhar; média = transporte/operabilidade/evidência. Não há achado que justifique alterar Foundation.

| ID | Gravidade/evidência ou contraexemplo | Decisão e resolução candidata |
|---|---|---|
| T01 | Alta — `comparison.changes` não contém unchanged (E07); usar só changes dá D0=1, em vez de 0.1 | União de observed + changes unknown; incluir unchanged, sem inventar arquivos em unknown_regions. Candidato §§3–5. |
| T02 | Alta — arquivo observed sob ancestral unstable não é conhecido; excluded ancestral pode não estar em before/after | Estados efetivos seguem View::state, com observation separado de Entry exata. Unknown precede scope/bytes; todas exclusões preservadas sem leitura. E26/E27/E40. |
| T03 | Alta — zero, empty_universe, zero_text_mass, nenhum texto e região desconhecida tinham fronteiras/listas de motivos incompletas | Fórmula cumulativa de motivos e tabela de status/exit. Empty-universe known_subset=null; vazio textual known_subset=0/1; partial nunca integral. E28/E31 têm no_text_units+unresolved_units. E38/E39. |
| T04 | Alta — hash de reserialização não identifica manifesto entregue; JCS do descritor não tinha política de aceitação explícita | Aceitar serializações JSON estritas não canônicas; hash bruto dos inputs, source_ref bruto, policy_id sobre JCS. Duplicados/BOM/trailing data rejeitados. Whitespace de manifesto exige rebinding; do descriptor só muda source_ref. E42, vetores fixos. |
| T05 | Alta — fonte compartilhada por source_ref/hash somente poderia esconder objeto adulterado em outra raiz | Uma leitura somente mesma raiz fixada, mesmo descriptor/binding e objeto; raízes distintas têm checks físicos separados. Pacote abc-adulterated tem source_ref igual ao abc, mas falha local com same-size. E29/E44 e fixtures completas. |
| T06 | Alta — links/FIFO/substituições e ausência de ferramenta de captura não estavam fechados no transporte | Raízes fixadas/openat2/regular/NONBLOCK/checks de nome+handle; mapping local preciso; pacote presente inválido é erro, objeto faltante local. Preparação manual reprodutível com inputs literais, sem inventar captura. Corridas/aberturas serão testadas em V1/I1, não alegadas testadas aqui. |
| T07 | Alta — memória remetida ao I1; “128 MiB” sem capacidades/overhead definidos não era verificável | Substituir promessa por ledger **lógico** versionado com S_manifest/S/Wb/W e peak explícitos; contar issues no overhead; preflight antes de domínio, descriptors incrementalmente. Sem garantia de RSS/alocador/OOM. Fórmulas e fronteiras conferidas. |
| T08 | Alta — EOF/byte budget e custo de falha ausente indefinidos; E30 dependia de leitura ilimitada | Reserva conservadora s+1 sem devolução; leitura ≤s+1 e hash somente EOF. Contadores reserved/read separados. **E30 corrigido**: prefixo abcd com s=3 dá size_mismatch, hash_check=null, actual_revision=null; não prova EOF. Isto é mudança deliberada do candidato, não ajuste ao código. E45/E47. |
| T09 | Alta — saldo insuficiente de células “continuar se regra permitir”; regra anterior tratava equal 4000×4000 como indisponível apesar de LCS exata conhecida | **D0-C2-A1:** após fonte, texto, normalização, limite de linhas e memória, igualdade exata das sequências normalizadas obriga `l=n0=nt` e reserva 0; hash/tamanho/contagem não bastam. Diferentes continuam `n0*nt`, per-unit → saldo, sem débito na recusa. E33 permanece; E36/E51–E55 cobrem igualdade sobre teto, saldo zero, normalização, fontes bloqueadas, mesma contagem diferente e saldo seguinte. |
| T10 | Alta — simetria matemática confundível com simetria de subconjuntos/bytes | Fórmula completa simétrica; r/a trocam. Reserva baseline-first sem devolução pode deixar saldos distintos em pares inadmissíveis. E45: tamanhos 2/0 sob teto 3 deixam saldo 0 vs 2; próximo q vazio bilateral torna reverse partial e forward unavailable. Não se promete simetria geral sob recursos. |
| T11 | Média — assinatura sem limits/prazo; configuração/default/exit/policy_hash incompletos | --limits objeto completo de reduções e --deadline-ms; limites default fechados, deadline no metric_policy, tipos/enums/null/absent e saída/error/exit definidos. Sem flags ocultas. Comandos Foundation preservados. |
| T12 | Alta — schema não consegue provar matemática, binding, segurança do FS ou ordenação nativa | Separar transportes e invariantes do núcleo. 5 schemas fechados; 6 negativos rejeitados; 1 transporte válido com massa errada é rejeitado por conferência semântica. Ordem nativa usa contraexemplo 0xfb/base64 '-w'. |
| T13 | Média — invalid_utf8 e NUL coexistem em E20; número exibido pode arredondar para zero | UTF8 estrito precede NUL; subtipos por lado, ambos bytes verificados antes de non_text. Half-even inteiro congelado; conferir Decimal independente e razão positiva com display zero. E20/E23/E49. |
| T14 | Alta — pesos lineares alteram o significado em relação à referência histórica | Contas locais/line-mass corretas; explicitar P2. E07 line-mass=0.100000; log=0.231378. Não alegar superioridade científica nem ativar por conveniência aritmética. |

Decisões técnicas T01–T13 estão resolvidas no candidato. T14 tem a comparação matemática resolvida e a escolha de produto ainda aberta. Recursos/canonical vectors não constituem medição de performance nem validação geral de RFC8785.

## 4. Conferência E01–E35 e fronteiras novas

| Exemplos | Resultado independente/resolução |
|---|---|
| E01–E06 | Conferidos l/r/a/c/m e razões 0/20,2/20,1/21,1/19,2/2,2/2. Sem correção matemática. |
| E07 | Numerador 2, massa20 incluindo unchanged; 0.1. Peso histórico ≈0.231378. Escolha P2. |
| E08 | AB=2/4, AA=0/4; extremos, sem somar comparações. |
| E09 | Removed+added,4/4; sem inferir rename por hash. |
| E10–E13 | Zero textual auditado 0/1/massa0; criação/remoção vazio ainda Foundation changed; acervo vazio convencionado; vazio+substituição2/2. Empty-universe subset fechado como null. |
| E14 | LCS ordenada e repetição:2/4 e1/3, não interseção de multiset. |
| E15–E19 | Normalização e ausência de trim/NFC/CR isolado conferidas por bytes hex, hashes e linhas literais. Identidade bruta preservada. |
| E20–E23 | Binário/UTF16 fora da métrica, mixture completa no universo textual. E20 especificado como invalid_utf8 primeiro; texto→NUL é nul. Ambos lados precisam de checks, falha não se oculta com non_text. |
| E24 | Metadata 0/2; duas unidades/hardlinks0/4; nomes/caminhos não fundidos, ordem nativa explicitada. |
| E25 | Scope-only indisponível/no_text_units; arquivo↔diretório é added/removed com descendants, não link sintético ausente. |
| E26–E27 | Known subset1/3 ou0/20; value integral=null, ancestral desconhecido e fronteira sem inventário preservados. |
| E28 | Falta baseline bloqueia p; outro E03 dá subset1/21; motivos globais agora cumulativos. |
| E29 | abc/abd hashes distintos, mesmo tamanho3: size true/hash false; pacote literal adulterado confirma. |
| E30 | **Esperado alterado com justificativa T08**: na reserva s+1, abcd atinge cap4 sem EOF; só size_mismatch comprovado, hash null. Menor tamanho com EOF admite ambos checks. |
| E31 | Hash/digest iguais e fonte omitida continuam indisponíveis; vazio sem objeto também, jamais shortcut zero. |
| E32 | 4194305 excede file cap; 4000²=16000000 excede cells cap para sequências **diferentes**. Sem leitura/diff aproximado. |
| E33 | p reserva4 e ratio2/4; q sem saldo bloqueia. Reserva rejeitada não altera saldo; perfil explícito reduced. |
| E34 | Workspace/policy/binding incompatible=2; manifesto/digest inválido=1. Sem resultado D0. Binding/serialização em fixtures; códigos negativos de operação serão exercitados em V1/I1. |
| E35 | SIGINT130/deadline1, nenhum measure final; regras/checkpoints fechados, sem alegação de experimento de timing nesta entrega. |

E36–E55 acrescentam: igualdade sobre teto com custo 0, saldo menor/custo 0, zero parcial, no-text com região unknown, unknown vs exclusion, binário+fonte faltante, reserialização/duplicados, links/FIFO/substituição, sharing, assimetria de bytes, memória/precedências, reservas sem devolução, segundo BOM, empate decimal, schema vs domínio u64/path, igualdade com saldo 0, normalização com bytes diferentes, fontes que bloqueiam antes da igualdade, mesma contagem diferente e saldo da unidade seguinte. `resource-frontiers.json` congela igualdade/excesso em file/lines/cells/memória/u64. Inputs enormes são dimensões declarativas, não arquivos gigantes ou execuções quadráticas em C2.

## 5. Decisões de produto necessárias para ativação

| ID | Opções concretas/recomendação | Consequência e dependentes |
|---|---|---|
| P1 — fonte/custo do operador | **Recomendar pacotes externos explícitos**, sem retenção automática. Alternativa: captura/retenção integrada em tarefa própria antes de I1 | Operador obtém/preserva revisões antigas, prepara/binda pacotes e controla segredos/disco. Sem bytes não há medida integral. Se rejeitado, replanejar lifecycle/fonte/CLI/fixtures; não improvisar árvore viva. |
| P2 — significado do agregado | **Recomendar massa bilateral linear**, com nome d0_line_mass_v1. Alternativa: peso log histórico, novo contrato/oráculo | Arquivos grandes dominam mais; line-mass é fração auditável de massa dos extremos. E07 muda0.1→0.231378 com log. Afeta valores, versões, policy_hash e consumidores. |
| P3 — universo textual e zeros | **Recomendar UTF8 sem NUL, BOM/CRLF/LF terminal normalizados, binário/transição fora, vazio verificado zero explícito**. Alternativa: outra elegibilidade/normalização ou exigir valor ausente nos vazios | Modified por bytes pode ter D0zero; texto→binário sai da métrica; criação de vazio pesa0. D0 não representa todos bytes nem invalidação. Mudar requer exemplos/schema/zero_basis novos. |
| P4 — rigor de evidência e disponibilidade | **Recomendar verificar unchanged/vazios, integral null sob bloqueio e subconjunto separado**, com perfil conservador explícito. Alternativa: sensor distinto com evidência relaxada (revisão própria) | Mais leitura/fornecimento de bytes e possíveis indisponibilidades. Igualdade exata já normalizada e verificada não reserva células; nenhum zero por hash/atalho. Ledger é admissão lógica, não RSS; tetos são política inicial sem benchmark. Afeta aceite operacional e mensagem ao usuário. |

Estas recomendações não pedem confirmação durante a preparação: os artefatos estão concretos e revisáveis, e a coordenação resolverá as escolhas com Emanuel. O parecer **não pode ser pronto_para_ativacao** enquanto mudanças de significado/ônus operacional não forem aceitas e registradas. Não restou lacuna técnica delegada silenciosamente ao I1.

## 6. Verificações realmente executadas e limites

Comando reproduzível, do diretório compass-foundation:

```sh
python3 research/proposals/d0-c2/evidence/conferir.py --cached-dependencies /home/many/.cache/uv/archive-v0
```

Python 3.12.3 e jsonschema 4.26.0 oficial já presente no cache local, com suas dependências. Nenhum pacote instalado, nenhum módulo/código/teste do legado importado. Sem a flag, o comando funciona onde jsonschema está instalado; caso indisponível, falha explicitamente, sem fingir validação.

Passaram após D0-C2-A1: 44 contas literais de LCS/contagens, 23 agregações, 961 pares exaustivos pequenos para simetria/faixa, A→B→A, 4 fronteiras de arredondamento, 22 lados de vetores byte/normalização/hash, 6 SHA-256 brutos, comparação com peso histórico, vetor policy prefix+JCS/hash, 5 metaschemas Draft2020-12, 3 políticas/perfis, 9 manifestos candidatos (schema/hash/policy/digest), 11 bindings de descritores, 10 relatórios (schema/semântica pequena/bytes canônicos) e 10 conferências dos mesmos buffers/inputs/reservas, 10 envelopes, 6 negativos schema, 1 erro semântico aceito pelo schema e rejeitado pela conferência, 3 classificações non_text, 12 fronteiras de recursos e 2 agendas de saldo. A evidência inclui igualdade exata sobre teto, saldo zero após unidade diferente, normalização com bytes distintos, fontes bloqueadas antes da igualdade, mesma contagem diferente e saldo preservado para unidade seguinte.

`evidence/conferencias.json` registra as contagens reais. Relatórios candidatos têm producer_version/build de fixture, **não build Rust**. Transporte/schema não prova que o futuro leitor implementará openat2/TOCTOU, nem que futuros reports serão matematicamente corretos. Esses testes, deadlines/SIGINT e campanha completa de mutações ficam para V1/I1/R1 após liberação. Canonical bytes desta entrega só cobrem fixtures ASCII/string-only; não há alegação de implementação JCS geral no checker.

Recalculados os 21 inputs conforme build.rs: `sha256:94866a9d405c283d1913c13e53ff0d5413823f3ce1a654e8bbc68619d7ddd4b0`, igual ao release Foundation registrado. Isto é conferência de identidade dos inputs, **não build/teste novo**. Inventário antes/depois de 34 arquivos preexistentes em `evidence/preservacao.json` confirma código, contratos/gabaritos ativos, C1 e documento principal preservados. Somente registros curtos de estado/coordenação mudaram fora dos artefatos C2. Nenhuma suíte Rust/CLI/benchmark/migration executada por esta tarefa documental.

## 7. Plano de ativação e entrega aos consumidores

1. Coordenação lê parecer/candidato/INDEX/SHA256SUMS, resolve P1–P4 e registra versão e hashes aprovados. Se mudar regra, reconciliar candidato/schema/esperados e repetir somente checagens afetadas antes de liberar consumidores.
2. Congelar o conjunto aprovado em destino ativo definido pela coordenação. Não copiar cegamente caminhos relativos de schemas: referências à Foundation devem continuar resolvendo; conferir após mover. Atualizar build inputs quando contrato ativado integrar research/contracts. Esta tarefa não faz essa publicação.
3. V1 receberá contrato/schema e E01–E55/bytes/relatórios/input-map/fronteiras congelados; produzirá laboratório completo em verification separado dos gabaritos Foundation. I1 receberá o mesmo conjunto, binding/ledger/CLI fechados; implementará fonte antes dos consumidores.
4. Só então permitir V1/I1 paralelos em áreas separadas; um escritor Rust. R1 integra regressão/CLI/oráculo/mutações/recursos com build real. A coordenação fornecerá prompts concretos; não são iniciados aqui.

Definição de encerramento C2: parecer e decisões rastreáveis; candidato transportável/verificável; índice/hashes e evidências reproduzíveis; progresso persistido; código/contratos ativos/esperados Foundation intactos. **C2 concluída; ativação e V1/I1 permanecem pendentes.**
