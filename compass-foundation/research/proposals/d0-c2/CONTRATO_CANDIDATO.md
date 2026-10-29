# D0-C2 — Contrato consolidado candidato

14/10/2026. **Candidato inativo**, D0 0.1.0/schema 1/métrica 1. Parecer em `../D0_C2_REVISAO.md`. C1 permanece histórico; este texto e os schemas desta pasta consolidam o candidato. Nada em `research/contracts/` é substituído. Decisões de produto P1–P4 e liberação da coordenação antecedem V1/I1.

## 1. Resultado, evidência e escopo

Sensor `d0_line_mass_v1`: fração de massa bilateral de linhas sem correspondência numa LCS exata entre dois snapshots explícitos. Proxy textual, sem promessa de impacto semântico ou qualidade de contexto; baseline não avança. Rust será produção; C2 apenas prepara contratos, exemplos e conferências documentais. Não há adaptador, sensor, CLI nova, retenção ou laboratório V1 nesta entrega.

Evidências: Foundation §§2–8; `model::Snapshot::validate` valida política/digest/cobertura mas não conteúdo externo; `storage::load_snapshot` lê para Vec sem teto D0; `compare::View::state` dá prioridade a ancestral desconhecido, comprova ausência por componentes e resolve exclusão de ancestral; `compare::compare` omite unchanged de changes; `codec::decode` rejeita propriedades duplicadas e aceita JSON não canônico; `codec::canonical` usa JCS. Documento principal cap.7, §9.2 e cap.17 separa identidade, divergência e disponibilidade. Não é necessário alterar esses comportamentos.

## 2. Inputs e fonte verificável

Manifestos Foundation 1/0.1.0 são lidos como arquivos regulares, sem links nos componentes da referência, por handles fixados, com teto e checks de estabilidade/nome equivalentes aos da fonte. Após preflight estrutural dos manifestos (§6), validar primeiro baseline, depois current; recalcular política e digest completo conforme Foundation; compatibilidade de workspace/namespace/policy antes de fontes. Usar **o mesmo buffer bruto** para parse e `snapshot_file_sha256`. Outros JSONs válidos, inclusive espaços/ordem de propriedades diferentes, são aceitos; duplicados, BOM JSON, trailing data e UTF-8 inválido são rejeitados. Digest de estado não é digest do arquivo.

Pacote explícito: diretório raiz, `source.json`, `objects/<sha256 lowercase 64>`. Descritor possui exatamente os seis campos do `source.schema.json`: versões 1/0.1.0, workspace_id, snapshot_id, policy_id e SHA-256 bruto do manifesto. JCS é a forma de preparação recomendada; **outras serializações JSON estritas são aceitas**. `source_ref=SHA256(bytes brutos de source.json)` sem prefixo; whitespace muda source_ref mas não binding lógico. Hash de metric_policy usa JCS, diferente de ambos. Não gerar assinatura/autoria implícita.

Abrir/fixar pacote e resolver source/objects por descritores relativos, openat2 com NO_SYMLINKS/NO_MAGICLINKS/BENEATH, sem fallback stat+open. A raiz explícita também não pode atravessar links. Rejeitar link em qualquer componente, FIFO/socket/dispositivo/diretório como objeto; usar NONBLOCK/NOCTTY e conferir tipo antes de ler. Conferir dev/ino/tipo/tamanho/mtime/ctime/modo antes/depois e associação nome↔handle; substituição ou instabilidade detectada prevalece sobre hash coincidente. Sem retry, sem cache persistente ou arquivos temporários de conteúdo; nunca percorrer extras de objects ou fronteiras excluídas. Falha da capacidade Linux requerida é erro operacional, não autoriza fallback inseguro.

Flags omitidas: sources daquele lado=null, leitura local source_missing quando necessária. Flag presente mas raiz/source.json ausente/ilegível/tipo inválido/instável/malformado: erro 1 `source_invalid`; binding divergente: erro 2 `incompatible_inputs`. Validar descritores presentes **mesmo se universo vazio/unknown/excluído**; não validar objetos desnecessários. Distinção intencional: pacote presente é declaração de origem; objeto faltante de pacote vinculado é unavailable local `object_missing`.

Objetos necessários: ler uma vez no buffer consumido; hash SHA-256 e contagem dos mesmos bytes; conferir hash **e** tamanho contra revisão Foundation. EOF completo permite registrar actual hash/tamanho e checks; prefixo truncado jamais recebe hash de revisão. Metadados não substituem os checks. Vazio exige objeto existente conferido. Hash igual/estado igual não dispensa bytes/elegibilidade. Adulteração é indisponibilidade, nunca novo snapshot ou binário aceito.

Mapeamento de falhas locais: ENOENT antes de obter objeto → object_missing; acesso/erro de leitura → source_unreadable; link/componente proibido ou tipo especial → source_invalid_kind; metadados/nome divergentes ou desaparecimento após handle → source_unstable. Não transformar substituição detectada em object_missing. Instabilidade pode preservar actual_revision/flags se houve EOF, mas state=failed e reason source_unstable sempre impede uso. Outros checks de hash/tamanho efetivamente feitos também constam como motivos se falharam. Leitura interrompida/erro sem EOF tem actual_revision/hash_check/size_check null, exceto prefixo excessivo que prova size_check=false. A reserva permanece debitada. Essas regras não dependem do texto do erro do SO.

Fontes compartilhadas: compartilhamento de uma leitura somente quando os dois argumentos resolvem à **mesma raiz fixada (dev/ino), mesmo descritor bruto/binding válido para ambos e mesmo nome de objeto esperado**; normalmente exige o mesmo snapshot/manifesto bruto. Repetir descriptor físico é permitido, mas cada lado tem sua referência. Mesma source_ref em raízes distintas, hardlinks/reflinks entre pacotes ou hashes esperados iguais não autorizam empréstimo de evidência; conferir fisicamente cada lado. Cache limitado à unidade atual; nenhum cache entre caminhos. Contabilizar uma leitura/reserva se verdadeiramente compartilhada, duas nos demais casos.

### Preparação reproduzível sem ferramenta fictícia

1. Escolher manifesto já persistido, cópia autorizada de bytes de cada observed necessário e diretório fora do acervo. Hash e tamanho contra cada entrada antes de copiar. Conteúdo antigo inexistente não é reconstruído do hash.
2. Criar `objects/`; gravar cada revisão única como arquivo regular no nome SHA-256. Copiar bytes brutos, incluindo BOM/CRLF/vazios; não converter encoding. Pacotes não precisam conter todos os objetos, mas falta explícita impede medida integral. Não copiar exclusões nem varrer alvos excluídos.
3. SHA-256 do arquivo de manifesto **exato** usado; escrever os seis campos do descritor. Para fixtures ASCII/string-only desta entrega, `json.dumps(sort_keys=True,separators=(',',':'),ensure_ascii=True).encode('ascii')` sem newline tem bytes JCS conferíveis. Isso não é serializador JCS geral: outras entradas requerem JCS conforme RFC8785.
4. Recalcular hashes/tamanhos dos objetos copiados e hash bruto do descritor. Tornar cópias estáveis por coordenação com escritores; permissões read-only não provam imutabilidade. Registrar autorização/origem separadamente se necessário. Não modificar manifestos. O checker C2 confere pacotes literais pequenos, não captura acervos vivos.

Pacotes expõem revisões/segredos e custam a soma das revisões únicas + metadados. Sem GC/criptografia/apagamento seguro automático. Operador conserva/remove bytes; excluir pacote pode tornar próxima medida indisponível. Dentro do acervo seria necessária exclusão explícita antes do snapshot; preferir fora. Exclusões padrão `.compass`, `.git`, `.aws`, `.ssh`, `.env`, `.env.*`, exclusões exatas/prefixos, links e tipos especiais seguem Foundation; nenhum inventário adicional.

## 3. Universo e precedências

Candidatos = união dos caminhos de entries observed + caminhos de changes unknown; deduplicar por bytes nativos. Todo unchanged conhecido entra. Diretório de unknown_regions sem entry/change unknown mantém somente fronteira, sem unidade/arquivo fictício. Unidade unknown pode representar entrada/tipo desconhecido, não afirma arquivo. Não usar changes isoladamente como universo.

Obter estados efetivos por semântica de View::state, **não por before/after apenas**. Estes são cópias das entradas exatas ou null; entrada observed sob ancestral unstable continua expected revision conhecida mas não utilizável. Exclusão por ancestral pode não ter entrada exata no descendente; side.observation=excluded informa o estado efetivo, expected_revision=null. Exclusões nos dois lados/ausência não geram unidade; preservar todas as entradas excluded de cada manifesto em scope.exclusions, ordenadas caminho e lado. Se desconhecimento vence uma fronteira excluída, manter unknown. Excluído+ausente, excluído+excluído e diretórios sem arquivos não são transições textuais. Universo comprovadamente vazio pode ter exclusões padrão em ambos os lados, sem candidates/unknown/observed.

Prioridade de resolução por unidade:

1. Unknown de qualquer lado/ancestral: unavailable/observation_unknown; nenhum objeto lido; dois checks not_attempted (estados efetivos preservados).
2. entered_scope/left_scope: out_of_scope/scope_transition; não ler nem o lado observed. Excluído é excluded; observed é not_attempted. Ausência demonstrada é absent e não exige objeto.
3. Demais pares observed/observed ou observed/absent: preflight de tamanhos declarados dos dois lados; se qualquer s exceder max_file_bytes, bloquear a unidade antes de Wb/leituras. Somente os lados excedentes têm reason file_bytes_limit; os outros observed ficam not_attempted sem erro presumido, e absent mantém check absent. Se esse preflight passar, aplicar recursos e conferir fontes baseline antes de current. Executar os dois lados quando individualmente admitidos, mesmo após falha de baseline; orçamento compartilhado pode impedir current. Não deduzir erros de checks não executados.
4. Só com **todas** as revisões necessárias verified: classificar non_text se UTF-8 inválido ou NUL. `invalid_utf8` precede `nul` no mesmo lado (E20 inválido+NUL); listar por lado em non_text. Um binário conferido não encobre falha do outro lado. Non_text resolvido é out_of_scope, não bloqueio.
5. Para textos: normalizar/contar linhas; line_count_limit antes de admissão de memória das linhas; memory_budget antes de reservas de células; diff_work_limit por unidade antes do saldo total. Motivos são apenas falhas efetivamente observadas, únicos em ordem lexical. Não relatar todos os limites hipoteticamente violados.

Arquivo↔diretório coberto: removed/added no caminho antigo e descendants individuais; diretório não é lado binário. Hardlinks, case e caminhos não UTF-8 são unidades distintas. Renomeação é remoção+adição. Metadados de execução não alteram D0.

## 4. Texto, matemática e saída numérica

UTF-8 estrito sem U+0000; não detectar outras codificações. Depois de hash/tamanho aceitos: remover exatamente um U+FEFF inicial; CRLF→LF; CR isolado fica; vazio→[]; split por LF e retirar somente o fragmento final se LF terminal. Sem trim/NFC/casefold. Segundo BOM e U+FEFF interior permanecem. Identidade Foundation continua sobre bytes brutos.

Para linhas X/Y: n0=|X|, nt=|Y|, l=LCS(X,Y), r=n0-l, a=nt-l, c=a+r, m=n0+nt. LCS compara linhas inteiras exatas, preserva ordem e não exige contiguidade. Nenhum hash de linha sem confirmação de igualdade. Substituição custa remoção+adição. delta=c/m; quando m=0, delta=0/1, zero_basis=zero_text_mass. Somar C e M de todas as unidades available, inclusive unchanged/vazios. D0=C/M quando M>0. Não reduzir frações de auditoria. Sem hunks/desempate de roteiro. 0≤c≤m decorre de l≤min(n0,nt); simetria de c/m; r/a trocam. A→B→A usa somente extremos.

Inteiros/string-decimal canônica 0..18446744073709551615; positivos para denominadores/limites. Operações intermediárias checadas u128; excesso é erro 1 arithmetic_overflow, nunca saturação. Decimal seis casas pelo quociente/reste de numerator*1000000/denominator, empate para par. Razão é autoridade; arredondado positivo pode exibir 0.000000. Value de massa positiva, mesmo C=0, conserva 0/M. Massa zero usa 0/1.

Massa linear é recomendação P2: favorece arquivos grandes, conta as duas pontas. Histórico usa log(1+max(n0,nt)); delta pode coincidir, agregado não. E07 linear=0.1; log=log(2)/(log(2)+log(10))≈0.231378. Não há evidência de superioridade científica de um deles.

## 5. Disponibilidade e motivos globais

B = comparison.quality partial **ou** pelo menos uma unidade unavailable. T = quantidade de available, incluindo textos de massa zero. K = candidates.

| Condição | status/exit | value/line_mass/zero_basis | known_subset |
|---|---|---|---|
| !B, T>0 | complete/0 | C/M ou zero_text_mass | mesmo agregado, scope resolved_text_units_only |
| !B, K=0 | complete/0 | 0/1, massa 0, empty_universe | null (nenhuma unidade textual) |
| B, T>0 | partial/3 | todos null | agregado conhecido, inclusive zero_text_mass |
| B, T=0 | unavailable/1 | todos null | null |
| !B, K>0, T=0 | unavailable/1 | todos null | null |

Motivos globais calculados cumulativamente e ordenados: foundation_partial iff comparação partial; unresolved_units iff unavailable>0; no_text_units iff T=0 **exceto** complete/empty_universe. Assim região unknown sem inventário pode gerar [foundation_partial,no_text_units] sem unresolved_units. Bloqueio+binários e fonte ausente de único texto incluem no_text_units. Esta explicitação altera listas antes ambíguas de E28/E31, não seus valores/status.

Scope_transition/non_text resolvidos não tornam parcial o universo textual definido; pode haver complete zero com binário/transição adicional. **Zero não declara igualdade Foundation**. !B,K=0 aceita exclusões conhecidas; não declara vazio físico fora da política. Subconjunto não é bound nem porcentagem do integral; não inventar massa/contagem em regiões desconhecidas.

## 6. Política de recursos fechada

Defaults de `metric-policy.json`; todos os overrides são reduções positivas (nunca null, zero ou acima dos tetos), exceto max_duration_ms que admite null ou 1..3600000. Contadores de recursos string-u64. MiB=1048576. Sem paralelismo/eviction. Ordem de caminhos por bytes decodificados; baseline antes de current. Atalhos não mudam reservas.

| Campo limits | Default/teto | Resultado |
|---|---:|---|
| max_manifest_bytes | 16777216 por manifesto | input_limit/1 |
| max_source_descriptor_bytes | 65536 por descritor | input_limit/1 |
| max_records | 100000 entries+coverage dos dois | input_limit/1 |
| max_json_depth | 64 | input_limit/1; vale inputs D0/manifestos/política |
| max_file_bytes | 4194304 por revisão | file_bytes_limit local |
| max_total_object_bytes | 134217728 | total_bytes_limit local, regra de reserva abaixo |
| max_lines_per_side | 100000 | line_count_limit local |
| max_cells_per_unit | 10000000 | diff_work_limit local |
| max_cells_total | 50000000 | diff_work_limit local |
| max_logical_memory_bytes | 134217728 | input_limit estrutural/1 ou memory_budget local |
| max_output_bytes | 16777216 | output_limit/1 |
| max_duration_ms | null | deadline_exceeded/1 se configurado |

**Memória: ledger de admissão lógico `logical_credits_v1`, não capacidades reais ou RSS.** Resolve a promessa vaga C1: não se alega controle do alocador. Com b0,bt bytes brutos dos manifestos; d0,dt descritores presentes; N soma de entries+coverage+issues dos dois, reservar S=8*(b0+bt+d0+dt)+4096*N+65536. N inclui issues para evitar overhead não contado; max_records continua entries+coverage. Primeiro admitir S_manifest=8*(b0+bt)+4096*N+65536 após parse de transporte delimitado e antes de construir/validar Snapshot e Comparison. Depois validar compatibilidade, ler descriptors baseline/current e acrescentar 8*d por descritor aceito ao ledger, recusando se exceder. S final é a fórmula acima, admitida antes das unidades/relatório. Nenhum domínio é construído para um input recusado pelo seu preflight de memória. Após parsing delimitado de transportes, apagar buffers que não serão consumidos, porém **S permanece debitado** até fim; parsing também está limitado em tamanho/profundidade. JSON final tem limite próprio; não se diz que o ledger garante todas suas alocações.

Por unidade, s0/st tamanhos esperados de revisões observed, zero para absent: reservar Wb=2*(s0+st+2)+65536 antes de objetos. Inclui espaço lógico para raw e normalização; não liberar por shortcut/compartilhamento de buffer. Se Wb+S excede teto, memory_budget e não ler lados observed. Depois de linhas conhecidas, W=Wb+32*(n0+nt)+8*(min(n0,nt)+1). Se W+S excede, memory_budget, checks verified preservados sem diff. Liberar W ao terminar unidade; resources.logical_memory_peak é max(S,S+Wb admitido,S+W admitido). Wb recusado não altera peak. Conteúdo/índices da unidade não retidos; relatório já coberto por S lógico. I1 deverá demonstrar o ledger; R1 medirá RSS separadamente. Mudar fórmula exige novo candidato/policy_id antes de consumidores, não decisão oculta do alocador. O ledger é uma política verificável de admissão com números concretos, **não garantia de segurança contra OOM**.

**Bytes de objeto: reserva determinística conservadora** (mudança C1 para fechar EOF/saldo). Antes de abrir objeto, depois de arquivo/memória/fonte admitidos, debitar s+1 de max_total_object_bytes, inclusive probe EOF. Sem saldo: not_attempted/total_bytes_limit, continuar candidatos que couberem. Reserva aceita não é devolvida após missing/erro/leitura curta. Lê no máximo s+1 bytes, em chunks ≤65536. EOF antes de consumir s+1 bytes permite checks reais; se s+1 bytes foram lidos sem EOF, interromper, size_check=false, hash_check=null, actual_revision=null (prefixo), size_mismatch. Se EOF ocorreu com tamanho diferente, hash e size podem falhar conjuntamente. Contar bytes físicos efetivos (até prefixo/erro) e bytes reservados em campos distintos. Zero precisa reservar 1 para provar EOF. Atingir o saldo insuficiente por reserva não impede unidade menor que caiba; saldo zero bloqueia inclusive vazios. Compartilhamento real da §2 debita uma vez. Declared s>max_file_bytes precede leitura/reserva; buffer limitado s+1 protege contra manifesto pequeno/objeto gigante. Esses números substituem ambiguidade de C1; não anunciar hash completo sem EOF.

**Células:** depois de fonte verificada, classificação textual, normalização, limite de linhas e admissão de `W`, comparar as sequências normalizadas inteiras por igualdade exata. Se forem iguais, usar obrigatoriamente `l=n0=nt`, `a=r=c=0` e reservar/debitar **zero** células; `LCS(X,X)=|X|` prova a exatidão sem DP. Hash declarado, tamanho, contagem de linhas ou igualdade antes da normalização não acionam essa regra. A igualdade precisa ser da sequência já verificada e normalizada da unidade atual. Se forem diferentes, calcular `n0*nt` com produto checado: per-unit antes de total; se produto exceder um teto ou saldo, bloquear sem débito e continuar unidades menores. Reserva aceita de sequências diferentes não é devolvida. Logo textos iguais de 4.000×4.000 são available com zero células (E36), enquanto sequências diferentes de mesma contagem continuam `diff_work_limit`; uma unidade igual não consome o saldo da seguinte (E51–E55). Lado vazio continua exigindo fonte, mas, se ambos os lados chegam a sequências vazias verificadas, satisfaz a igualdade e custa zero células. Esta é política obrigatória `normalized_lines_exact_equal_zero_else_n0_times_nt_v1`, não otimização opcional. Inputs/subconjuntos sob orçamento de bytes e baseline-first não têm simetria garantida ao trocar lados; fórmula completa e reserva de células por caminho são simétricas sob ambos os lados admitidos. Não prometer mesmo JSON ao inverter proveniência/r/a.

Prazo opcional inicia no começo da operação após argumentos válidos, inclui input/output preparado; relógio monotônico. Checar SIGINT antes de deadline, depois erro descoberto (cancelamento vence no último checkpoint). Entre leituras/chunks/units e a cada ≤4096 células/operações LCS. Deadline/SIGINT abortam com envelope, nenhuma medida/subconjunto final. Syscall bloqueada pode atrasar prazo. Preparar saída canônica completa antes de stdout; bytes de stdout falho podem ser parciais por erro de I/O, sem promessa de transação de pipe.

## 7. Interface e transporte

Assinatura candidata (inexistente hoje):

```text
compass d0 --baseline <json> --current <json>
  [--baseline-source <dir>] [--current-source <dir>]
  [--limits <limits.json>] [--deadline-ms <1..3600000>] [--json]
```

Sem limits: defaults. limits.json é **objeto completo** com todos campos estáticos de limits.schema.json; leitura ≤65536 bytes, JSON estrito/profundidade64; configuração inválida/ilegível/oversized → invalid_arguments/2. deadline_ms só via flag, null por default; não em limits.json. CLI monta metric_policy.limits incluindo max_duration_ms e ledger enum. Perfil reduzido é limits completo com reduções explícitas; **sem** flag de teste oculta/perfil implicitamente inferido. `--json` saída um objeto JCS sem newline; humano mostra status, snapshots, razões e motivos sem chamar known_subset de integral. stdout JSON sem conteúdo/hunks/raízes; stderr operacional.

0 complete, 3 partial, 1 unavailable (relatório) ou falha registro/operacional (envelope), 2 argumentos/incompatibilidade, 130 SIGINT. Erro é `{error:{code,message}}`; code enum: invalid_arguments, incompatible_inputs, input_invalid, input_limit, source_invalid, output_limit, arithmetic_overflow, operational_error, deadline_exceeded, cancelled. message fixa por code em erros do núcleo (igual ao code); detalhe operacional em stderr. Não incluir campos de relatório no erro. Prioridade: argumentos → transporte dos manifestos/preflight S_manifest → validação baseline/current → compatibilidade → descriptors baseline/current e admissão incremental S → Comparison/unidades → output; dentro de fase SIGINT/deadline em checkpoints precedem publicação. `init/snapshot/compare` preservados, sem adaptar seus envelopes.

Campos fechados nos schemas `report`, `source`, `metric-policy`, `limits`, `error`; fields adicionais rejeitados. Todos os nullable definidos são obrigatórios mesmo null; apenas `unit.calculation` é **ausente** se status≠available. Não pôr contagens/delta null em unavailable. before/after são Entry exatas ou null e seguem ausências do Foundation. `observation` dos checks é observed/absent/excluded/unknown; `state` verified/not_attempted/failed/absent/excluded. expected_revision para entrada exata observed inclusive unknown ancestor; nenhum hash sintético absent. actual_revision somente EOF completo (mesmo falho); hash_check/size_check null se não verificáveis; size_check=false pode existir para prefixo excessivo sem hash.

Report `baseline/current`: snapshot_id, schema_version, contract_version, producer_version/build, started_at,finished_at,consistency,quality,state_digest nullable,snapshot_file_sha256. Fontes null ou descritor completo+source_ref. `comparison` exatamente objeto Foundation separado; redundantes foundation_quality/relation precisam coincidir. producer_version/build identificam emissor real em I1/R1; fixtures usam `candidate-fixture`/`fixture:no-production-build`, não build alegado.

Unit: path_key,classification, before,after,status,reasons,non_text (array side/reason),checks {baseline,current}, calculation somente available. calculation tem n0,nt,l,a,r,c,m,delta,zero_basis. Classificações unchanged/added/removed/modified/metadata_changed/entered_scope/left_scope/unknown. reasons enum está no schema; non_text subtypes invalid_utf8/nul. Scope conta candidates/resolved_text_units/unavailable_units/non_text_units/scope_transitions e exclusions {side,entry}; unknown_regions já está em comparison (sem duplicação). Resources: object_bytes_read,object_bytes_reserved,cells_reserved,logical_memory_peak. Todas as contagens são strings u64; schema limita formato/20 dígitos, núcleo aplica teto u64 preciso. Policy está em `metric-policy.json` e schema congela texto/matemática/ledger/reservas; todos limites efetivos + prazo incluídos no hash.

`metric_policy_id=SHA256(b"compass-d0-policy-v1\0" || JCS(metric_policy))`, onde `\0` nesta notação significa **um byte NUL**, não dois caracteres. Sem produtor/raiz/manifestos/fontes no hash. Vetor raw prefix hex em evidence para eliminar ambiguidade. Não inferir que JSON dumps é JCS geral. Report JCS usa biblioteca de produção já existente; fixtures ASCII/string-only de C2 têm canonical bytes fixos.

Ordenações: candidates/changes/exclusions/unknown_regions por bytes nativos decodificados; empate exclusions/regions por side lexical (baseline,current). Reasons únicos lexicais; non_text por side; propriedades JCS UTF-16; unidades não ordenadas por base64url nem display. Declared object hashes não determinam ordem. Sem UUID/timestamp novos/duração de execução no relatório.

### Validações semânticas do núcleo (schema não substitui)

- u64 preciso, paths base64url canônicos/componentes válidos, ordem/uniqueness nativa, snapshots/Foundation IDs/digest/árvore/counters/janelas e comparação recomputada.
- Descritor binds cada manifesto bruto e raiz/objeto aberto seguro; fonte real não inferida de schema/source_ref. Report refs correspondem aos descriptors presentes, expected revisions aos snapshots; checks verified exigem hashes/tamanhos iguais, flags true e nenhum motivo; source_ref null para fonte omitida/ausência/exclusão.
- Estado efetivo vs entrada exata; before/after e classificação recompostas, candidatos e exclusões sem omissões; unknown anterior a scope; ambos verificados antes de non_text. Out_of_scope somente reasons non_text/scope_transition; unavailable sem calculation; available com ambos verified/absent, non_text/reasons vazios.
- l≤min; a/r/c/m pelas equações; valores/decimal e zero_basis; agregado por todas available, known_subset/scope/status/reasons/exit conforme §5. Nenhuma massa unknown.
- Policy hash JCS, limites reduzidos/consistência max_cells_total≥max_cells_per_unit não é obrigatória (saldo menor é legítimo); deadline domínio; ledger e reservas conforme §6; read≤reserved≤teto; cells igual reservas aceitas; comparação inteira preservada.
- Estrutura do schema não prova links/TOCTOU, observação atômica ou hash autenticado. Mesmo transporte válido pode ter fraude numérica/binding. V1 acrescentará validação independente e negativos, I1 valida no núcleo.

## 8. Entrega para consumidores e plano condicionado

C2 entrega este contrato, schemas, `fixtures/cases.json` (E01–E55), políticas bytes fixos, pacotes/manifests-base, relatórios representantes/envelopes e `evidence/conferir.py` com contas pequenas e schemas. Índice SHA-256 relaciona todos. As evidências de C2 não são laboratório completo nem conformidade de futura produção.

| Etapa futura | Dependência/áreas | Aceite/checagem |
|---|---|---|
| Coordenação/ativação | Decidir P1–P4, conferir índice/parecer e registrar aprovação; destino ativo definido pela coordenação | Mesmo conjunto contrato/schema/esperados identificado por hash. Mudança de significado reabre revisão e fixtures; nenhuma liberação implícita deste parecer |
| V1 | Só após liberação; verification/reference e fixtures novos separados dos Foundation | Transformar E01–E55 em campanha completa, transporte+semântica, corridas sincronizadas/limites/mutações. Usar esperados congelados antes de Rust; nenhum import de produção |
| I1-a | Liberação; núcleo novos tipos/leitores/ledger/fonte | Binding/mesmos bytes/aberturas seguras, limites/checkpoints reproduzíveis, sem retenção e sem alterar Foundation |
| I1-b | I1-a; universo/normalização/LCS/relatório | Todas equações, disponibilidade/reservas/ordem, inclusive unchanged e E36; checks focados de contrato |
| I1-c | I1-b; CLI/readme | Novo comando com limits/deadline completo; Foundation preservada; só depois integrar V1 |
| R1 | V1+I1; integração | Release, regressão pertinente, oráculo CLI, recursos reais, revisão de fonte/build e limitações; nenhuma alegação de eficácia contextual |

Comandos reais futuros: `./scripts/cargo-local.sh test --workspace --locked --offline`, `fmt --all -- --check`, `clippy --workspace --all-targets --locked --offline -- -D warnings`, `build --release --workspace --locked --offline`; `python3 verification/reference/acceptance.py --binary target/release/compass` e mutations.py. Eles verificam Foundation hoje, não D0; novos comandos só existem quando V1 os criar. Paralelo futuro apenas V1/I1 em áreas separadas após liberação; um escritor Rust. Não executar essas fases por editar candidato.

Ponta a ponta futura: literal→manifesto→descriptor→abertura segura→mesmos bytes/hash/tamanho→linhas→LCS→razão/aggregate→report/exit. Retirar árvore viva não muda pacote válido; retirar/adulterar objeto impede integral; partial ancestor não é corrigido por fonte; normalização mantém modified; limites e SIGINT não publicam aproximação. Preservar hashes dos inputs, conferir report repetido e registrar build real. C2 está preparada quando artefatos/documentos/conferências são reproduzíveis; **ativação ainda depende das decisões de produto**.
