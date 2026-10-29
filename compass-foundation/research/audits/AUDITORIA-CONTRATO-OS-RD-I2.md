# Auditoria de contrato — OS-RD-I2 v1

**CONFORME. Nenhum achado obrigatório aberto.** O contrato e a lista de escrita permitem implementar a leitura verificável por linhas sem alterar o leitor seguro, as ferramentas históricas ou a semântica de search. A coordenação pode alterar somente o estado administrativo da OS para `LIBERADA_PARA_EXECUCAO`. Este parecer não declara read implementado nem substitui a auditoria da entrega.

19/10/2026, America/Sao_Paulo. Intenção: auditoria. Skill `auditor-red-team` v2.0, modo `contrato`, conforme `references/achados.md`. Escopo: [contrato de leitura](../../docs/entregas/rd/CONTRATO-RD-LEITURA-V0.1.md), [OS-RD-I2](../../docs/entregas/rd/OS-RD-I2.md), RL01–RL16 e I2-AC-01–10. RD/C01 aceita é baseline herdada; sua implementação não foi reauditada.

## Referência concreta

Branch `main`, sem HEAD/commit: `git rev-parse HEAD` retornou 128. Status e diffs inspecionados; inventário de 195 arquivos fixado antes de qualquer escrita. Nenhum resultado é atribuído a commit inexistente.

| Autoridade/baseline | SHA-256 |
|---|---|
| Contrato leitura v1 | `54b4826516d9552598a71cba05c09403e64022ba9ba4f1a12f936c358c0e44d6` |
| OS-RD-I2 v1, PREPARADA_PARA_AUDITORIA_DE_CONTRATO | `e699051a55bc45d7af4be72e4aa238ac9ba15c5084f958d2fa5e82aee0e1771f` |
| Contrato busca RD | `0774a45fee54051c2a86126e1d0d9dfb87795850b658eda2537d7f1b517e8e72` |
| Parecer RD-I1-C01 | `1dd71986c3e533b34c2fc739d66322b858dfffaba917668afd6c37a6e9f6d9f8` |
| search.rs antes da mudança de visibilidade | `04a8e696d7fd3739419872aa5b7ed7760f7c9d5e1cf654fcb7167c968f5f65aa` |
| Checker C01 | `5e15e4df2d6a3d5610cdb274880c5877d28b8ac02f9bf084e8282852177e85cd` |
| Release C01 | `670922517d5c35f2d6adc794ed86d334bead81b3b65997a0a0847752900645e8` |

Recomposição independente dos **41 inputs**, com o framing de `build.rs:48–61`: `sha256:e9bd48d20b9ecadd70390aff63ca11455fb0ff1829e79bb08c2b14b3ee47c012`. Os números e hashes da OS conferem. Os quatro acréscimos previstos — módulo read, testes core/CLI e contrato explicitamente coletado — levam a 45 inputs; não é uma identidade de build futuro já produzido.

## Conferência dos requisitos e da fronteira

| Área / aceites | Evidência e conclusão |
|---|---|
| Escrita e visibilidade — AC-01 | `search.rs:22–24,123–193` contém exatamente `Limits.inner`, `manifest`, `text_reason` e `line_preflight` a tornar crate. `d0/mod.rs:2–7`, `input.rs:56–59,106–170`, `resources.rs:11–26` e `source.rs:301–320` já permitem acesso a Manifest, Context e Source::object. `d0::Limits.values/duration` são crate (`types.rs:39–43`); o relatório pode enumerá-los sem abrir o método privado `search::Limits::value`. Não há necessidade comprovada de editar D0 ou duplicar o leitor. |
| Argumentos e binding — AC-04/06 | `model::decode_path_key` é público (`model.rs:165–180`); read deve acrescentar a rejeição explícita da chave vazia, aceita pelo helper para cobertura da raiz. Lookup exato após manifesto e `input::source` preserva o binding bruto e sua validação no universo vazio. Chave válida sem observed resulta em path_not_observed, sem acesso a objetos. |
| Revisão, falhas e parcialidade — AC-04/05 | Uma chamada de `Source::object` para a entry selecionada basta: reserva s+1, EOF, revisão real e estabilidade já estão em `source.rs:315–377`. Arquivos não selecionados ficam fora da leitura e do orçamento de objetos; validação estrutural integral do manifesto permanece. Available é disponibilidade do arquivo; quality=partial e scope permanecem explícitos. Falhas do descritor/raiz anteriores ao lookup geram envelope, conforme §§1/4 e RD herdada; não devem ser convertidas em relatório de arquivo. |
| Linhas e continuidade — AC-02/03/07 | §§2–3 fixam LF-only, terminador incluído, sufixo final, bytes exatos e preflight integral antes da seleção. As fórmulas independentes abaixo conferem. Comparar start_line com total_lines antes de converter índice/somar permite tratar u64::MAX, vazio e além EOF. Cada continuação reverifica o objeto e devolve snapshot/revisão; não exige cursor, cache ou sessão. |
| Recursos e cancelamento — AC-06 | §5 conserva S_manifest/S/W, ordem das admissões, reserva sem reembolso e cap canônico. Context tem hook de teste/expiração (`resources.rs:8–63`) e o leitor oferece `object_before_stability` (`source.rs:365`), permitindo as barreiras previstas dentro de read.rs. O limite de 65536 bytes aplica-se aos novos laços; os primitivos compartilhados continuam sem preempção interna, com checkpoints preservados e checagens ao redor das chamadas. Isso mantém o fechamento de BUG-RD-C-001. |
| Erros e transporte — AC-06 | §1 exige consumo do token de path-key, inclusive --json, e §4 fecha envelope/exit/JCS/newline/nulls. O precedente real é `main.rs:269–285`: consumo especial de query no erro. A alteração autorizada em main.rs permite acrescentar a decisão equivalente para read e seu roteamento, preservando search e Foundation/D0. `d0::Error` já representa path_not_observed/exit 1 sem alterar types.rs (`types.rs:6–18`). |
| Provas, regressões e reversão — AC-02/07–10 | RL01–16 cobrem controles e mutações específicas antes de Rust. `rd-c01/check.py:82–112` aceita binário absoluto sem pin ao release antigo e monta campanhas em cópia, gravando seus próprios HERE; a cópia externa prevista pela OS protege também rd-c01. Os JSONs incorporam RD e regressões Foundation/D0; R1 fixada fica fora. A lista autoriza todos os novos artefatos exigidos. Cópias/deltas/binário salvo delimitam reversão; nenhum acesso remoto/acervo real é autorizado. |

O objetivo corresponde ao fluxo de consulta do documento principal §6.3 e ao marco de recuperação direta §17.3. Os estados administrativos antigos em plano/coordenação são contexto a atualizar pela coordenação; a OS nova aponta expressamente o parecer C01 aceito e exige auditoria antes de execução. Não representam autorização para o auditor editar esses arquivos.

## Conferências independentes realizadas

Somente leitura estática, hashes e contas pela biblioteca padrão Python; sem importar oráculo RD e sem chamar o executável:

- RL01/RL02: seis seleções conferidas, incluindo `[2,4)`, `[2,6)`, vazio/LF e start=4/u64::MAX com `[6,6)`, zero linhas retornadas e sem continuação.
- RL03: 15 bytes, intervalos `[0,8)`/`[8,15)`; BOM+alvo+LF tem oito bytes; CR/U+2028 não separam linhas. Escalar UTF-8 cruzando a posição 65536 foi construído com linhas admitidas.
- RL15: receita literal herdada produz objeto de 4096 bytes/64 linhas, BASE só em j=31; start=30/max=5 resulta em `[1856,2176)`, 320 bytes, next=35/truncated=true, leitura 4096/reserva 4097.
- Crédito independente b=1234/N=5/d=321/output=4096/s=8192: S_manifest=128656, S=131224, W=81922, peak=213146. As fronteiras igual/um abaixo e s+1 são compatíveis com o mecanismo existente.
- Framing e SHA dos 41 inputs, hashes normativos e do binário conferidos; whitespace dos dois documentos e `git diff --check` aprovados.

Evidência auxiliar em `/tmp/compass-audit-contract-rd-i2-i9hzq_zn/`: `checks.json`, SHA `c3449a36c512dc45349a0589aa4deb3ee19d83fac0a935e1427bf9a49274b3f1`; `inventory-before.json`, SHA `d66d62d1e6566a954dd041ccb3a7d54128a55e719c20fa70ba57efb2076d5746`; status/diffs preservados. A conferência final manteve os 195 arquivos inventariados e acrescentou somente este parecer. Contrato e OS conservaram os hashes auditados.

## Registro e limite do parecer

| ID | Tipo | Estado | Evidência/reprodução | Critério de fechamento | Tentativas | Responsável | Versão |
|---|---|---|---|---|---|---|---|
| — | — | sem achado obrigatório | Revisão delimitada acima | Não há correção exigida nesta auditoria | 0 | — | Contrato/OS v1 identificados acima |

Nenhum BUG novo foi confirmado ou corrigido. BUG-005 e BUG-RD-C-001 permanecem verificados nas respectivas auditorias; não foram reabertos por esta análise. Não foram executados build, testes Rust, sensores, suites/campanhas de produção ou medições nativas. RL01–16 e AC-01–10 são obrigações futuras de execução e auditoria na identidade nova.

Uso compartilhado consultado: 24% → 24% na janela de 10080 minutos, sem diferença visível na resolução apresentada; isso não mede tokens exatos nem implica consumo zero.
