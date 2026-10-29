# OS-D0-R1 — integração final e recursos do sensor D0

```yaml
id: OS-D0-R1
versao: 1
estado: LIBERADA_PARA_EXECUCAO
porte: padrao
origem: plano D0; V1 aceita; I1-C01 auditada CONFORME, BUG-004 verificado
objetivo: entregar fluxo reproduzível pela CLI, medição real de recursos e evidências consolidadas para aceite
dependencias: baseline I1-C01 e contrato operacional de aceite R1
execucao: sequencial; nova conversa; GPT-6.1 Sol / Alto; um escritor
autorizacoes_remotas: nenhuma
```

19/10/2026. Diretório: `/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation`.
Intenção: execução de laboratório/verificação. Produção somente leitura, sem mudanças de contrato do sensor, operações externas ou acervos reais. O porte padrão é proporcional a essa fronteira; o trabalho terá conferência independente posterior dos artefatos. Esta ordem não declara R1 já executada.

## Autoridade e baseline

O [CONTRATO-ACEITE-D0-R1.md](CONTRATO-ACEITE-D0-R1.md), versão 1, faz parte desta OS: define cargas, contas, negativos, medição e herança de evidência. Seus aceites operacionalizam o contrato D0 ativo; não alteram P1–P4 ou limites/schemas.

Ler `AGENTS.md`, `COMPASS_STATE.md`, [ativação](ATIVACAO.md), contrato operacional e [auditoria I1-C01](../../../research/audits/AUDITORIA-OS-D0-I1-C01.md). Consultar contrato C2-A1 §§2–8, schemas, V1/coverage/ACCEPTANCE, resultado I1 e seus JSONs, `run-v1.py`, `build.rs` e `verification/reference/profile.py` conforme necessário. O profile antigo mede Foundation com bytes binários e não serve como medição D0 sem adaptação em ferramenta separada.

| Artefato fixado | SHA-256 |
|---|---|
| Auditoria I1-C01 CONFORME | `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3` |
| Resultado I1 com C01 | `1d19a188de9b44170cb6655752dc5554f7787a027692370b1568ae4b3e906ebf` |
| V1 acceptance executada em C01 | `f3ccbaf7818b86ef2d05fd427405a99c0bd5798586b4d1b2fd07ab840d71b9cb` |
| V1 mutations executada em C01 | `7dc51a4b6204d65fed8136082901c18530feb9102eebd16e1d214cda7e84741d` |
| Adaptador I1 `run-v1.py` | `94d9ada79741fe38121e2884ee964394ba2c7073f1a2805234f5d51bb2c13298` |
| Matriz histórica V1 | `0ed2fc8894852b95c1e55c2cc2609f09f3714db51e64cc0dbc992c9e48afdba3` |

Binário aprovado: `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f`; produtor: `sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`. Os 37 inputs e a política default devem reproduzir a identidade já auditada. Conferir Git/diff e inventário antes de escrever; branch main sem HEAD. Salvar cópias/diff/hash em `/tmp`, sem reset/clean ou sobrescrita de trabalho existente.

## Escrita autorizada — lista fechada

Somente estes **nove arquivos novos**:

- `verification/d0-r1/run.py`: runner sequencial, gerador sintético, empacotador de laboratório, verificações e coleta.
- `verification/d0-r1/fixtures.json`: receitas e esperados independentes do contrato, definidos antes do uso do binário.
- `verification/d0-r1/inputs.json`: bytes exatos de manifestos/descritores dos dois casos normais, em base64 + hash; variantes negativas identificadas separadamente se necessárias para reexecução.
- `verification/d0-r1/reports.json`: stdout representativo completo, em base64 + hash, exit e ID do cenário; distinguir relatório D0, comparação Foundation e envelope de erro.
- `verification/d0-r1/integration.json`: passos/comandos, asserções específicas, casos positivos/negativos, repetibilidade sem árvore viva, falhas e modalidade real.
- `verification/d0-r1/resource-profile.json`: seis amostras válidas, métricas brutas/unidades, ambiente e custo de disco, separados do ledger lógico.
- `verification/d0-r1/coverage.json`: matriz E01–E55 e provas internas/BUG-004, com estados de herança/execução e referências rastreáveis.
- `verification/d0-r1/provenance.json`: inventário inicial/final, 37 inputs/hashes, produtor/binário, hashes do runner/fixtures e dos artefatos finais, versão da OS/contrato e evidências herdadas.
- `verification/d0-r1/RESULTADO-D0-R1.md`: retorno por AC, evidências, limites e pendências para conferência final.

Os demais arquivos são protegidos: Rust/CLI/Cargo/toolchain/build/binário, V1 inteira, I1 e sua baseline/adaptador/JSONs/resultado, todos os contratos/OS/fixtures/auditorias anteriores, evidências Foundation, README, documento principal e documentos centrais. Não editar nem instalar dependências. Nenhum novo arquivo fora da lista, schema D0, API, flag de produção ou captura automática de conteúdo.

Temporários e logs podem usar `/tmp`. O runner só cria/muta/remove acervos/pacotes artificiais sob o TemporaryDirectory que ele próprio criou e inventariou. Não aceitar raiz fornecida pelo operador, não tocar outros temporários ou acervos existentes. Se algum arquivo de saída já existir, conferir e preservar sua versão anterior antes de nova execução; não apagar silenciosamente evidência anterior.

## Trabalho sequencial

1. **Fixar proveniência e fixtures.** Conferir hashes, integridade C2-A1 e os 37 inputs; registrar que os testes auditados serão herdados. Implementar receitas/literais do contrato e self-test. Bibliotecas padrão e verificador V1/jsonschema já disponível são suficientes; cache local opcional, sem instalação/rede.
2. **Fluxo público.** Usar o binário fixado e `init/snapshot/compare/d0`; preparar pacotes explicitamente em temporários, copiar manifestos brutos e conferir binding. A saída D0 deve ser validada por literais/equações e verificador independente existente por leitura, sem importar Rust. Conferir hashes do conjunto congelado; não chamar a proteção dos 34 fontes históricos contra a árvore I1 atual como se devessem ser idênticos. O adaptador/ambiente histórico já entregue pode ser usado por leitura quando necessário; não alterá-lo.
3. **Medição e controles.** Executar M/L e três amostras D0 por carga; coletar GNU time do filho nativo e wall. Executar a retirada da árvore viva artificial e os quatro negativos de L, com restauração e controle válido. Não confundir indisponibilidade contratada com falha do laboratório.
4. **Consolidar.** Guardar inputs/outputs brutos necessários, métricas, matriz de evidências e retorno. Validar vínculos/hashes e preservação final. A proveniência exclui seu próprio hash e o resultado textual ainda em escrita; registrar hashes finais desses arquivos no retorno externo, sem autorreferência circular.

Usar os resultados auditados de C01 sem repetir indiscriminadamente 64 testes/build/253 verificações. Se uma divergência de identidade ou lacuna exigir repetição, registrar por quê, comando, versão e resultado. Não recompilar/substituir o binário para contornar baseline divergente. Testes adicionais proporcionais podem usar temporários, sem ampliar a lista de arquivos de projeto.

## Aceites

| ID | Dado / quando / então |
|---|---|
| R1-AC-01 | Git/inventário e hashes confirmam baseline C01 antes/depois, nenhuma escrita em produção ou artefatos protegidos; executável estável, 37 inputs e produtor emitido coincidem. Nove saídas pertencem à lista; corpo do runner/fixtures fica identificado. |
| R1-AC-02 | Self-test confere receitas, comprimento/linhas, contas de M/L, frações/decimais e recursos contra literais independentes, sem binário. Controle válido passa e cópia temporária de esperado adulterado é rejeitada por asserção específica. Fixture permanece intacta durante runtime; nenhum esperado é capturado de D0. |
| R1-AC-03 | Fluxos públicos geram snapshots reais e pacotes vinculados; comparison/reports/exits/checks/cálculos/recursos correspondem às duas cargas e ao contrato. Sem árvore viva, D0 repete os mesmos bytes; manifestos externos/pacotes permanecem imutáveis durante cada medição válida. |
| R1-AC-04 | Os quatro controles negativos de L atingem seus motivos/envelopes/exits exatos; sem cálculo/integral indevido, preservando current verificado quando aplicável. Restauração recupera o resultado válido. Falha genérica, setup ou timeout não conta como rejeição correta. |
| R1-AC-05 | Há três amostras válidas por carga, RSS individual medido por processo nativo, tempo/CPU/unidades brutos, cache/ambiente/dados/build identificados. Bytes de objeto/células/ledger e disco são conferidos separadamente. Não há SLO inventado nem RSS substituído por memória lógica. |
| R1-AC-06 | Todos E01–E55 têm vínculos a evidências executadas ou herdadas com hashes/versão, sem afirmar reexecução. E35/E43/E46/E49 e BUG-004 estão reconciliados; RSS tem evidência nova. Pendência obrigatória ausente ou NAO_VERIFICADO impede aceite; HIP-I1-01 não é promovida a obrigação. |
| R1-AC-07 | Inputs/outputs representativos são preservados byte a byte, com hashes e IDs; resultados distinguem self-test/runtime, exit esperado e erro de laboratório. Retorno por AC não declara R1 concluída ou corrige produção por conta própria; identifica limites e entrega para conferência independente. |

## Interface e comandos

Os dois comandos do runner são **novos, a entregar**, não ferramentas já existentes:

```sh
python3 verification/d0-r1/run.py --self-test
python3 verification/d0-r1/run.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0
(cd research/proposals/d0-c2 && sha256sum --check --quiet SHA256SUMS)
git diff --check
```

O self-test não invoca produção nem publica métricas runtime fictícias; se escrever integration.json, marcar modalidade self-test/runtime_executed=false. O modo binary executa os fluxos/medidas e registra falhas com retorno não zero. Nenhum modo termina com aprovação se faltarem amostras/casos obrigatórios. A opção de cache pode ser omitida quando as dependências de verificação já estiverem disponíveis. `/usr/bin/time` com as flags do contrato foi encontrado nesta máquina; recurso ausente em outra deve ser relatado, sem instalação ou métrica fabricada.

Se precisar executar testes herdados que criam socket Unix, usar permissão técnica pelo mecanismo da ferramenta ou registrar restrição. Isso não autoriza rede/acervos reais. Guardas do laboratório e deadline D0 têm significados distintos.

## Retorno, bloqueios e reversão

Retorno em `verification/d0-r1/RESULTADO-D0-R1.md`, estado IMPLEMENTADO_PENDENTE_VERIFICACAO: AC→evidência, arquivos escritos, comandos realmente executados vs provas herdadas, identidades completas, seis amostras, negativos, limites e itens de conferência. Preservar mensagens/erros relevantes sem credenciais; dados do laboratório são sintéticos.

Nova falha de produto gera achado/reprodução para coordenação; esta ordem não autoriza patch de Rust, contrato ou esperado. Identidade incompatível, dependência/arquivo fora da lista ou medição inviável exige bloqueio específico, com evidência. Duas falhas da mesma abordagem pedem reinvestigação antes de novo patch automático no laboratório.

Reversão limitada às nove saídas próprias depois de conferir deltas/cópias; limpeza somente dos temporários registrados pelo runner. Sem deploy, publicação, migration, Git commit, operação remota ou mensagem a outro chat. Após o retorno haverá conferência independente da R1 e registro de aceite pela coordenação; nenhuma próxima capacidade de produto está autorizada por esta OS.
