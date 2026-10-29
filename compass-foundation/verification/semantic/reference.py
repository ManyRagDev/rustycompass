#!/usr/bin/env python3
"""Independent HF Transformers/PyTorch oracle; production never imports this.
Run with an existing local pinned model; HF_HUB_OFFLINE forbids downloads.
"""
import argparse, hashlib, json, os
from pathlib import Path
os.environ['HF_HUB_OFFLINE']='1'
os.environ['TRANSFORMERS_OFFLINE']='1'
import torch
import transformers
from transformers import AutoModel, PreTrainedTokenizerFast

CASES = [
    ('query', 'Onde fica a sessão de autenticação?'),
    ('query', 'How can I recover my password?'),
    ('passage', 'A sessão do usuário expira após trinta minutos. Renove o token para continuar.'),
    ('passage', 'Para recuperar a senha, envie um link de redefinição para o email cadastrado.'),
    ('passage', '\ufefffn somar(a: i32, b: i32) -> i32 {\r\n    a + b // café ☕\r\n}\r\n'),
    ('passage', '東京都の駅と鉄道路線。 البحث في الوثائق المحلية. Δοκιμή.'),
    ('passage', ''),
]

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--model',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);a=ap.parse_args()
    torch.set_num_threads(4)
    special=json.loads((a.model/'special_tokens_map.json').read_text())
    tokenizer=PreTrainedTokenizerFast(tokenizer_file=str(a.model/'tokenizer.json'),**special)
    model=AutoModel.from_pretrained(a.model,local_files_only=True,attn_implementation='eager').eval().to('cpu')
    texts=[f'{kind}: {text}' if kind=='query' else f'passage: {text}' for kind,text in CASES]
    encoded=tokenizer(texts,padding=True,truncation=False,return_tensors='pt')
    with torch.inference_mode():
        hidden=model(**encoded).last_hidden_state
        mask=encoded['attention_mask'].unsqueeze(-1).float()
        pooled=(hidden*mask).sum(1)/mask.sum(1)
        vectors=torch.nn.functional.normalize(pooled,p=2,dim=1)
    cases=[]
    for i,(kind,text) in enumerate(CASES):
        cases.append(dict(kind=kind,text=text,input_ids=tokenizer(texts[i],truncation=False)['input_ids'],vector=vectors[i].tolist()))
    value=dict(producer='independent Hugging Face Transformers / PyTorch CPU eager attention; exact pinned tokenizer.json',torch=torch.__version__,transformers=transformers.__version__,revision='614241f622f53c4eeff9890bdc4f31cfecc418b3',model_sha256=hashlib.sha256((a.model/'model.safetensors').read_bytes()).hexdigest(),absolute_tolerance=0.00003,cases=cases)
    a.output.write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in value.items() if k!='cases'},indent=2))
if __name__=='__main__':main()
