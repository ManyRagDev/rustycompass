# Verificação do encoder semântico local

O backend Rust usa CPU/F32, pesos safetensors e tokenizer da revisão
`614241f622f53c4eeff9890bdc4f31cfecc418b3` de
[intfloat/multilingual-e5-small](https://huggingface.co/intfloat/multilingual-e5-small/tree/614241f622f53c4eeff9890bdc4f31cfecc418b3).
A instalação é explícita. Consultas/carregamento não fazem downloads nem enviam texto.
Os SHA256 e tamanhos estão fixados em `semantic.rs`; os bytes usados pelo loader são
os mesmos que foram conferidos. O diretório local do modelo não é parte do código.

`reference.py` é um oráculo separado: Transformers/PyTorch CPU, atenção eager,
pooling médio pela attention mask e normalização L2. A inferência recebe um lote
com padding, enquanto Rust processa sequências sem padding; a comparação cobre
esse detalhe do pooling. `reference.json` contém sete exemplos PT/EN, código com
BOM/CRLF, árabe/japonês e passagem vazia, IDs e 384 componentes completos. O limite
absoluto de erro é `3e-5`, fixado antes da comparação.

O tokenizer é aberto explicitamente por `PreTrainedTokenizerFast(tokenizer_file=...)`
com os special tokens distribuídos. Nesta máquina, `AutoTokenizer` do Transformers
5.17 reescreveu o normalizer e o pretokenizer pinados de XLM-R; um CRLF terminal
produziu um ID diferente. Foi observado comparando os dois JSONs e seus IDs.
Usar o arquivo pinado evita depender dessa reconfiguração de outra versão da
biblioteca. O modelo neural continua sendo a implementação independente BertModel
em PyTorch; nenhuma implementação Rust é importada pelo oráculo.

Reprodução (ambiente Python separado com torch, transformers, tokenizers e safetensors):

```sh
python verification/semantic/reference.py \
  --model .compass/models/multilingual-e5-small \
  --output /tmp/semantic-reference.json
cmp verification/semantic/reference.json /tmp/semantic-reference.json
COMPASS_SEMANTIC_MODEL="$PWD/.compass/models/multilingual-e5-small" \
  RAYON_NUM_THREADS=4 scripts/cargo-local.sh test -p compass-core \
  semantic::model_tests --locked --offline -- --ignored --nocapture
```

O teste de modelo é explicitamente ignorado na suíte padrão: exige pesos instalados.
A suíte normal cobre cosseno, vetores inválidos e cancelamento anterior a qualquer
instalação/carregamento. O teste com pesos confere os IDs/vetores do oráculo,
cobertura contígua dos bytes UTF8, offsets de linhas, orçamento de 512 tokens,
recusa explícita de query grande, hash adulterado e cancelamento.

O segmentador preserva toda a entrada e os offsets brutos, incluindo BOM, CRLF,
espaços e LF final. As unidades opcionais de sintaxe são limites sobre o documento
inteiro, sem simular revisões de subdocumentos. Unidades grandes usam subdivisão
UTF8 determinística até caberem no modelo. O consumidor inclui o produtor/limites
de sintaxe na identidade do derivado. Não há sobreposição nem promessa de que uma
unidade maior que o orçamento preserve uma sentença inteira.
