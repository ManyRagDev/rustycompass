# Auditoria de entrega — OS-D0-R1 v1

**Veredito: CONFORME. R1-AC-01–07 atendidos; nenhum achado obrigatório confirmado.** A conferência independente permite à coordenação aceitar R1 e encerrar a entrega técnica D0 v0.1 no escopo contratado. Isso não representa conclusão do aplicativo ou validação das capacidades posteriores.

19/10/2026, America/Sao_Paulo. Skill `auditor-red-team` v2.0, modo `entrega`. Critérios preservados: [OS-D0-R1](../../docs/entregas/d0/OS-D0-R1.md) e [contrato de aceite](../../docs/entregas/d0/CONTRATO-ACEITE-D0-R1.md), ambos v1. Examinados [resultado](../../verification/d0-r1/RESULTADO-D0-R1.md), nove artefatos R1, autoridades delimitadas e provas herdadas I1/C01. Nenhum código, contrato, esperado ou teste entregue foi corrigido durante a auditoria.

## Versão e preservação

Branch `main`, sem HEAD/commits (`git rev-parse --verify HEAD`: exit 128). Git/diff e inventário capturados antes das verificações. As adições Foundation e os diretórios não rastreados eram preexistentes; não são atribuídos à R1 nem a um commit inexistente.

| Referência auditada | SHA-256 |
|---|---|
| OS-R1 v1 | `6be86427ba5d3e0473361c4ccb1b0ad04fec87080b8c75c8ffbccc31364aa635` |
| Contrato R1 v1 | `10f6463b91269147d59752ec8799469341d743c60f0d15158879ad840205afa1` |
| Resultado entregue | `4b714d6641069cb10e000a755ce81d918fafbb5e91df29cb2bf0f21211af2f2d` |
| Proveniência entregue | `beb99704a1ecb0f55525aae327d03222025884a7734bae8f6fac5fdcef0e13fb` |
| Runner entregue | `f97882cb6090c7b363b7358068c760d52bf7cf892a72f3a049b8025c872a6a5c` |
| Fixtures entregues | `53f06546adc49940f9b2a54ab83dc3bf271309bcb403c479bcd89d900f3b2b7f` |
| Binário entregue e usado pelo auditor | `7c4e61fd142353733626dcea68192a25b83591ef10f5e8ca42cc55679a5ebd2f` |

Os hashes finais informados pelo executor coincidem. Recompostos independentemente os **37 inputs** e o framing do build: `producer_build=sha256:075d6160b11859329b02ee0557aff64cb42d0d8064c4907090d2b2febdef1676`. Política default e 81 entradas C2-A1 intactas. Não houve rebuild ou substituição do executável.

O mapa inicial do executor, `/tmp/compass-r1-baseline-kmw6vsx5/inventory.json`, tem o SHA declarado `34cb0dc32a09db6c684d466a59fcc87631cdf8a7f6bebabd3d6004e9208c2a67` e corresponde integralmente ao embutido na proveniência. Seus **165 arquivos protegidos** conferiram com os bytes atuais; os três inventários são iguais. A diferença do conjunto inicial para a entrega é exatamente a lista de nove arquivos R1. O inventário próprio do auditor inclui **172 arquivos locais, contando o executável e as nove entregas**; as duas referências externas também foram autenticadas pelo mapa do executor.

## Conferência independente

Execução em cópia isolada sob `/tmp/compass-audit-r1-dqf4grfy/replica/compass-foundation`, preservando também a baseline histórica e as referências externas. O runner e os testes entregues permaneceram sem alterações. A campanha completa terminou com exit 0: **24 comandos CLI, M/L, retirada das duas árvores vivas, quatro negativos, quatro restaurações e seis novas medições nativas**. A execução do auditor não sobrescreveu os JSONs originais.

Uma segunda ferramenta, `independent.py`, usa somente a biblioteca padrão, sem importar runner, oráculo, validador V1 ou Rust. Ela regenera os conteúdos a partir das receitas contratuais, confere hashes/tamanhos de cada objeto e manifesto, binding e source_ref, ordem nativa, transporte canônico dos casos ASCII, contagens, disponibilidade, cálculos por unidade, agregado e ledger. Para sequências diferentes usa DP independente de LCS. Conferiu também os bytes brutos dos medidores, os 24 exits/vínculos, a proveniência e os pointers/hashes da cobertura. Passou tanto nos artefatos originais quanto nos regenerados pelo auditor: **20.243 conferências de campos/hashes por conjunto**, não contadas como novos testes comportamentais de produção.

Resultados matemáticos confirmados:

- **M:** 1.000 arquivos/4.096.000 bytes por snapshot; 985 unchanged, 10 modified, 5 removed e 5 added. 1.005 unidades; LCS 64/63/0 conforme a classe; **660/128000 = 0.005156**, incluindo unchanged no denominador. Leitura/reserva/células: **8192000 / 8194000 / 40960**.
- **L:** 2.000 linhas por lado, LCS 1.999; a=r=1; **2/4000 = 0.000500**. Leitura/reserva/células: **40000 / 40002 / 4000000**.
- S é calculado com os comprimentos brutos reais e N; peak=S+max(W). Na entrega, **12.477.076 bytes em M** e **407.820 em L**. Esses créditos lógicos não foram confundidos com RSS.

Os quatro negativos atingiram seus resultados específicos: source_missing/not_attempted, object_missing/failed, hash_mismatch/failed com tamanho verdadeiro e hash falso após EOF, e envelope exato incompatible_inputs/exit 2. Nos três relatórios indisponíveis, current permanece verified, calculation está ausente e value/line_mass/zero_basis/known_subset são null. Restaurar cada intervenção recuperou o relatório válido byte a byte.

Além do self-test com esperado C=661 rejeitado por `M.aggregate_numerator`, o auditor apresentou ao validador um relatório L válido e quatro cópias adulteradas, sem chamar produção ou alterar testes:

| Adulteração | Rejeição específica observada |
|---|---|
| Decimal integral 0.000501 | `report_measurement` |
| LCS da unidade alterada para 2000 | `report_units` |
| Células reservadas alteradas para zero | `report_resources` |
| source_ref baseline alterado para 64 zeros | `report_sources` |

O controle válido passou. Nenhuma rejeição genérica, falha de montagem ou timeout foi usada como prova desses controles.

## Resultado por aceite

| Aceite | Veredito e evidência |
|---|---|
| R1-AC-01 | CONFORME. Nove arquivos autorizados; 165 hashes do executor e identidade binário/37 inputs conferidos; nenhum delta de produção. |
| R1-AC-02 | CONFORME. Self-test antes do sensor, fixture estável e negativo específico. Receitas, matemática e recursos conferidos por implementação independente do auditor. |
| R1-AC-03 | CONFORME. Snapshots/compare reais, pacotes observados vinculados aos manifestos brutos, relatórios validados; repetição idêntica sem árvore viva em M/L. Campanha completa reproduzida na cópia. |
| R1-AC-04 | CONFORME. Quatro negativos com motivos/envelopes/exits exatos e quatro restaurações válidas; current verificado e ausência de cálculo/integral indevido. |
| R1-AC-05 | CONFORME. Três amostras nativas por carga reproduzidas; GNU time bruto coerente com os campos, RSS individual, wall/CPU/unidades, ambiente, disco e ledger separados. Sem SLO inventado. |
| R1-AC-06 | CONFORME. 55 IDs com pointers/hashes/testes correspondentes. E35/E43/E46/E49 e BUG-004 reconciliados com os testes permanentes/auditoria da mesma versão; RSS novo identificado separadamente. |
| R1-AC-07 | CONFORME. Cinco registros de input e 14 stdout completos recuperáveis por base64/hash, incluindo negativos/envelope. Retorno distingue execução própria de herança e preservou o estado pendente até esta conferência. |

E01–E55 estão identificados como HERDADO_VERIFICADO, não como 55 novas execuções R1; E46 acrescenta EXECUTADO_R1 para RSS. Os **64 testes Rust, 23 Foundation, 253 verificações V1 e 14 mutações** permanecem provas herdadas da auditoria C01 `8fe3c653c0ba497d423282adcc75caa999d41e3a1e42b7508ba252f9f01ea7b3`. Seus hashes/versão e testes permanentes coincidem; as suites não foram repetidas sem motivo. HIP-I1-01 não foi promovida a defeito ou obrigação nova.

## Medições do auditor

GNU time lançou o mesmo binário nativo, com `-q -f "%U %S %M %e %x" -o <temporário>`; wall usa perf_counter_ns e inclui o wrapper. Todos os exits foram 0; os três stdout de cada carga coincidiram dentro da própria execução. Identificadores dos snapshots podem diferir entre campanhas independentes.

| Amostra | User/System (s) | Max RSS (KiB) | GNU elapsed (s) | Wall (ns) |
|---|---:|---:|---:|---:|
| M-1 | 0.08 / 0.04 | 30120 | 0.12 | 123329132 |
| M-2 | 0.06 / 0.06 | 30224 | 0.12 | 128107909 |
| M-3 | 0.07 / 0.04 | 30172 | 0.12 | 121411442 |
| L-1 | 0.01 / 0.00 | 3856 | 0.01 | 18361873 |
| L-2 | 0.01 / 0.00 | 3792 | 0.01 | 13037723 |
| L-3 | 0.01 / 0.00 | 3792 | 0.01 | 13436413 |

Os dados originais do executor também foram conferidos: M 30.024–30.272 KiB; L 3.748–3.880 KiB. São medições descritivas locais com cache não controlado, sem inferência de escala, latência garantida, cache frio ou proteção global contra OOM. Não houve benchmark concorrente iniciado pelo auditor, instalação, rede ou acesso a acervos reais. Nenhum comando ficou bloqueado por permissão nesta auditoria; não foi necessário repetir os cenários herdados de socket Unix.

## Comandos e rastreabilidade

| Comando/verificação do auditor | Resultado |
|---|---|
| `python3 verification/d0-r1/run.py --self-test`, na cópia | Exit 0; nenhuma invocação do sensor. |
| `python3 verification/d0-r1/run.py --binary target/release/compass --cached-dependencies /home/many/.cache/uv/archive-v0`, na cópia | Exit 0; M/L, 24 comandos CLI, quatro negativos/restaurações e seis amostras. |
| `python3 /tmp/compass-audit-r1-dqf4grfy/independent.py <raiz>`, na origem e na cópia | Exit 0 em ambas; contas, bytes, binding, recursos, cobertura e proveniência conferidos. |
| `python3 /tmp/compass-audit-r1-dqf4grfy/report-controls.py` | Exit 0; controle válido e quatro rejeições específicas. |
| `sha256sum --check --quiet SHA256SUMS`, em C2-A1; `git diff --check` | Exit 0; 81 artefatos e whitespace íntegros. |

Evidências auxiliares sob `/tmp/compass-audit-r1-dqf4grfy/`, sujeitas à retenção de temporários. `evidence-hashes.json` reúne os hashes; exemplos:

| Evidência | SHA-256 |
|---|---|
| `before.json` | `31a64b6d03b61dcc8b7a19143a26ac96a2e5164d92db44122a1b466b09ad7af6` |
| `runtime.log` | `b70f8db33337ace947c6125ced1682f8faad2be3c99467ee999956453331e70d` |
| `independent.py` | `d2312f7cf89565c2c76c4aeff8f09e96e8720b765e213053cef4b7579d1cbb33` |
| `independent-original.json` e `independent-replica.json` | `b07b9f444548a730e62c5a10e0eab4e443c571ef610c4ed421960541e9b463b2` |
| `report-controls.json` | `286bec865e3c774d61f1c0e183bc7931cf8a2a476e7f42fc773b47d109f0e4b1` |
| Proveniência da campanha do auditor | `bb39f57f102fda69342cc3b9f0613d06729bb0cb0892bff7de20ad7d2fd10a08` |
| Perfil de recursos da campanha do auditor | `e886b51d473c2b15e44bc3d178773c7fca88c90a8057dcc7cb508853df4f4c1c` |

## Encerramento

Sem BUG novo confirmado, hipótese bloqueante ou prova obrigatória pendente no escopo R1. Os 172 arquivos locais inventariados ficaram idênticos durante as verificações; as duas referências externas também permaneceram válidas. As nove entregas, fontes, binário, contratos, testes, esperados e pareceres anteriores são preservados.

Após este parecer, a coordenação registra o aceite somente em `COMPASS_STATE.md`, `docs/COORDENACAO.md` e `docs/entregas/d0/PLANO-ENTREGAS.md`. Esses deltas administrativos posteriores não pertencem à implementação R1 nem são regravados na sua proveniência histórica. A reprodução de uma campanha fixada usa a cópia da baseline correspondente; atualizações posteriores de estado/auditorias não são tratadas como alteração dos 37 inputs ou do binário.

R1 concluída/aceita no escopo contratado, sem autorizar a implementação de outra capacidade. Uso compartilhado consultado: 19% → 20% na janela de 10.080 minutos, aproximadamente +1 ponto percentual, sujeito a arredondamento e outros trabalhos; não é contagem de tokens desta auditoria.
