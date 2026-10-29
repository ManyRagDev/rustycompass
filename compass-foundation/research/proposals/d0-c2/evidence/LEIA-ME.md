# Evidência documental D0-C2

Reprodução do diretório `compass-foundation/`:

```sh
python3 research/proposals/d0-c2/evidence/conferir.py --cached-dependencies /home/many/.cache/uv/archive-v0 > /tmp/d0-c2-reproduzido.json
diff -u research/proposals/d0-c2/evidence/conferencias.json /tmp/d0-c2-reproduzido.json
```

A flag carrega somente distribuições oficiais já instaladas no cache uv (jsonschema/attrs/referencing/rpds/jsonschema_specifications/typing_extensions). Não instala nada nem importa módulos do legado. Em outro ambiente com jsonschema 4.26.0 instalado, omitir a flag; versão Python pode mudar campos ambientais do registro. O módulo jsonschema não estava nos dois runtimes padrão consultados; o cache local forneceu a ferramenta apropriada, portanto a validação real foi possível. Não foi escrito um validador JSON Schema caseiro.

Inputs literais: cases.json usa listas de linhas e LCS esperadas de contas manuais; byte-vectors.json usa hex e linhas esperadas; hash-vectors.json fixa prefixo/bytes/hash; resource-frontiers.json fixa dimensões e resultados de admissão. Relatórios foram compostos a partir de entradas literais/contagens manuais, sem sensor ou CLI. report-inputs.json indica manifestos/pacotes físicos correspondentes. Todos os esperados antecedem implementação.

Métodos: enumeração do conjunto de subsequências para LCS; Decimal alta precisão/half-even para decimal; hashlib sobre o buffer bruto; cálculo fechado de budgets; JSON Schema Draft2020-12 oficial + format checker + referências locais; conferência semântica somente das pequenas fixtures. 961 pares com alfabetos {a,b} e comprimentos≤4 verificam propriedades, não demonstram eficácia contextual. A evidência E01–E35 combina recomputação matemática e conferência manual de regras; os casos operacionais não foram executados por uma produção inexistente.

D0-C2-A1 acrescenta uma regra de admissão, não uma otimização livre: após fonte verificada, classificação textual, normalização, limite de linhas e admissão de memória, sequências normalizadas exatamente iguais reservam zero células porque `LCS(X,X)=|X|`. O checker cobre o teto 4.000×4.000, saldo já zerado, CRLF/LF com bytes diferentes, fontes ausentes/adulteradas que não chegam à igualdade, mesma contagem com sequências diferentes e o saldo preservado para uma unidade seguinte. Não mede performance nem exercita o leitor futuro.

Fontes/binding: packages/abc-adulterated contém **deliberadamente** abd no objeto cujo nome/revisão esperada é abc, com descriptor válido igual ao pacote abc. A fixture unavailable-hash espera a falha, sem corrigir o objeto. packages/a-whitespace tem descriptor válido não canônico, fonte/ref distintos por hash bruto. packages/unknown não fornece objeto porque o unknown ancestral impede leitura; não inferir object_missing não observado.

Byte arrays/fonte estão nos mesmos arquivos usados para conferir checks dos reports. `.jcs` e source.json/manifests canônicos não têm newline; JSON indentado de leitura humana tem newline. A função canônica do checker aceita apenas fixtures ASCII/string-only/bool/null/arrays/objects, sem números JSON. **Não implementa JCS geral.** V1/I1 precisam dos vetores/comportamento da biblioteca JCS de produção e de validações independentes pertinentes; não generalizar json.dumps.

Não exercitado: abertura segura real/links/FIFO/corridas, EOF em syscall, cancelamento/prazo, CLI pública, desempenho/RSS, parser contra carga adversarial, campanha completa de mutações. Essas provas exigem V1/I1/R1 após aprovação, não justificam iniciar V1 nesta tarefa. Ledger é política lógica, não medição de RAM. C2 conferiu inputs de build, não recompilou.

preservacao.json guarda o inventário SHA-256 capturado antes e conferido depois dos 34 arquivos protegidos. Inclui C1, Rust/CLI, contratos/fixtures ativos, verificação/scripts Foundation e documento principal original. COMPASS_STATE.md e docs/COORDENACAO.md são os dois registros de progresso autorizados e não entram na lista de preservação imutável. O índice global de artefatos candidatos é SHA256SUMS, conferível sem executar Python/Rust.

Uso da conta: consultas de início/fim da janela compartilhada de 10080min mostraram 28% e 29% (+1 ponto percentual). Aproximação compartilhada com outros trabalhos; não é contagem de tokens desta tarefa nem demonstra economia. Nenhum commit criado/resultado atribuído a commit inexistente.
