# OS-RD-I2 — leitura de contexto da revisão declarada

```yaml
id: OS-RD-I2
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: critico
origem: recuperação direta 6.3/17.3; busca RD/C01 aceita
objetivo: abrir faixas de linhas verificadas ao redor de um resultado de search, com continuação explícita
execucao: sequencial; nova conversa; GPT-6.1 Sol / Alto; um escritor
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.

## Autoridade e baseline

[CONTRATO-RD-LEITURA-V0.1.md](CONTRATO-RD-LEITURA-V0.1.md), versão 1, integra esta OS: interface read, seleção/RL01–RL16, estados, bytes e recursos. Busca já localiza linhas; read permite obter a vizinhança e continuar na mesma revisão. Não cria um registro abstrato de manutenção ou índice sem consumidor. Nenhuma escolha de modelo/banco/ranking necessária.

Porte crítico por selecionar/entregar conteúdo; auditoria prévia de contrato antes de liberar e auditoria de entrega posterior. Ler AGENTS.md/COMPASS_STATE.md, este contrato, plano RD e [auditoria C01](../../../research/audits/AUDITORIA-OS-RD-I1-C01.md). Consultar contrato RD §§1–5, modelo/codec, search e D0 input/source/resources apenas no necessário. Documento principal §§6.3/7/17.3 dá a direção; legado não é fonte de código ou instruções.

Main sem HEAD. Inspecionar Git/diff e inventário antes de escrever; salvar cópias/hash/deltas/binário aceito em /tmp, sem reset/clean. Baseline aceita:

- Release `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8`.
- Produtor dos 41 inputs `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`.
- Parecer C01 `1dd71986c3e533b34c2fc739d66322b858dfffaba917668afd6c37a6e9f6d9f8`.
- Contrato busca RD `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72`.
- Núcleo search antes da visibilidade `04a8e696d7fd3739419872aa5b7ed7760f7c9d5e1cf654fcb7167c968f5f65aa`.
- Checker C01 `5e15e4df2d6a3d5610cdb274880c5877d28b8ac02f9bf084e8282852177e85cd`.

O contrato de leitura ainda não é input do build antes da execução. Incluir explicitamente no build novo; coletar hashes depois de finalizar fontes/testes/formatação. Inputs esperados: 45 (41 atuais + módulo/testes core/testes CLI/contrato novos), mantendo framing existente. Não atribuir provas C01 ao binário novo.

## Escrita fechada

Existentes:

| Caminho | Delta autorizado |
|---|---|
| `crates/compass-core/src/lib.rs` | Exportar módulo read. |
| `crates/compass-core/src/search.rs` | **Somente visibilidade crate** de Limits.inner e helpers manifest/text_reason/line_preflight para reuso; corpos, algoritmos, testes e saída search idênticos. |
| `crates/compass-core/build.rs` | Acrescentar explicitamente o novo contrato de leitura à coleta; manter demais inputs/framing. |
| `crates/compass-cli/src/main.rs` | Subcomando read fino, argumentos/sinal/transporte; distinguir valor de path-key de flag real nos erros, preservando a correção search BUG-005 e demais comandos. |
| `README.md` | Uso search→read/continuação, revisão declarada, limites e semântica do EOF/indisponibilidade. |

Novos, somente:

- `crates/compass-core/src/read.rs`: operação pública, seleção/relatório e testes internos de barreira/checkpoints; reutilizar leitor/contexto/limits e helpers existentes.
- `crates/compass-core/tests/read.rs`: contrato público/imutabilidade/seleção sem overflow.
- `crates/compass-cli/tests/read.rs`: CLI/erros/transporte e preservação de valores hifenizados.
- `verification/rd-i2/fixtures.json`: gabaritos RL01–RL16 antes de Rust.
- `verification/rd-i2/acceptance.py`: oráculo/validador/campanha e coleta independente, biblioteca padrão.
- `verification/rd-i2/coverage.json`: RL01–RL16 ↔ cenários internos/CLI realmente executáveis.
- `verification/rd-i2/integration.json`: controles/mutações/inputs/outputs brutos representativos base64+SHA, exits e vínculos.
- `verification/rd-i2/resource-profile.json`: três amostras read e ambiente/disco/ledger separados de RSS.
- `verification/rd-i2/regressions.json`: check C01 completo e seus outputs RD/Foundation/D0, em cópia, contra release novo.
- `verification/rd-i2/provenance.json`: baseline/deltas/inputs/binário/ferramenta/fixture e logs/hashes completos.
- `verification/rd-i2/RESULTADO-RD-I2.md`: retorno por AC, comandos, evidências/limitações.

Todos os demais arquivos protegidos: Cargo/lock/toolchain, D0/codec/model/storage/observe/compare, testes antigos, todos os contratos/OS/auditorias/planos/estado/coordenação, **verification/rd e rd-c01 inteiras**, demais laboratórios/evidências e documento principal. Não instalar dependência, criar migração/índice/retentor/serviço ou tocar acervo real. Nova necessidade fora da lista é BLOQUEIO_DE_FRONTEIRA fundamentado; não copiar/enfraquecer o leitor seguro ou mudar gabarito para acomodar resultado.

Temporários/targets/logs/cópias em /tmp; testes só em árvores sintéticas próprias e registradas. Desabilitar bytecode Python. Outputs anteriores devem ser preservados em cópia antes de reexecução. Build release pode substituir o artefato gerado somente após salvar o anterior.

## Sequência, aceites e verificações

Uma conversa/um escritor: (1) fixtures/oráculo/self-test independente; (2) núcleo/read+CLI e testes; (3) fmt/Clippy/testes/release estável, integração/medidas/regressão. Não criar fase documental adicional nem executar capacidades posteriores. A auditoria independente acontece depois sobre o release final.

| AC | Comportamento observável |
|---|---|
| I2-AC-01 | Lista de escrita, baseline e preservação conferidas; search.rs tem só visibilidade; contrato novo incluído no build, 45 inputs atuais autenticados. |
| I2-AC-02 | Fixtures antes da produção; offsets/linhas/EOF/fronteiras derivados de bytes literais e fórmulas herdadas; self-test sem sensor rejeita esperado adulterado especificamente. |
| I2-AC-03 | Read entrega somente revisão observed selecionada, texto/offsets/linhas/next exatos; vazio/além EOF/u64::MAX sem overflow; Unicode/BOM/CRLF/LF-only preservados (RL01–RL04). |
| I2-AC-04 | Binding validado mesmo em universo vazio/key não observed; arquivo não selecionado não é lido; invalid/excluded/unknown não viram busca por fallback ou prova de remoção (RL05–RL07). |
| I2-AC-05 | Falhas, adulteração, tipos e corrida sincronizada impedem excerpt; textualidade/linha longa tardia respeitam precedências/checks e nulls; controle estável passa (RL08–RL10). |
| I2-AC-06 | Limites S/W/reservas/EOF, campos/u64/output e cancelamento/deadline conferidos; parser distingue valor de flag, sem regressão BUG-005 (RL11–RL13). |
| I2-AC-07 | Fluxo search→read→continuação, árvore viva ausente, S0/S1 e entradas imutáveis; três medidas nativas só de read da carga definida, sem SLO (RL14–RL15). |
| I2-AC-08 | Controle válido e mutações de text/offset/revisão/continuação/disponibilidade atingem asserções específicas; RL01–RL16 correspondem a testes/cenários reais (RL16). |
| I2-AC-09 | Testes atuais+novos/fmt/Clippy; check C01 intacto 48 controles + RD151/cinco mutações + Foundation23/três + V1 253/11 passam no release novo estável, sem regravar evidência antiga. |
| I2-AC-10 | Retorno/proveniência/deltas/raw bytes, comando real versus prova interna, limitações/preservação completos; ausência de prova obrigatória/skip/restrição não vira aceite. |

Comandos existentes, na raiz:

```sh
scripts/cargo-local.sh fmt --all -- --check
scripts/cargo-local.sh test --workspace --locked --offline
scripts/cargo-local.sh clippy --workspace --all-targets --locked --offline -- -D warnings
scripts/cargo-local.sh build --release --workspace --locked --offline
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

Interface nova de acceptance.py, **a entregar**:

```sh
python3 verification/rd-i2/acceptance.py --self-test
python3 verification/rd-i2/acceptance.py --binary target/release/compass
python3 verification/rd-i2/acceptance.py --regressions --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
```

Self-test sem produção; binary executa RLs públicos/mutações/carga read; regressions copia workspace sem .git/target/bytecode e referências necessárias para TemporaryDirectory e chama **verification/rd-c01/check.py --binary <release novo absoluto>** com cache. Esse checker já executa os 48 controles e as campanhas intactas; não repetir suites/medições RD fora dele sem razão. Incorporar os JSONs/logs completos da cópia em regressions.json, sem alterar o destino HERE das ferramentas originais nem rodá-las sobre a origem. Os números são referência de não perda, nunca motivo para omitir teste.

Medição read e medições históricas RD lançadas pelo checker têm rótulos/comandos distintos, sequenciais e sem compilação concorrente. Não usar o runner R1 fixado ao binário C01 antigo ou remover seus pins. Testes de socket Unix podem exigir permissão técnica; registrar restrição sem aprovação fictícia, sem rede/instalação.

## Retorno, bloqueios e reversão

Entregar verification/rd-i2/RESULTADO-RD-I2.md como **IMPLEMENTADO_PENDENTE_VERIFICACAO**: AC→RL/teste/evidência, comandos reais, deltas/hashes, interfaces/limitações, três amostras e provas de preservação. Proveniência exclui seu hash e resultado ainda em escrita; informar os dois hashes finais externamente.

Nova falha ou necessidade de contrato/escrita fora da lista: diagnóstico/bloqueio para coordenação, sem patch livre. Duas falhas do mesmo achado exigem reinvestigação. Reversão somente dos deltas autorizados mediante cópias/diff e executável salvo, sem reset/clean. Nenhum commit, dado real, ação remota, outra capacidade ou mensagem a outro chat. Aceite depende da conferência independente posterior.
