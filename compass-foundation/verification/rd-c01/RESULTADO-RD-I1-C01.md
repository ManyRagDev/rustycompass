# Resultado OS-RD-I1-C01 — BUG-005

**IMPLEMENTADO_PENDENTE_VERIFICACAO**. Versão 1; 19/10/2026,
America/Sao_Paulo. Skill executor-blindado v2.0. Correção de implementação,
**tentativa #1**, do mesmo BUG-005 confirmado pela auditoria RD-I1.
O executor não fecha o BUG nem declara aceite RD; aguarda conferência focada.

## Baseline e fronteira

`main`, sem HEAD/commit: verificação inicial retornou 128. Git/status/diff e
hashes foram inspecionados antes da escrita. A baseline administrativa atual
contém 189 arquivos, incluindo auditoria/coordenação posteriores à RD original;
não foi confundida com o inventário histórico RD.
Cópias, inventário, diffs e binário anterior: `/tmp/rd-c01-baseline-zf5_t_8_`.
O inventário bruto tem SHA-256 `81c2b6d36a705ace727aeb263244a1ac294936ebb227a787dd7ff5b5510ee69b`.

| Referência | SHA-256 / identidade |
|---|---|
| OS-RD-I1-C01 liberada | `4ea245f6137b41e4c33ca279360e8bbb4d40ca60dd334a99f06ffcc37b44bae6` |
| Auditoria delimitada RD-I1 | `7700e7e1f310994e2226167c177eb6aa377e3268d90f0a87463ba53d04fd964f` |
| Binário RD anterior preservado | `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d` |
| Produtor anterior (41 inputs) | `sha256:6192a943a657f23ebaea7267eea67f63ee482741c85c37d44634817113bfebf9` |
| Núcleo search.rs preservado | `04a8e696d7fd3739419872aa5b7ed7760f7c9d5e1cf654fcb7167c968f5f65aa` |
| Binário C01 final, antes/depois das campanhas | `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8` |
| Produtor C01 (41 inputs) | `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012` |

Todos os hashes de baseline enumerados pela OS conferiram antes das escritas.
A recomposição dos inputs com o framing Foundation identifica ambos os builds;
somente os dois inputs CLI autorizados mudaram. Versão do executável: 0.1.0.
Nenhum commit, reset/clean, operação remota ou outra capacidade foi iniciado.

## Prova antes/depois e correção

Antes do patch, foram criados os controles públicos e as duas regressões
permanentes, sem mudar main.rs. O checker original executou 44 controles contra
a cópia do release RD: **43 passaram; um falhou somente em `BUG-005.format`**.
O teste Rust também falhou exatamente em `BUG-005.format`, depois de conferir
exit 2, stderr vazio e valor invalid_arguments. A outra regressão Rust, de
queries válidas hifenizadas, passou antes do patch.

O caso específico era `search --snapshot <ausente> --source <ausente>
--query --json --max-matches bad`, sem flag real de saída. O parsing falha antes
de tentar ler arquivos; o erro observado era correto, mas compacto sem LF.
O esperado fixado é JSON legível com LF. O controle equivalente
`--query=--json` já passava e permaneceu válido.

A mudança mínima acrescenta `search_error_json_requested`, usada somente no
ramo de erro de parsing de search. Ela respeita o consumo do valor separado de
`--query`, a única opção search que admite valores hifenizados; a forma com `=`
não expõe o valor como token de opção. Continua examinando tokens posteriores
para encontrar uma flag real, inclusive após o argumento inválido, e respeita
`--`. Uma opção de caminho sem valor deixa `--json` como flag. Não há nova
leitura de arquivo, alteração do parser tipado ou proibição de query hifenizada.
A decisão histórica Foundation/D0 permaneceu igual.

Após o patch, o checker foi ampliado com quatro controles de snapshot válido e
fonte ausente. A cópia imutável do binário anterior foi reconferida nessa matriz
ampliada: **47/48**, novamente somente `BUG-005.format`. Essa reconferência é
posterior ao patch dos fontes e usa o executável antigo; não substitui nem
redata a primeira prova de 44 casos anterior ao patch.
O release corrigido passou **48/48** controles.

| Controle, em ambas as formas de query | Binário anterior | Binário C01 |
|---|---|---|
| Sem flag, query separada e max-matches bad | Falha específica: compacto sem LF | Legível com LF, bytes exatos |
| Sem flag, query com `=` e max-matches bad | Legível com LF | Legível com LF, mesmos bytes esperados |
| Flag real antes de search | Compacto sem LF | Compacto sem LF |
| Flag real após search/antes de query | Compacto sem LF | Compacto sem LF |
| Flag real após max-matches bad | Compacto sem LF | Compacto sem LF |
| Snapshot sem valor seguido de flag real; query comum | Compacto, invalid_arguments/2 | Mesmo comportamento |
| Parsing válido com snapshot ausente / fonte ausente | input_invalid/1 ou source_invalid/1, modo correto | Mesmo valor/exit/modo |
| Pacote sintético válido; query --json/--needle/-x/d0/init/search | Query literal e linha correspondente | Mesma semântica, com/sem flag |
| Help/version, search/init/d0 help | Passam | Passam |

Nos erros de parsing, ambas as sintaxes mantêm exatamente
`{"error":{"code":"invalid_arguments","message":"invalid_arguments"}}`,
exit 2 e stderr vazio. O modo legível tem indentação e newline final; o modo
JCS é compacto sem newline. A matriz e seus bytes/argv/exits/hashes recuperáveis
estão em [integration.json](integration.json), `controls` e `before_after`.
Os controles válidos e com flag real nunca foram classificados como falha esperada.

## AC → evidência

| AC | Resultado do executor e prova |
|---|---|
| C01-AC-01 | Reproduzido antes do patch no release anterior por uma única asserção `BUG-005.format`; controle com `=` passa. `integration.json.controls[0]`, reconferência ampliada em controls[1]; logs públicos/Rust brutos em `provenance.json.commands`. |
| C01-AC-02 | Duas formas sem flag legíveis com LF; flag real nas três posições compacta sem LF; código/valor/exit/stderr invariantes. `integration.json.controls[2]`: 48/48; pares `before_after`. |
| C01-AC-03 | Testes permanentes `bug005_query_json_is_a_value_and_real_flags_select_transport` e `bug005_valid_hyphenated_queries_keep_typed_transport`; fonte e snapshot ausentes, queries hifenizadas/literais e help/version cobertos. Parsing inválido é demonstrado por invalid_arguments, sem compensação por acesso ao acervo. Testes antigos preservados integralmente como prefixo do arquivo. |
| C01-AC-04 | 76 testes Rust, fmt/Clippy/release passam. RD intacta em cópia: 151 comandos, 1.482 assertivas auxiliares, cinco mutações e três medidas. Foundation: 23 testes/três mutações; V1: 253 verificações/119 cenários e 11 mutações D0, sem skips finais. `integration.json.campaign`, `regressions.json` completo e logs da proveniência. |
| C01-AC-05 | Exatamente dois deltas e cinco novos autorizados; 187 arquivos preexistentes preservados, incluindo verification/rd inteira e núcleo. Inventário/diffs/hashes e 41 inputs em `provenance.json`; 81 entradas C2-A1 e whitespace conferidos. |
| C01-AC-06 | BUG-005→RD22/RD24/RD-AC-07 e testes permanentes em `integration.json.correction_mapping`; provas antes/depois e logs recuperáveis. Retorno pendente de conferência independente. |

As 1.482 assertivas não representam cenários independentes. Os testes novos
complementam RD22/RD24 na evidência corretiva; nenhum gabarito, cobertura ou
verificador histórico foi editado para artificialmente incluir BUG-005.

## Deltas e preservação

| Arquivo autorizado | SHA antes | SHA depois |
|---|---|---|
| `crates/compass-cli/src/main.rs` | `9338279a2f36151e2806071bdc78848bbf976b9a0caea08abfa774a87cca39c8` | `864c93850e8962508858591cc112cedacfdcb5eba51c2e47140d8802b29f1395` |
| `crates/compass-cli/tests/search.rs` | `6cba0c053c2502dfcbc95d389fc40c532090191336ed9b6e0e50d8b88b1d3f84` | `96ea97897ccee2460f40c930a1834af274cfb9582fdff089e5e551da9de7a0b0` |

Novos, somente: `check.py`, `integration.json`, `regressions.json`,
`provenance.json` e este resultado, todos em `verification/rd-c01/`.
Os 187 demais arquivos da baseline ficaram idênticos. Isso inclui núcleo/D0,
contratos, fixtures, Cargo/lock/toolchain, README, verification/rd, Foundation,
V1/I1/R1, estado, coordenação, OS e auditorias. Não foram alteradas políticas,
recursos ou fórmulas RD. Diffs completos e cópias permitem reversão limitada aos
dois arquivos e às saídas próprias, sem sobrescrever mudanças anteriores.

## Execuções reais

```sh
# Antes do patch: retornos 1 e 101 esperados, por BUG-005.format.
PYTHONDONTWRITEBYTECODE=1 python3 verification/rd-c01/check.py --control-only --binary <baseline>/compass-rd-before
scripts/cargo-local.sh test -p compass-cli --test search bug005 --locked --offline

# Depois: todos retornaram 0.
scripts/cargo-local.sh test -p compass-cli --test search --locked --offline
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
PYTHONDONTWRITEBYTECODE=1 python3 verification/rd-c01/check.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

O checker copiou o workspace sem .git/target/bytecode e a referência documental
necessária para TemporaryDirectory. Conferiu os hashes de verification/rd antes
da execução, e chamou o acceptance original da cópia com `--binary` e depois
`--regressions`, usando o executável C01 absoluto. A campanha/regressões
alteraram somente os outputs da cópia, incorporados completos nos JSONs C01.
A montagem V1 é autenticada pelo mecanismo original verify/mount; adapter.run
na origem e runner R1 fixado ao binário antigo não foram chamados. Bytecode Python
foi desabilitado. Comandos, cwd, stdout/stderr brutos e identidades são registrados.

## Três novas medições nativas

Receita RD preservada: 1.000 arquivos de 4.096 bytes/100 diretórios, BASE somente
em j=31, um snapshot/pacote. Em todas: 1.000 linhas matching, 20 retornadas,
truncated=true; stdout idêntico; leitura 4.096.000, reserva 4.097.000 e peak lógico
14.690.378 bytes. GNU time individual e wall com wrapper:

| Amostra | User/System (s) | Max RSS (KiB) | GNU elapsed (s) | Wall (ns) |
|---|---:|---:|---:|---:|
| 1 | 0.05 / 0.02 | 12584 | 0.08 | 83068735 |
| 2 | 0.06 / 0.01 | 12728 | 0.08 | 85929601 |
| 3 | 0.06 / 0.01 | 12692 | 0.08 | 85533841 |

Perfil completo em `integration.json.campaign.resource_profile`: ambiente,
limites, disco, cache, medidor/argv brutos, stdout/base64/hashes e ledger separado
de RSS. São amostras nativas do novo release, sequenciais e sem compilação
simultânea iniciada pelo executor; não são medidas históricas reatribuídas.

## Limitações e ocorrências

- As falhas pré-patch são reproduções específicas do BUG-005, com controles
  válidos passando; não foram contabilizadas como sucesso da implementação.
  Houve um único patch de produção. Não ocorreu falha de campanha após o patch.
- O gerador temporário de proveniência teve um SyntaxError por chave version
  duplicada; foi corrigido antes de executar. Não envolveu fonte, contrato,
  esperado ou teste comportamental e está registrado como erro de ferramenta.
- Testes e campanhas usaram permissão técnica para socket Unix local. Nenhum
  skip/restrição final; nenhuma ação remota. Só Linux/openat2 e dados sintéticos.
- As anotações runtime_pending nos JSONs V1 foram preservadas literalmente;
  não há promoção de smoke histórico a prova sincronizada ou reauditoria do
  núcleo. Esta correção trata exclusivamente transporte pós-erro.
- Cache não controlado, sem flush ou SLO; créditos lógicos não são RSS nem
  garantia contra OOM. Limites cooperativos e limitações do núcleo RD permanecem.
- Temporários podem ser removidos pelo sistema; hashes, inventário, deltas,
  matriz e logs relevantes estão incorporados nos artefatos corretivos.
- Uso compartilhado da conta: 23%→23% arredondados na janela de 10.080 minutos;
  não representa contagem de tokens desta correção.
- Nenhuma instalação, rede, migration, deploy, commit, acervo real, mensagem a
  outro chat ou próxima capacidade. BUG-005 segue aguardando verificação externa.

## Hashes dos artefatos corretivos

| Arquivo | SHA-256 |
|---|---|
| `verification/rd-c01/check.py` | `5e15e4df2d6a3d5610cdb274880c5877d28b8ac02f9bf084e8282852177e85cd` |
| `verification/rd-c01/integration.json` | `dac9438aae218fff155726c3bbff0a7e925d99d92d24e40e328afdc77c9daefd` |
| `verification/rd-c01/regressions.json` | `26fc20d4df4daf693ed3319858ce6cc82abacb83c967571eb19e72257c1f6903` |

A proveniência exclui seu próprio hash e este resultado para evitar circularidade;
os dois hashes finais constam da mensagem externa.

**IMPLEMENTADO_PENDENTE_VERIFICACAO**
