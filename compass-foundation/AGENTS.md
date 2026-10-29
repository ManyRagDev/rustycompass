# Instruções do Compass Foundation

Leia `docs/PLANO-IMPLEMENTACAO.md` para o objetivo/progresso atual e consulte `COMPASS_STATE.md` para um resumo. Leia as especificações técnicas pertinentes antes de alterar comportamento, incluindo `research/contracts/FOUNDATION.md` quando afetado. A direção de produto está em `../compass-documento-principal/COMPASS_DOCUMENTO_PRINCIPAL.md`; os arquivos dessa edição são uma referência preservada.

O núcleo Rust é a única implementação de produção. A CLI chama esse núcleo. Python em `verification/` é verificação independente. Não importar código, testes ou instruções do legado nem ampliar Foundation para D0, grafos, embeddings ou MCP sem uma tarefa correspondente.

Preserve gabaritos independentes. Uma divergência legítima no contrato exige uma alteração explícita do contrato e de seus consumidores; não ajuste o esperado só para fazer o código passar.

Build e testes: `scripts/cargo-local.sh test --workspace --locked --offline`. Verificação independente: consulte `verification/ACCEPTANCE.md`. Execute formatação e Clippy ao concluir mudanças Rust. Não atribua resultados a um commit inexistente ou a uma árvore diferente do build testado.

## Continuidade e contexto

Retome o objetivo autorizado a partir de `docs/PLANO-IMPLEMENTACAO.md` e das referências técnicas pertinentes. O checklist e o progresso ficam somente nesse plano; não crie outra OS ou relatório por tarefa. Atualize o estado quando necessário para retomar trabalho longo ou registrar uma capacidade duradoura, sem transformar cada teste ou correção em nova documentação.

OSs, planos antigos e `docs/COORDENACAO.md` são exclusivamente históricos. Não exigem listas fechadas de arquivos, hashes administrativos, nova conversa, troca de modelo ou liberações entre fases técnicas. Correções, integração e verificação pertencem à mesma etapa do plano. Na implementação autorizada, siga até integração e verificação. Resolva dependências técnicas no mesmo objetivo e continue nas partes independentes de um bloqueio real.

Preserve contratos de comportamento, gabaritos independentes e evidências históricas. Hashes do conteúdo, do protocolo e da proveniência técnica continuam necessários onde o produto os exige; hashes de controle administrativo não são uma obrigação genérica de entrega. Não altere critérios para acomodar defeitos.

Leia somente o contexto necessário e use as verificações focadas durante a implementação, fechando o fluxo integrado conforme o efeito da mudança. Revisão independente entra quando exigida pela entrega ou justificada pelo risco, sem reabrir tudo por rotina. Não declare prova executada em outro build como se fosse do atual.

Não abra chats, envie mensagens ou delegue sem autorização aplicável. Modelo e esforço são escolhas explícitas; uma recomendação histórica não muda a configuração. Limites de custo são respeitados, sem exigir consulta de uso a cada tarefa.

Ao entregar, explique primeiro o que a solução agora permite fazer, as condições e limitações. Depois resuma as verificações e referências técnicas necessárias.
