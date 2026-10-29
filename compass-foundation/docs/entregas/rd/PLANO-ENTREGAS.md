> **Histórico encerrado como instrumento de execução em 21/10/2026.** O [plano único](../../PLANO-IMPLEMENTACAO.md) substitui as OSs, as liberações administrativas e os prompts abaixo. Conteúdo preservado para consulta histórica; não representa a fila atual.

# Recuperação direta — busca e leitura de contexto

19/10/2026. Foundation e sensor D0/R1 concluídos/aceitos. Próximo marco do documento principal §17.3: **busca e entrega de trechos da revisão declarada**.

Primeiro incremento aceito: `compass search` literal sobre um snapshot e seu pacote explícito verificado, com linhas/offsets originais, disponibilidade e limites. Próximo incremento: `compass read`, para abrir linhas vizinhas e continuar no mesmo snapshot/revisão. Esses incrementos entregam evidência utilizável pela CLI; não comprovam benefício para agentes ou eficácia de ranking.

Estado atual: RD-I1/C01 aceitas após [auditoria corretiva CONFORME](../../../research/audits/AUDITORIA-OS-RD-I1-C01.md). BUG-005/RD-AC-07 resolvidos; BUG-RD-C-001 continua fechado. **Próxima execução: [OS-RD-I2](OS-RD-I2.md), LIBERADA**, com [contrato de leitura](CONTRATO-RD-LEITURA-V0.1.md) e [auditoria prévia CONFORME](../../../research/audits/AUDITORIA-CONTRATO-OS-RD-I2.md). Nova conversa, GPT-6.1 Sol / Alto, um escritor e auditoria de entrega posterior.

| Etapa | Avanço | Execução/portão |
|---|---|---|
| Contrato RD | [Contrato RD v0.1](CONTRATO-RD-V0.1.md), interface e RD01–RD24 | Arquitetura atual; auditoria prévia pelo risco de conteúdo/fronteira de leitura |
| RD-I1 | [OS-RD-I1](OS-RD-I1.md): fixtures independentes → núcleo/CLI → integração/recursos/regressão | Implementada e aceita após C01 |
| RD-I1-C01 | [Correção BUG-005](OS-RD-I1-C01.md): distinguir flag real de valor literal de query | Concluída/aceita; dois deltas/cinco novos; 187 preexistentes preservados |
| Conferência corretiva | BUG-005/RD-AC-07, antes/depois e regressões | CONFORME; 48/48 controles, 76 Rust e campanhas RD/Foundation/D0; núcleo e contrato preservados |
| RD-I2 | [OS de leitura](OS-RD-I2.md): gabaritos independentes → read/CLI → fluxo/medidas/regressões | LIBERADA após parecer prévio CONFORME; cinco existentes/onze novos; sem implementação nesta preparação |

Uma OS executa as três fases na mesma conversa. Não criar C1/C2/V1 separados sem achado que justifique outra fronteira. A entrega deve terminar em comportamento público demonstrado, com documentação e evidência, não numa nova proposta.

A busca exige ler conteúdo e verificar fonte; a reutilização da fronteira D0 é delimitada e coberta por regressão do novo release. Cargo/dependências permanecem iguais. O contrato RD entra expressamente nos inputs do build existente; não há novo serviço, banco, ID de geração, mecanismo de retenção ou policy_id paralelo. A política de busca é versionada pelo contrato e pela configuração devolvida no relatório.

Limites iniciais são de engenharia e não uma promessa de escala. Medição local de três invocações nativas da carga definida identifica ambiente, fonte/build, RSS e créditos separados. Não repetir a pesquisa D0 ou medir eficácia de ranking sem recuperador/comparador e protocolo próprios.

RD-I2 completa a leitura explícita por path_key/faixa, verificando o objeto inteiro antes de entregar texto; next_start_line permite continuar sem sessão/cursor opaco. Reutiliza limites/leitor/semântica de linhas RD, preserva a busca e traz RL01–RL16. Três fases sequenciais na mesma conversa; auditoria prévia de contrato antes de liberar e conferência independente após implementar. Não há implementação de read nesta arquitetura.

Posteriormente, busca lexical/indexação e manutenção podem receber escopos próprios conforme a necessidade demonstrada. Este plano não os autoriza nem fixa bibliotecas/modelos. MCP pode adaptar o núcleo direto quando utilizável, sem exigir o término de pesquisa térmica. Nada do legado será importado.

Reversão: preservar cópias/deltas, binário e evidências C01/R1; retirar somente alterações autorizadas da RD. Nenhuma operação remota ou acervo real autorizado. Estado durável em COMPASS_STATE.md e prompt corrente em docs/COORDENACAO.md.
