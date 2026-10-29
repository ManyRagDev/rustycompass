# OS-RD-I1-C01 — transporte da query literal `--json`

```yaml
id: OS-RD-I1-C01
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: curto
origem: BUG-005 confirmado pela auditoria OS-RD-I1
objetivo: distinguir flag de saída JSON de valor literal de query também após falha do parser
execucao: sequencial; mesma conversa do implementador RD; GPT-6.1 Sol / Alto; um escritor
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.

Esta OS é o contrato mínimo de correção, sem mudar [CONTRATO-RD-V0.1.md](CONTRATO-RD-V0.1.md) ou aceites originais. O porte curto é proporcional à causa: transporte de argumentos inválidos na CLI, sem alteração de fonte, algoritmo, conteúdo entregue, recursos ou política de acesso. Conferência independente focada após a execução; não há decisão de produto ou outro gate de ativação pendente.

## Diagnóstico e baseline

Ler AGENTS.md, COMPASS_STATE.md, [auditoria RD-I1](../../../research/audits/AUDITORIA-OS-RD-I1.md), esta OS e contrato RD §§2/4. Consultar CLI main.rs/testes search e laboratório RD conforme necessário. Não reiniciar a OS-RD-I1 ou reimplementar a busca.

`main`, sem HEAD/commit. Inspecionar Git/diff e inventariar arquivos, não rastreados e binário antes de escrever; guardar cópias/deltas em /tmp, sem reset/clean. A baseline desta correção é a entrega RD auditada:

| Referência | SHA-256 |
|---|---|
| Auditoria RD-I1 | `7700e7e1f310994e2226167c177eb6aa377e3268d90f0a87463ba53d04fd964f` |
| Binário RD entregue | `e0ebfab777fcdf08fec02364566bc903303694adcd191dff49b45b7f2056b41d` |
| CLI main.rs | `9338279a2f36151e2806071bdc78848bbf976b9a0caea08abfa774a87cca39c8` |
| CLI tests/search.rs | `6cba0c053c2502dfcbc95d389fc40c532090191336ed9b6e0e50d8b88b1d3f84` |
| Núcleo search.rs protegido | `04a8e696d7fd3739419872aa5b7ed7760f7c9d5e1cf654fcb7167c968f5f65aa` |
| Resultado RD original | `1a0613b0f705e23e1c260011a486a7f0a63189189963b6d94adc9d6b908d1f50` |
| Proveniência RD original | `54ebd07bae46ff450c1fc6eba07ec4038d740523797b55f6bdc7762c5624ba52` |
| Acceptance RD original | `609882519abfa81d02d54e28176ef2bda8c988141b83854635c1c71a271c01ba` |

Produtor anterior, 41 inputs: `sha256:6192a943a657f23ebaea7267eea67f63ee482741c85c37d44634817113bfebf9`. Preservar cópia do binário anterior antes do build; identificar o novo release depois das alterações finais, sem inventar hashes futuros. A baseline administrativa inclui os registros de auditoria/coordenação posteriores à entrega; não exigir igualdade desses registros com o antigo inventário RD nem regravar sua história.

## Comportamento obrigatório e regressão

BUG-005: a detecção prévia `args_os().any(|a| a=="--json")` trata o valor de `--query` como flag; o erro posterior usa essa decisão. No binário anterior:

```sh
target/release/compass search --snapshot x --source x --query --json --max-matches bad
target/release/compass search --snapshot x --source x --query=--json --max-matches bad
```

A primeira forma emite compacto sem LF e a segunda legível com LF, embora nenhuma peça saída JSON. Ambas precisam produzir exatamente o mesmo valor de erro invalid_arguments, exit 2 e stderr vazio, **JSON legível com newline quando não há flag real**. Um `--json` real continua selecionando JCS compacto sem newline. O formato não pode depender da grafia equivalente da query.

Não proibir valores hifenizados, reinterpretar query como opção, exigir `=` como workaround ou alterar o contrato. Não corrigir Foundation/D0 por preferência nem mudar comandos/valores de seus erros. O mecanismo escolhido deve distinguir opção de valor nas formas realmente suportadas; evitar uma varredura que volte a tratar qualquer token homônimo como flag. Nenhuma leitura de arquivo é necessária para resolver o erro de parsing.

Matriz mínima, sempre cobrindo **as duas** formas `--query --json` / `--query=--json`:

| Controle | Esperado |
|---|---|
| `--max-matches bad`, sem flag JSON real | Erro invalid_arguments/2, stdout legível byte a byte e newline; stderr vazio. |
| Mesmo erro, flag real `--json` antes de `search` | Mesmo valor/exit, compacto JCS sem LF. |
| Mesmo erro, flag real após `search` e antes de query | Mesmo valor/exit, compacto JCS sem LF. |
| Mesmo erro, flag real depois de `--max-matches bad` | Mesmo valor/exit, compacto JCS sem LF; não confundir com o valor consumido por query. |
| Falta valor de `--snapshot`, seguido por flag real `--json`, com query comum | Erro invalid_arguments/2 compacto, preservando o comportamento; opções de caminho não herdam allow_hyphen_values da query. |
| Query válida sem erro de parsing, com/sem flag real | Query permanece literal e modo respeita a flag; fluxo/código do domínio permanecem os previstos, inclusive fonte sintética ausente. |

Manter controles de query comum e queries `d0/init/search`, help/version e comandos anteriores. Os erros do teste devem ser de parsing, não setup, timeout ou acesso operacional. Acrescentar regressão permanente na suíte CLI com identificação BUG-005; a evidência pública nova deve falhar no binário anterior pela comparação de formato específica e passar no corrigido. Controles válidos e com flag real não podem ser classificados como falha esperada.

## Escrita fechada

**Dois existentes apenas:**

- `crates/compass-cli/src/main.rs`: correção mínima da decisão do transporte após erro, preservando os demais fluxos.
- `crates/compass-cli/tests/search.rs`: regressão permanente BUG-005 e controles relacionados, sem remover testes.

**Cinco novos apenas**, em pasta separada para preservar a entrega original:

- `verification/rd-c01/check.py`: controles públicos BUG-005 e orquestração das campanhas intactas em cópia temporária; biblioteca padrão, sem importação de produção.
- `verification/rd-c01/integration.json`: matriz antes/depois com argv/exits/stdout/stderr brutos/hash, asserção específica, vínculos RD22/RD24/testes; incluir campanha RD e resource-profile completos obtidos da cópia, distinguindo-os dos controles novos.
- `verification/rd-c01/regressions.json`: regressões Foundation/D0 completas da cópia, vinculadas ao binário novo.
- `verification/rd-c01/provenance.json`: baseline/deltas/preservação, comandos/logs, inputs efetivos, versões/binários e hashes dos artefatos de correção.
- `verification/rd-c01/RESULTADO-RD-I1-C01.md`: retorno por AC e evidências, sem regravar resultado histórico.

Todos os demais arquivos protegidos: search.rs/núcleo, D0 inteiro, lib/build/Cargo/toolchain, README, testes anteriores fora do arquivo listado, **verification/rd inteira**, V1/I1/R1/Foundation, contratos/OS/planos/auditorias, estado/coordenação e documento principal. Não editar fixture, verificador/cobertura/evidência histórica para incluir artificialmente o caso novo. O mapeamento corretivo fica na integration.json nova.

Targets/build release são artefatos gerados pelos comandos previstos; preservar o executável anterior em /tmp antes de substituí-lo. O build novo continua com 41 inputs, dos quais somente os dois arquivos autorizados em crates podem mudar. Nenhuma dependência, índice, retenção, serviço, migration ou ação remota.

## Execução e aceites

1. Preservar baseline/binário; criar os controles e regressão. Demonstrar erro específico no executável anterior, com controles válidos, **antes** do patch. Registrar a rejeição como reprodução do defeito, não como sucesso da implementação.
2. Corrigir a CLI e rodar testes focados. Sem refatoração oportunista, novo parser de produto ou mudança de semântica.
3. Finalizar fmt/testes/Clippy/release; capturar identidade. Executar matriz corrigida e campanhas intactas em cópia isolada contra o release novo absoluto. Guardar resultados em rd-c01; preservar todos os JSONs originais.

| AC | Evidência de fechamento |
|---|---|
| C01-AC-01 | Matriz reproduz BUG-005 no executável antigo por asserção de formato específica, incluindo o controle equivalente que já passava; stdout/exit/hash preservados. |
| C01-AC-02 | Duas formas sem flag ficam legíveis com LF; flag real nas três posições previstas produz compacto sem LF; valor/código/exit e stderr permanecem corretos. |
| C01-AC-03 | Teste CLI permanente verifica a mesma falha e controles; valores hifenizados continuam literais e nenhum acesso ocorre para compensar argumento inválido. Comandos e erros anteriores preservados. |
| C01-AC-04 | Testes Rust atuais+regressão, fmt, Clippy e campanhas RD/Foundation/D0 passam sobre release estável. Três novas medidas já produzidas pela campanha RD são preservadas, sem inventar SLO. |
| C01-AC-05 | Somente dois deltas existentes e cinco novos; núcleo/contratos/esperados/verification/rd e demais artefatos históricos intactos. Inputs/produtor/binário atuais conferidos; novo retorno não se atribui ao release anterior. |
| C01-AC-06 | Retorno por AC, prova antes/depois, cobertura BUG-005→RD22/RD24, logs/casos brutos e limitações. Termina pendente de conferência; não fecha BUG-005 pelo executor. |

Comandos existentes, na raiz, após finalizar a alteração:

```sh
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Interface nova de check.py, **a entregar**:

```sh
python3 verification/rd-c01/check.py --control-only --binary <copia-do-binario-anterior>
python3 verification/rd-c01/check.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
```

Control-only verifica a matriz e registra evidência, sem suites históricas. No binário antigo deve retornar não zero **pelo BUG-005**, preservando diagnóstico; no novo, zero. O modo completo executa controles e copia o laboratório RD/fonte de montagem histórica para TemporaryDirectory, rodando acceptance.py --binary e --regressions com o binário corrigido absoluto. Pode copiar o workspace excluindo .git/target/bytecode, ou o subconjunto autenticado suficiente, sem alterar o verificador original nem seu destino HERE na origem. Os outputs da cópia são incorporados aos três JSONs corretivos autorizados; não criar arquivos adicionais no projeto.

O laboratório completo já inclui 151 comandos RD, cinco mutações, três medições e regressões de 23 Foundation/3 mutações + 253 verificações V1/11 mutações D0. Números históricos não substituem os resultados reais; nenhum teste pode ser retirado para conservar contagem. Desabilitar bytecode Python. Socket Unix local pode precisar de permissão técnica; reportar restrição/skip sem aprovação fictícia. Não usar R1 congelada ao binário antigo nem adapter.run sobre a origem, que sobrescreveria evidências.

## Retorno, bloqueio e reversão

Retorno em verification/rd-c01/RESULTADO-RD-I1-C01.md, **IMPLEMENTADO_PENDENTE_VERIFICACAO**: AC→evidência, deltas, comandos, erros de desenvolvimento versus falhas esperadas, antes/depois e hashes completos. Excluir autorreferências circulares da proveniência; informar hashes finais do resultado e proveniência na mensagem externa.

Regressão fora do transporte ou necessidade de tocar arquivo protegido: bloqueio específico para coordenação, sem patch livre. Duas tentativas falhas do mesmo achado exigem reinvestigação antes de terceiro patch. Reversão só dos dois deltas e saídas próprias, mediante cópias/diff; sem reset/clean. Não iniciar a próxima capacidade, enviar mensagens a outros chats ou declarar aceite RD. A revisão posterior confere o mesmo BUG-005/AC-07 e regressões afetadas, sem reiniciar auditoria irrestrita.
