# Auditoria — OS D0-C2-A1

**Veredito: CONFORME para o ajuste documental D0-C2-A1.** Nenhum desvio obrigatório dos critérios da ordem foi identificado. A questão T09/E36 está resolvida no candidato: igualdade exata das sequências normalizadas, após as admissões anteriores, reserva zero células; sequências diferentes mantêm a reserva quadrática. Este parecer não ativa D0 nem homologa uma implementação.

Data: 18/10/2026, America/Sao_Paulo. Intenção: auditoria. Skill: `auditor-red-team` v2.0, modo `entrega`. Ambiente das conferências: Linux, Python 3.12.3 e jsonschema 4.26.0 disponível no cache local. Sem instalação ou operação remota.

## Ordem e versão auditada

Critérios congelados: os cinco itens de [Prompt pronto — D0-C2-A1](../../docs/COORDENACAO.md), linhas 124–141. SHA-256 do conteúdo do bloco, sem delimitadores e sem newline final: `8951f1f4dc81123f2b71019a3c36463a14b29ab59c78c37f0e61e45463814985`.

Baseline: árvore de trabalho da branch `main`, ainda sem commits/HEAD. Git mostrava 34 entradas Foundation com alterações locais, além de `docs/` e `research/proposals/` não rastreados. Não há diff histórico de commit que isole A1; a auditoria compara a entrega presente aos critérios originais, aos hashes protegidos e à identidade anterior da política registrada na coordenação desta conversa.

Antes das verificações, foi fixado um inventário SHA-256 de 119 arquivos, incluindo os protegidos. Seu digest é `b0517fc6064633ec43719beb1be379db632e23de920ec5928acf2cc90e40f105`. Os 119 permaneceram idênticos durante a auditoria. O inventário e saídas auxiliares desta sessão estão em `/tmp/compass-audit-d0-c2-a1-64fkovef/`; os identificadores duráveis do conjunto auditado estão abaixo.

| Artefato | SHA-256 auditado |
|---|---|
| `research/proposals/d0-c2/SHA256SUMS` | `63168b321845393168134a30afa39811e702eecc678b31c06f3bc24d3c17208e` |
| `research/proposals/d0-c2/CONTRATO_CANDIDATO.md` | `884eecc521bfa123ad3d0cc608c7c7619ab63bb87affa2112be217ac47f7291e` |
| `research/proposals/d0-c2/evidence/conferir.py` | `33e241929b598b8cd685890d9e5b8379fb7c88df847c86f70e42d4f57d19b164` |
| `research/proposals/d0-c2/evidence/conferencias.json` | `2ee943d8d60257b982e907af6caba5d356a76b3f307f52992df88bc4ddd41945` |
| `research/proposals/d0-c2/RESULTADO-D0-C2-A1.md` | `b6f90ef6171f21e9bd11ad9d35de9793862ccf1b95f03568fba5f57e8399db1a` |
| `docs/COORDENACAO.md` | `2bdf0f3072a8ecba2617667c853208b5e0d0164eaa4908583cc24805220f8c2c` |

## Confronto com os critérios

Os itens abaixo correspondem à ordem existente, sem acrescentar critérios retroativos.

| Item da OS | Resultado e evidência |
|---|---|
| 1 — regra, precedências e limites | Verificado no contrato candidato §§3, 4 e 6, especialmente linhas 99–105. A igualdade ocorre depois de fonte/texto/normalização/linhas/memória. Hash, tamanho e contagem isolados não bastam. Bytes, memória conservadora, limites e cancelamento continuam exigidos. |
| 2 — prova, política e identificadores | `LCS(X,X)=|X|` está explícita. O schema exige `normalized_lines_exact_equal_zero_else_n0_times_nt_v1`. Policy JSON/JCS, vetor de hash e os dez relatórios concordam com o novo identificador. A regra antiga é rejeitada pelo schema atual. |
| 3 — exemplos e preservação da matemática | E36 e E51–E55 cobrem os seis cenários pedidos. Os novos agregados são E51: `2/6 = 0.333333`; E52: `0/2 = 0.000000`; E55: `2/8 = 0.250000`. Igualdade mantém massa/denominador; não transforma fontes ausentes/adulteradas em cálculo disponível. |
| 4 — consistência e preservação | Checker reproduzido sem divergência de saída; 81 hashes válidos, inventário completo sem duplicatas; 34 arquivos protegidos idênticos. A identidade dos 21 inputs Foundation coincide com o release registrado. |
| 5 — registro e fronteira | Resultado, estado e coordenação registram A1 e mantêm P1–P4 pendentes. Contrato permanece candidato. Não há alteração dos arquivos protegidos de produção/Foundation nem liberação de V1/I1. |

## Verificações executadas nesta auditoria

O resultado do executor foi tratado como relato. A auditoria executou novamente:

- `python3 research/proposals/d0-c2/evidence/conferir.py --cached-dependencies /home/many/.cache/uv/archive-v0`, com saída em diretório temporário. A saída é idêntica byte a byte a `evidence/conferencias.json`: 55 casos catalogados, 44 cálculos LCS, 23 agregações, 961 pares pequenos, cinco schemas, dez relatórios/inputs e demais verificações enumeradas no JSON.
- `sha256sum --check --quiet SHA256SUMS` dentro de `research/proposals/d0-c2/`. Conferiu também que as 81 entradas cobrem todos os artefatos candidatos e o parecer, sem omissão ou duplicata; o índice exclui a si próprio.
- Comparação dos bytes atuais dos 34 arquivos de `evidence/preservacao.json` contra os hashes anteriores/posteriores registrados. Todos coincidem.
- `git diff --check` e `git diff --cached --check`, ambos sem erros. Esses comandos não cobrem arquivos não rastreados; a evidência principal dos candidatos é o inventário, a leitura e o checker.

Além da repetição do checker, foram feitas conferências independentes, sem importar seu código ou a produção:

1. Recalculadas as cinco LCS literais novas de E51/E52/E55 por programação dinâmica, em contraste com a enumeração de subsequências do checker. Os três agregados foram recalculados com divisão inteira e arredondamento de empate para par; todos coincidiram.
2. Conferidas sequências distintas com 4.000 linhas e mesmo tamanho, cujo produto é 16.000.000 e continua excedendo o teto de 10.000.000. Para duas sequências iguais de 4.000 linhas, a identidade matemática determina `l=4000`, `C=0`, `M=8000`; não foi executado diff quadrático.
3. Conferidos bytes CRLF/LF diferentes, hashes brutos diferentes e igualdade após a normalização de E52; conferidas as agendas de E51/E55 que totalizam quatro células e deixam igualdade com custo zero.
4. Em cópia em memória da política nova, substituído somente `cell_budget` pelo identificador antigo. O hash voltou exatamente ao anterior: `2a85b5efcd51056049661396a6dac0cdc0ec8c2f24071ce660589ae1e9d8b504`. Isso confirma a preservação dos demais campos da política padrão. A cópia com a regra antiga foi rejeitada pelo schema novo. Nenhum arquivo foi modificado por essa conferência.

Identificador da política auditada: `51d824e4f7e229cfeb61b63c33681fff65a879e173956bdb0029dd4473d358a5`.

Identidade reproduzida dos inputs Foundation: `sha256:94866a9d405c283d1913c13e53ff0d5413823f3ce1a654e8bbc68619d7ddd4b0`. Não representa um novo build.

## Achados e encaminhamento

Nenhum novo `BUG-###` confirmado ou hipótese bloqueante foi encontrado no escopo A1. T09/E36 fica **verificado no nível documental**, encerrando o ajuste solicitado. Não há motivo sustentado por esta auditoria para outra rodada de correção da mesma regra.

As pendências abaixo já existiam e não constituem defeitos da entrega. A responsabilidade pelo próximo passo permanece explícita:

| ID | Tipo | Estado para ativação | Decisão pendente | Responsável |
|---|---|---|---|---|
| P1 | produto | bloqueado | Pacotes externos e preparo dos conteúdos pelo operador | Emanuel |
| P2 | produto | bloqueado | Massa bilateral linear como significado do agregado | Emanuel |
| P3 | produto | bloqueado | Universo textual, normalização e zeros | Emanuel |
| P4 | produto | bloqueado | Rigor da evidência e disponibilidade | Emanuel |

Após essas decisões, cabe à coordenação registrar o conjunto autorizado por hashes e liberar V1/I1. O parecer não atribui aprovação ao usuário nem inicia essas etapas.

## Limitações e alterações desta auditoria

A campanha é documental: 55 casos catalogados não equivalem a 55 testes ponta a ponta de um sensor. Não foram executados Rust/CLI D0, corridas de filesystem, prazo/SIGINT real, benchmark/RSS ou V1/I1. O checker também declara essas limitações. Sem baseline histórico completo de A1 em Git, não se afirma ter reconstruído um diff integral anterior/posterior de todos os candidatos.

Somente este relatório foi acrescentado ao projeto. Código, contratos, fixtures, checker, critérios, estado e coordenação permaneceram intactos.

Uso da conta consultado no início/fim: 7% → 8% na janela compartilhada de 10.080 minutos, diferença de 1 ponto percentual. Medida arredondada e compartilhada com outros trabalhos; não é contagem de tokens da auditoria.
