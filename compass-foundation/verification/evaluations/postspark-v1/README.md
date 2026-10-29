# Controle independente PostSpark v1

Preparado em 22/10/2026, por inspeção dos fontes de `/home/many/Projetos/PostSpark 3/`, sem alterar ou executar o aplicativo. A árvore tem alterações locais; a identidade é dos bytes inventariados, não do commit Git. Não houve acesso a credenciais, banco, serviço remoto ou chamada LLM.

## Material

- `PERGUNTAS.md`: único conteúdo de avaliação a fornecer ao agente, além da conexão e instruções operacionais.
- `gabarito.json`: **reservado ao avaliador**. Oito respostas por afirmações verificáveis, 13 intervalos de evidência, caminhos, chaves, hashes, offsets, texto bruto e erros de interpretação a rejeitar. Inclui consultas literais sugeridas apenas para o ensaio técnico controlado.
- `manifest.json`: inventário de 64 arquivos congelados (355.679 bytes), origem e identidade do recorte.
- `mcp-source.json`: ligação entre o corpus independente, o snapshot preparado e a configuração MCP separada.
- `verify_control.py`: valida integridade do gabarito contra o corpus ou contra a origem, sem consultar Compass.

O corpus contém TypeScript de `client/src/editor`, `shared/creative`, `shared/typography` e seis arquivos explícitos de migração/contratos compartilhados. Há implementações, testes existentes e arquivos vizinhos, não somente trechos de resposta. Não é uma cópia completa nem executável do produto; dependências externas, assets e outros módulos foram omitidos. Componentes legados são avaliados como código presente, sem presumir uso na interface atual. O estudo é uma amostra inicial pequena, não um benchmark representativo de todo PostSpark.

As respostas foram definidas por leitura do código ANTES de qualquer consulta de busca/leitura ao Compass. Linhas começam em 1 e o fim é inclusivo. Offsets são bytes, começam em 0 e o fim é exclusivo. LF delimita linhas; CRLF e Unicode foram conservados. O caso PS05 distingue um comentário de garantia do comportamento real de exceções; PS08 limita uma afirmação de ausência à função inspecionada.

## Conexão para este ensaio

```json
{
  "command": "/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/scripts/compass-mcp.sh",
  "args": ["/home/many/Documentos/Projetos linux/Bússola SGC/compass-foundation/.compass/evaluation/postspark-v1/config.json"],
  "env": {}
}
```

Alias: `postspark-control`. A configuração padrão `demo`/`compass-source` não foi alterada. Inicie uma conexão nova com os argumentos acima. Handshake, schemas e limites seguem `docs/MCP-INTEGRACAO.md`.

O corpus congelado para o modo nativo está em `.compass/evaluation/postspark-v1/corpus/`. Para comparação justa, use essa mesma árvore no modo nativo e a fonte `postspark-control` no modo Compass. Não use a árvore viva como referência de uma revisão congelada.

O gabarito e este README não estão no pacote MCP. O agente avaliado deve receber apenas perguntas e parâmetros operacionais; restrinja também seu acesso direto aos arquivos do avaliador. Separação de diretórios não é uma sandbox. Não indexar a pasta `verification/evaluations/postspark-v1` nem incluí-la em prompts. O agente que já leu o gabarito não deve ser usado como execução cega.

## Dois ensaios distintos

1. **Correção técnica:** o avaliador usa as consultas literais de controle. Calcule ocorrências esperadas diretamente nos bytes congelados, respeitando a semântica literal por linha. Compare respostas completas da busca, cobertura/truncamento e leitura dos intervalos de referência. Pode entregar caminho/linha neste ensaio; ele não mede descoberta autônoma. Para testar continuação, peça páginas de cinco linhas e recomponha os bytes do intervalo, sem normalização.
2. **Investigação pelo agente:** forneça uma pergunta por execução, sem consultas sugeridas nem caminhos esperados. O LLM decide a investigação. Registre chamadas, respostas MCP brutas, resposta final, citações e métricas. Compare cada resposta com `required_claims` e `reject_claims`. Aceite evidências equivalentes após conferência humana; o gabarito não enumera toda evidência relevante possível.

Não confundir os resultados: uma consulta ruim pode produzir resposta tecnicamente correta e investigação insuficiente. Uma resposta final correta sem leitura registrada pode vir de adivinhação e não comprova recuperação. Não atribuir desempenho de ferramentas nativas ao Compass se houver fallback: marque a execução como mista.

## Como pontuar

Registre uma linha por caso e por repetição:

| Medida | Regra |
|---|---|
| Integridade | Todos os trechos devolvidos correspondem aos bytes/offsets/revisões declarados; falha adulterada como válida reprova este eixo. |
| Cobertura de referência | União das linhas efetivamente lidas sobre os 13 intervalos esperados, por caso; não contar sobreposições duas vezes. É cobertura desta referência, não recall absoluto de todo conteúdo relevante. |
| Resposta correta | Cada afirmação exigida está correta e sustentada por citações; alegações proibidas devem estar ausentes. Julgamento semântico do avaliador, não comparação literal de redação. |
| Material útil | Classificação humana de trechos relevantes, apoio útil e irrelevantes. Arquivo fora da lista do gabarito não é automaticamente irrelevante. |
| Eficiência | Chamadas, bytes de contexto únicos/duplicados, tokens de entrada/saída, tentativas, latência e custo por caso correto. |
| Modo | Nativo, Compass ou misto; modelo/configuração/budgets idênticos entre comparações. |

Um caso é concluído quando suas afirmações estão sustentadas e não há erro material. Não exigir um único caminho de busca. Reporte separadamente fidelidade, cobertura e sucesso; não esconder erro de integridade em uma média. Faça repetições e preserve falhas, sem escolher só a melhor execução. Metas de custo/latência ainda não foram estabelecidas.

O caso PS06 cruza composição e PRNG; PS05 cruza migração e resolução. São casos iniciais de navegação entre arquivos, mesmo sem grafo implementado. Não há afirmação de que o aplicativo inteiro foi validado.

## Verificar o controle

Na raiz de `compass-foundation`:

```bash
python3 -B verification/evaluations/postspark-v1/verify_control.py
python3 -B verification/evaluations/postspark-v1/verify_control.py --root '/home/many/Projetos/PostSpark 3'
```

Ambas as verificações passaram na preparação: 64 arquivos e 13 intervalos, oito casos. A montagem do pacote também foi conferida contra os hashes/bytes dos 64 arquivos. Os testes do PostSpark não foram executados. Não há ainda resultado de recuperação ou uso por agente neste ensaio. Os arquivos locais sob `.compass/` são ignorados pelo Git; preserve-os junto do controle para repetir o experimento. Não regenere gabarito para acomodar uma resposta incorreta; uma nova revisão do acervo exige nova versão identificada do controle.
