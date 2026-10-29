# Auditoria de contrato — OS-RD-I1 v1

**Veredito: CONFORME na versão normativa identificada abaixo. Nenhum achado obrigatório aberto.** O único achado contratual foi corrigido e reconferido antes da liberação. Esta auditoria autoriza tecnicamente o próximo portão da coordenação; não declara a implementação RD entregue nem substitui sua auditoria posterior.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo `contrato`, com `references/achados.md`. Escopo: primeira busca literal verificável, RD01–RD24 e RD-AC-01–10. Nenhum código, contrato, OS, gabarito ou teste foi alterado pelo auditor; sua única escrita no projeto é este parecer.

## Referência concreta

Branch `main`, sem HEAD: `git rev-parse --verify HEAD` retornou 128. Git/status/diff inspecionados; arquivos adicionados e não rastreados já existiam. Nenhuma conclusão é atribuída a commit inexistente.

| Autoridade auditada | SHA-256 |
|---|---|
| [Contrato RD v0.1, versão 1 revisada](../../docs/entregas/rd/CONTRATO-RD-V0.1.md) | `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72` |
| [OS-RD-I1 v1, PREPARADA_PARA_AUDITORIA_DE_CONTRATO](../../docs/entregas/rd/OS-RD-I1.md) | `3e48d1e40adab74b6e4eab7fcd1022499418e94088d21d5424754a8fcb25f11f` |
| [Plano RD](../../docs/entregas/rd/PLANO-ENTREGAS.md) | `ce401a09bd90f06b69d0bbb0688e4389bfd2140eb82e6488c91122f29e2405af` |

Baseline técnica conferida por bytes: binário C01/R1 `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`; recomposição independente dos 37 inputs com o framing existente produz `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`. Os novos documentos RD não pertencem a esses inputs históricos. Índice D0 `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` e suas 81 entradas conferiram. O parecer R1 conserva o SHA informado pela OS: `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9`.

## Achado e fechamento

| ID | Tipo | Estado | Evidência/reprodução | Critério de fechamento | Tentativas | Responsável | Versão |
|---|---|---|---|---|---|---|---|
| BUG-RD-C-001 | contrato | verificado | O §5 original quantificava checkpoints em toda “validação/linhas/busca”, mas o leitor protegido calcula `digest(&bytes)` depois de ler o objeto inteiro, entre `read_finished` e `object_before_stability`; parse, validação Foundation e JCS também são chamadas indivisíveis. | Limitar 65536 bytes aos novos laços RD; preservar checkpoints D0; declarar ausência de preempção interna dos primitivos compartilhados e exigir checagens antes/depois. | 1 | arquiteto/coordenação | Contrato inicial `3095d73d25f738877f2a83b2f8fee380a211d56bd865d1551f2ad6e5b98c5f84` → versão normativa acima |

Evidência na árvore real: [leitura e hash](../../crates/compass-core/src/d0/source.rs), linhas 258–285 e 343–365; [parse compartilhado](../../crates/compass-core/src/d0/input.rs), linhas 14–49; [codec](../../crates/compass-core/src/codec.rs), linhas 10–11 e 98–102. A redação inicial poderia exigir editar arquivos expressamente protegidos pela OS ou alegar uma frequência inexistente. O contrato final, §5, linha 103, corrige precisamente essa incompatibilidade e mantém o requisito nos laços novos, inclusive sem LF/hits. Não é defeito novo de implementação D0.

As explicitações adicionais de que o envelope fixo não está sujeito ao cap do relatório e de que CR isolado/U+2028 não separam linhas são coerentes com as regras já existentes. Não geraram requisito de produto ou achado separado. A correção foi realizada pela coordenação somente no novo contrato; OS, plano e produção permaneceram iguais durante esta conferência.

## Conferência de implementabilidade e semântica

| Área | Resultado e evidência |
|---|---|
| Fronteira real de reutilização | `d0/input.rs` expõe internamente parse, Manifest e source; `resources.rs` contém Context; `source.rs` contém Root/raw_file/Source::object. Os campos de Limits necessários já têm visibilidade crate. Tornar os três módulos `pub(crate)` permite a reutilização prevista sem duplicar o leitor ou mudar D0. O preflight de manifesto único/N incluindo issues é próprio RD; a OS proíbe simular um par de snapshots em `measure`. |
| Bytes, revisão e estabilidade | Matching ocorre após EOF/hash/tamanho/estabilidade, com bytes originais; offsets são intervalos no objeto bruto, BOM/CRLF são preservados e nomes nativos servem para identidade/ordem. O leitor existente conserva fatos de EOF mesmo com instabilidade e não fornece hash completo para prefixo de s+1 sem EOF. RD10/12/21/23 e AC-04 exigem provas específicas dessa fronteira. |
| Cobertura e seleção | `searched/non_text` são resolvidos; unavailable e snapshot parcial impedem complete. Universo completo vazio/excluded/non_text pode ser complete sem hits. `matching_lines_known` conta searched e `matches_truncated=(known>returned)` indica somente seleção. RD16 exige continuar após o limite e registrar falha tardia, evitando ausência global falsa. |
| Limites e créditos | N inclui entries+directory_coverage+issues; S_manifest admite o domínio, S acrescenta descritor e W precede reserva/abertura. Reserva s+1 não é reembolsada; recusa não debita nem aumenta peak, permitindo arquivo menor posterior. Output excessivo gera envelope, sem remover hits/diagnósticos; ledger é lógico e não promete RSS/OOM. |
| Transporte e cancelamento | Campos fechados, strings u64, JSON estrito/JCS, binding bruto e erros/exits têm definição verificável. §5 final é compatível com o leitor protegido; RD19/22 e AC-06/07 exigem controles sincronizados e ausência de publicação parcial preparada. Argumentos que coincidem com comandos têm regressão explícita. |
| Preservação e regressão | Lista de escrita fecha as alterações D0 em visibilidade. Contrato RD entra expressamente no build novo. V1 intacta pode ser montada via verify/mount autenticados e receber o executável RD absoluto; a OS evita adapter.run na origem e o runner R1 fixado ao release antigo. Evidências antigas e gabaritos são preservados, com regressão do novo release obrigatória. |
| Produto, acesso e reversão | A fatia entrega busca/linhas da revisão declarada pela CLI, coerente com documento principal §§6.3/7/13/17.3. Não depende de ranking, índice ou D0 como gate. Só pacotes sintéticos, nenhum acervo real/rede/deploy; cópias e deltas permitem reversão delimitada, preservando binário e evidências anteriores. |

Não há conflito entre o objetivo, a lista fechada de escrita e RD-AC-01–10 na versão final. RD01–RD24 são obrigações da execução futura, não 24 testes de produção já aprovados por este parecer.

## Conferências independentes executadas

Biblioteca padrão Python, sem importar Rust/oráculo RD e sem invocar produção:

- RD01: 20 bytes, 3 ocorrências e 2 linhas matching, intervalos `[0,5)`/`[10,20)`; RD02: um hit `[0,4)` apesar da sobreposição.
- RD05: 15 bytes, linhas `[0,8)`/`[8,15)`; RD06: 8 bytes com BOM preservado. Controle `a\rb<U+2028>c\n`: uma linha de 8 bytes. Vazio/LF terminal, ordem nativa `10/2/0xff`, query UTF-8 de 256/257 bytes e linha de 4096/4097 bytes também conferidos.
- Carga textual regenerada: 1.000 arquivos de 4.096 bytes em 100 diretórios, BASE somente em j=31; leitura 4.096.000, reserva 4.097.000, 1.000 linhas matching, seleção 20/truncated=true.
- Crédito literal independente: b=1234, N=5, d=321, output=4096 e s=8192 produzem S_manifest=128656, S=131224, W=81922 e peak admitido=213146. Igualdade ao teto admite; um byte abaixo recusa. Nenhuma contradição aritmética encontrada nos limites ou gabaritos.
- `run-v1.py` verify/mount autenticou o tar `bbc9dd44c871585fa41e2599bc497d1598898918d472a561683340fc55c86c62`, seus 34 membros históricos e a cópia intacta de acceptance.py/mutations.py. A montagem foi feita em /tmp; nenhuma campanha/sensor foi executado.

Evidências auxiliares: `/tmp/compass-audit-rd-contract-s4fbxxqp/checks.json`, SHA-256 `e54b04c4c3be569c44ae2bd68c700ae1f0c5c6a16e21d8d6e733bc2622397193`; inventário de 175 arquivos, `before.json`, SHA-256 `e2a7444cf569188965790c9c43422ccbe83691c37bfa2403979fe57d763a1be6`. Esse inventário foi capturado após a correção contratual; os hashes iniciais das autoridades foram registrados na leitura anterior. A conferência final preservou os 175 arquivos e identificou somente este parecer como acréscimo. `git diff --check` e checagem de whitespace do arquivo novo passaram.

Limitação: não foram executados build, testes Rust, suites Foundation/D0, campanha RD ou medições nativas, pois a entrega auditada é documental. Nenhum resultado runtime anterior foi atribuído à busca futura. A execução e sua auditoria devem demonstrar os aceites na identidade nova.

## Portão

Sem requisito bloqueante pendente. A coordenação pode liberar OS-RD-I1 alterando somente seu estado administrativo para LIBERADA_PARA_EXECUCAO; o conteúdo normativo aprovado é o vinculado acima. Mudanças posteriores de escopo/contrato/aceites exigem identificação e conferência próprias. A entrega implementada permanece sujeita à auditoria independente prevista na OS.
