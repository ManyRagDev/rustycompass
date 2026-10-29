# Compass D0 — Documento principal, edição 1.0

O conteúdo principal está em `COMPASS_DOCUMENTO_PRINCIPAL.md`.
`COMPASS_DOCUMENTO_PRINCIPAL.pdf` é a versão de leitura/impressão gerada dessa fonte.
Não manter os dois conteúdos manualmente em paralelo.

## Arquivos

- `COMPASS_DOCUMENTO_PRINCIPAL.md`: documento canônico editável.
- `COMPASS_DOCUMENTO_PRINCIPAL.pdf`: cópia de leitura da edição.
- `gerar_documento.py`, `estilo.tex`, `layout.lua`: composição do PDF.
- `verificar_edicao.py`: conferências editoriais e cálculos didáticos.
- `VERIFICACAO_DA_EDICAO.json`: resultados dessas conferências.
- `MANIFESTO_EDICAO.json`: identificação dos entregáveis e limites da verificação.

Esses arquivos de apoio não são planos adicionais do projeto. Nenhum arquivo de
código do Compass legado foi incluído. Nenhum motor novo foi implementado aqui.

## Regenerar o PDF

Requisitos locais: Python 3, Pandoc, XeLaTeX e os pacotes LaTeX indicados em
`estilo.tex` (incluindo fancyhdr, titlesec, fvextra, lastpage e xurl).
Fontes de sistema utilizadas: Liberation Serif, Liberation Sans e DejaVu Sans Mono.
Nenhum arquivo de fonte é distribuído neste pacote.

Na pasta desta entrega:

```bash
python gerar_documento.py
```

O programa usa as ferramentas locais, sem enviar o documento a um serviço externo.
A versão exata das ferramentas pode alterar paginação e bytes do PDF. Regenerar
conteúdo equivalente não promete igualdade binária de PDFs.

Para repetir as conferências aritméticas/editoriais, o ambiente precisa de SymPy:

```bash
python verificar_edicao.py
```

Após alterações, revisar o documento, gerar uma nova edição e produzir um novo
manifesto. O manifesto desta edição não é válido para arquivos editados depois.

## Uso no novo projeto

A edição registra decisões de direção, hipóteses, resultados históricos e pendências.
Não equivale à aprovação de todos os futuros schemas, algoritmos ou parâmetros.
O próximo trabalho delimitado é o contrato dos estados/revisões do Compass Foundation.
