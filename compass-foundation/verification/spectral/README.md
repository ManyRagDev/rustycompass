# Controles independentes do incremento espectral

Somente `fixtures/dev` ou `fixtures/reserved` entra na coleção. Gabaritos em
`controls/` e `frozen-cases.json` ficam fora do acervo. Os reservados não devem
ser consultados por autores da política; sua medição exige perfil congelado.
`prepare_cases.py` é a autoria reproduzível dos controles, não uma etapa para
regravar gabaritos após resultados. As evidências históricas permanecem intactas.

Referência NumPy (Python com NumPy):

```sh
python3 verification/spectral/numerical_reference.py
python3 verification/spectral/compare_numerical.py --probe target/debug/examples/spectral_probe
python3 verification/spectral/check_comparator_mutations.py
python3 verification/spectral/runtime_controls.py --probe target/debug/examples/spectral_probe
python3 verification/spectral/benchmark_numerical.py --probe target/debug/examples/spectral_probe
```

A referência foi escrita antes de consultar o solver novo. Decomposição simétrica,
momentos espectrais e residual são comparados a dois nós analíticos e invariantes
de reescala/tempo físico. Truncamento analítico e arredondamento estimado ficam
separados; allowances do oráculo flutuante não certificam arredondamento.
O comparador confere status completo, qualidade não informativa/indeterminada,
intervalos finitos/ordenados, erros distintos de `h` e `y`, e trabalho acumulado.
As mutações verificam a rejeição desses relatos incorretos mesmo quando os
vetores e observáveis centrais continuam corretos. Os controles de execução
incluem cancelamento, limites, valores inválidos, composição temporal e perda
de massa observável com tolerância de truncamento relaxada.

O benchmark numérico mede 21 grafos em duas repetições, com até 16.384 nós e
limite interno de 250 ms por requisição. Usa o executável debug identificado
pelo hash, mede também tempo/RSS do processo e mantém os casos limitados no
resultado. Construção do grafo e serialização ficam fora do relógio interno;
o tempo total inclui ambas. Isso não projeta desempenho para 100 mil nós nem
para um build release. `--repetitions` controla de uma a cinco repetições.

Baseline legado, sem build automático nem LLM:

```sh
python3 verification/spectral/acquisition_baseline.py --binary target/release/compass
```

Os braços `none`, BFS e heat usam geração/consulta/orçamento iguais. O legado
não tem escopo nem o materializador experimental: essa comparação não isola
escolha de escala. Contratos completos/incompletos são controles separados;
seus seletores nunca alimentam braços abertos. Cada pack é conferido contra
bytes, intervalos, hash e tamanho JSON canônico. CLI e MCP stdio real concordam
nos três braços e nos dois controles de obrigações a 16 KiB.

Para a comparação que isola seleção, `matched_acquisition.py` espera um exemplo
JSON-lines sobre **o mesmo materializador/fechamento/sementes**. Entrada por linha:

```json
{"case_id":"public-control:16384:fixed_heat","store":"/tmp/store","generation":"generation-id","query":"symbol","scope":"dependencies","budget_bytes":16384,"strategy":"fixed_heat","theta":1.0}
```

`strategy` admite `none`, `bfs`, `fixed_heat`, `spectral_window_v1`. `theta` existe
somente no controle fixo. Saída por linha:

```json
{"case_id":"public-control:16384:fixed_heat","pack":{},"pack_json":"{}","work":{},"wall_ns":0,"trajectory":[]}
```

`pack_json` contém os bytes UTF-8 canônicos exatos do pack como string JSON, sem
newline; `pack` é o mesmo objeto decodificado. Erro explícito pode retornar
`{"case_id":"...","error":{"code":"...","message":"..."}}`. Trabalho deve
contar leituras/candidatos/nós/arestas/operador/escalas e trajetória; informação
indisponível permanece ausente, nunca inferida como zero. Tempo/RSS de todo o
processo não deve ser apresentado como custo individual de uma escala.

```sh
python3 verification/spectral/matched_acquisition.py --probe target/debug/examples/acquisition_probe
```

Para a análise retrospectiva, `--fixed-theta-grid .015625 1 16 64 256` mede cinco
escalas fixas na mesma geração dos braços none/BFS/adaptativo: 112 aquisições
no conjunto de sete casos/dois orçamentos. Cada escala extra tem custo próprio;
o relatório distingue aquisições via adaptador, chamadas públicas e preparação
instrumental. `summarize_matched.py --input resultado.json --output resumo.json`
mede suporte final e a existência retrospectiva de candidato com suporte na
trajetória ou no grid fixo, sem usar o gabarito durante a seleção.

Somente após o sinal explícito de revisão/congelamento, preparar a identidade:

```sh
python3 verification/spectral/freeze_profile.py --dev-results verification/spectral/matched-dev-final.json --probe target/debug/examples/acquisition_probe --binary target/debug/compass --review-reference referencia-da-revisao
```

O comando não executa reservados. Registra fontes do núcleo, binários, perfil,
classes, orçamentos e critérios de medição; não cria meta de promoção numérica
que não foi acordada. A execução reservada exige `--split reserved
--frozen-profile verification/spectral/profile-frozen.json` e rejeita fonte ou
binário alterado desde o congelamento. Não divulgar gabaritos aos autores nem
ajustar regras após a avaliação reservada.

O perfil congelado inclui revalidação/reparse da **geração completa**, limitada
a 128 documentos e 22.369.621 bytes de fonte. Acima do limite a aquisição retorna
pack vazio terminal com `structural_work_limit`. O benchmark do solver não
demonstra aquisição de acervos extensos. Metadados atuais não cabem no perfil
curto de 3 KiB: suas recusas permanecem no resultado, e custos individuais não
retornados são desconhecidos, não zero.

Após as duas medições, `python3 verification/spectral/consolidate_evaluation.py`
confere a identidade congelada e produz `evaluation-summary.json` com custos,
denominadores, suporte e limites de interpretação. `review-evidence-metadata.json`
preserva somente metadados das provas de revisão independente, distinguindo-as
dos controles executados pelo avaliador.

Os binários exatos da medição congelada foram preservados pelo coordenador em
`.compass/spectral-evaluation-build/{compass,acquisition_probe}`. Use esses caminhos
para reprodução e confira os hashes em `profile-frozen.json`. Uma compilação
posterior de todos os exemplos relinkou o caminho padrão do probe sem mudança
de fontes/perfil; essa nova identidade não é a identidade medida. Regressões
release pertencem a outra evidência, sem transferir tempos deste build debug.

O runner ainda necessita desse adaptador de produção para execução. Casos com
lacunas, erros e sem atendimento permanecem nos denominadores. Cobertura mede
suporte presente no pack por conjuntos alternativos aceitáveis; presença de um
corpo com callback/configuração externa não torna a resposta respondível.
