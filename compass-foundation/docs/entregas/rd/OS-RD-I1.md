# OS-RD-I1 — primeira recuperação direta pela CLI

```yaml
id: OS-RD-I1
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: critico
origem: pedido para avançar após R1; documento principal 17.3, recuperação direta
objetivo: buscar texto literal e entregar linhas verificadas da revisão declarada
dependencias: Foundation e D0/R1 aceitas; contrato RD v0.1; auditoria prévia do contrato
execucao: sequencial; um escritor; nova conversa; GPT-6.1 Sol / Alto
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.

## Autoridade e risco

O [CONTRATO-RD-V0.1.md](CONTRATO-RD-V0.1.md), versão 1, integra esta OS e fixa interface, matching, bytes/offsets, parcialidade, limites/créditos e RD01–RD24. Nenhuma decisão de ranking, modelo ou backend é necessária para esta busca literal inicial. A escolha de pacotes explicitamente vinculados preserva a revisão verificável já disponível e evita introduzir retenção ou consistência de índice antes de contratá-las.

Porte crítico pela leitura/entrega de conteúdo e reutilização da fronteira segura D0. Auditoria prévia `auditor-red-team` modo contrato antes de liberar; depois da execução, auditoria de entrega. Não iniciar implementação enquanto houver achado obrigatório aberto ou estado não liberado. Não exige nova aprovação de produto para corrigir inconsistências técnicas fundamentadas no escopo.

Ler AGENTS.md, COMPASS_STATE.md, contrato RD, plano RD, documento principal §§6.3/7/13/17.3 e auditoria R1. Consultar Foundation/codec/model, D0 input/source/resources/types e CLI somente nas dependências pertinentes. Não ler/importar o legado nem reabrir as decisões P1–P4.

## Baseline anterior à implementação

Branch main sem HEAD; inspecionar Git/diff, inventariar fontes, documentos, evidências, não rastreados e executável antes de escrever. Não reset/clean. Versão aceita:

- Binário C01/R1: `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`.
- Produtor de 37 inputs: `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`.
- Parecer R1: `3c49a7af32ccfda6ab258cdd5d1d086b751dd2a0c86fc28ab2caae5df93434a9`.
- Índice C2-A1: `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e`, 81 entradas.

Salvar diff/cópias/hash em /tmp, inclusive binário aceito antes do build. Os novos documentos RD não integram os 37 inputs históricos; build.rs passará a incluir o contrato RD expressamente. Nenhum hash futuro é predefinido. Finalizar fontes, testes e formatação antes de recompor inputs e identificar o release novo.

## Lista fechada de escrita

**Existentes**, apenas alterações delimitadas:

| Arquivo | Alteração autorizada |
|---|---|
| `crates/compass-core/src/lib.rs` | Exportar módulo search. |
| `crates/compass-core/src/d0/mod.rs` | Somente visibilidade crate de input/resources/source para reutilização; preservar funções/testes e comportamento D0. |
| `crates/compass-core/build.rs` | Acrescentar explicitamente `docs/entregas/rd/CONTRATO-RD-V0.1.md` à coleta, mantendo domínio/framing e demais inputs. |
| `crates/compass-cli/src/main.rs` | Subcomando search fino, argumentos/erros/saída/sinal; preservar comandos existentes e seus transportes. |
| `README.md` | Uso real de search, pacote/revisão, limites, cobertura e seleção limitada, exemplo sintético e limitações. |

**Novos**, somente estes:

- `crates/compass-core/src/search.rs`: domínio RD, limites, crédito, matching/linhas, relatório e testes internos com barreiras/checkpoints.
- `crates/compass-core/tests/search.rs`: contrato público do núcleo e preservação de entradas.
- `crates/compass-cli/tests/search.rs`: roteamento/erros e fluxo CLI real, Unicode, parcialidade e repetibilidade.
- `verification/rd/fixtures.json`: esperados independentes RD01–RD24, fixados antes da implementação Rust.
- `verification/rd/acceptance.py`: oráculo/validador independente, campanhas, mutações específicas e coleta; biblioteca padrão, reuso V1 por leitura quando pertinente.
- `verification/rd/coverage.json`: RD01–RD24 ↔ testes/cenários executáveis, distinguindo CLI/interno/pendente.
- `verification/rd/integration.json`: execução RD, assertivas, negativos/mutações, inputs/outputs brutos representativos em base64+hash e identidade.
- `verification/rd/resource-profile.json`: três amostras nativas RD, máquina/limites/disco/cache, ledger separado de RSS.
- `verification/rd/regressions.json`: resultados/logs brutos Foundation e D0 contra o novo binário; JSONs D0 completos aninhados, sem sobrescrever I1/V1.
- `verification/rd/provenance.json`: inventário/deltas, hashes de contrato/fixtures/ferramenta, inputs efetivos e binário antes/depois das campanhas, comandos e limitações.
- `verification/rd/RESULTADO-RD-I1.md`: AC→evidência, resultados e retorno para auditoria.

Demais arquivos protegidos: Cargo manifests/lock/toolchain, .gitignore, Foundation/model/storage/observe/codec, D0 input/source/resources/types/metric/report, todos os testes antigos, contratos/schemas/gabaritos/ativação/OS/auditorias e verification/d0, d0-i1, d0-r1, reference/evidence. Também estado/coordenação/planos/contrato RD e documento principal. Não instalar dependências, criar banco/índice/capturador/daemon, mudar flags D0 ou publicar em rede. Nova dependência/escrita necessária fora da lista gera BLOQUEIO_DE_FRONTEIRA fundamentado; não usar a lista para enfraquecer requisito ou copiar o leitor seguro.

Temporários/logs/cópias/targets auxiliares em /tmp. O laboratório cria e limpa somente suas árvores/pacotes sintéticos registrados. Não usar acervos reais. Preservar cópias de outputs existentes antes de reexecução. Desabilitar bytecode Python para não acrescentar arquivos fora da lista.

## Execução em três fases, na mesma conversa

1. **Contrato para fixtures.** Escrever/verificar os esperados de RD01–RD24 e self-test independente antes da lógica Rust; contas de bytes/linhas/limites e controle de esperado adulterado por asserção específica. Essa fase produz ferramenta utilizável, não outra proposta documental.
2. **Núcleo e CLI.** Reutilizar o leitor e validação existentes via visibilidade mínima. RD faz preflight de manifesto único e reserva de resposta conforme seu contrato, sem chamar `d0::measure` com dois snapshots artificiais. Implementar busca/offsets/relatório, erros e checkpoints; CLI usa o mesmo núcleo. Testes sincronizados verificam source_unstable e cancelamento; sem flags públicas de teste. Não alterar D0 para acomodar o novo módulo.
3. **Integração e release.** Fmt/Clippy/testes, release estável com identidade nova; RD pela CLI e sem árvore viva, três medições nativas. Rodar regressão Foundation/D0 pertinente **uma vez no release final**, consolidar evidências e preservação. Falha ou mudança posterior relevante exige repetir apenas os checks afetados antes de fechar a identidade final.

Não há implementação em paralelo nesta fatia: testes novos, CLI e núcleo compartilham contrato/identidade. Auditoria posterior trabalha em cópia isolada. O executor não cria chats nem envia mensagens a outros chats.

## Aceites

| ID | Comportamento observável |
|---|---|
| RD-AC-01 | Lista de escrita respeitada; baseline autenticada; contrato RD incluído no novo build; entradas/artefatos protegidos preservados. Deltas de D0/mod.rs são apenas visibilidade. |
| RD-AC-02 | Fixtures derivadas do contrato antes da produção; self-test confere bytes/offsets/contas/limites e rejeita esperado adulterado especificamente. Validador independente não importa Rust nem copia resultados. |
| RD-AC-03 | CLI search chama o núcleo e entrega linhas/offsets/revisões exatos, literal sensível a caixa, um hit por linha, ordem nativa e UTF-8/BOM/CRLF preservados; RD01–RD08. |
| RD-AC-04 | Ausência/adulteração/links/tipos/instabilidade/binding recebem motivos/exits exatos, sem leitura externa nem texto não verificado. Corridas com barreiras e controles estáveis; RD09–RD13/21/23. |
| RD-AC-05 | Cobertura e seleção têm semânticas distintas. Continua após max_matches; falha tardia aparece. Parciais/vazios/non_text/limites não viram ausência completa; RD03/04/08/09/13/16. |
| RD-AC-06 | Limites/aritmética/créditos/reservas/EOF e memória S/W conferidos nos dois lados das fronteiras. Query/linha são contadas em bytes. SIGINT/deadline têm prova sincronizada inclusive no scan sem LF/hits; RD14–RD19/22. |
| RD-AC-07 | JSON estrito/JCS/campos/u64/erros/roteamento corretos; cinco mutações RD24 atingem trecho, offsets, revisão, cobertura e truncamento separadamente, com controle válido. Falha genérica não conta. |
| RD-AC-08 | Fluxo init→snapshot→pacote→search pela CLI, com ausência da árvore viva, igualdade de stdout e nenhuma escrita em entrada; três amostras da carga textual, RSS nativo separado do ledger, sem SLO. |
| RD-AC-09 | Testes Rust atuais+novos, fmt, Clippy e campanhas Foundation/D0 passam contra o novo release estável; nenhum resultado do release C01 é apresentado como execução do binário RD. Sem alterações dos gabaritos ou sobrescrita de evidências antigas. |
| RD-AC-10 | Matriz RD01–RD24 corresponde aos cenários/testes reais, AC→evidência rastreável, inputs/outputs representativos recuperáveis e proveniência completa. Pendência obrigatória/skip/restrição impede aceite, não vira aprovação. |

## Comandos e regressão histórica

Existentes, na raiz:

```sh
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
python3 verification/reference/acceptance.py --binary target/release/compass
python3 verification/reference/mutations.py --binary target/release/compass
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Interfaces novas da ferramenta **a entregar**:

```sh
python3 verification/rd/acceptance.py --self-test
python3 verification/rd/acceptance.py --binary target/release/compass
python3 verification/rd/acceptance.py --regressions --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
```

Modo self-test não invoca produção nem gera falso runtime aprovado. Modo binary executa RD, positivos/negativos/mutações e três medições. Modo regressions executa Foundation e V1/D0 intactos em cópia temporária, guarda resultados/logs em regressions.json e exige binário estável. Não importar/rodar produção Python.

Para D0, reutilizar por leitura `verification/d0-i1/run-v1.py` **verify/mount** com o tar histórico autenticado; executar acceptance.py/mutations.py da V1 montada contra o executável RD absoluto. Não chamar adapter.run sobre o projeto original, pois ele sobrescreve JSONs I1. Preservar JSONs completos produzidos pelo laboratório em regressions.json. Referências atuais: 253 verificações V1/119 cenários, 11 mutações D0, 23 testes Foundation/3 mutações e 64 Rust, acrescidos dos novos; números são conferência de não perda, não motivo para remover casos. Foundation também tem seus testes Rust no workspace.

Não executar o runner R1 congelado contra o novo binário: ele fixa a identidade C01 por definição. Nenhuma falha de fingerprint autoriza remover seu pin. Regressão D0 da nova versão vem de V1 intacta + testes Rust; relatórios/medidas R1 antigos permanecem históricos. Dependências cacheadas são somente leitura, sem instalação/rede. Socket Unix pode precisar de permissão técnica; restrição/skip deve ser registrada, sem converter em sucesso.

## Retorno e reversão

Estado do executor: **IMPLEMENTADO_PENDENTE_VERIFICACAO** em verification/rd/RESULTADO-RD-I1.md. Informar baseline/deltas, AC→RD/testes, antes/depois de controles, comandos realmente executados, hashes completos de inputs/binário/artefatos, três medições, limitações e pendências. Proveniência exclui seu próprio hash e o resultado ainda em escrita; informar os dois hashes finais na mensagem externa, sem ciclos.

Nova falha de produção ou necessidade de mudar contrato gera achado/bloqueio específico; não ajustar esperado. Duas falhas do mesmo achado exigem reinvestigação antes de terceiro patch. Reversão somente dos deltas desta lista, mediante cópias/diff; manter binário C01 salvo e artefatos históricos. Sem reset/clean, commit, migrations, rede, dados reais ou próxima capacidade. A coordenação confere a entrega e decide aceite antes de liberar o consumidor seguinte.
