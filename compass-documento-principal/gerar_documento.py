#!/usr/bin/env python3
"""Gera o PDF da fonte Markdown. Requer Pandoc e XeLaTeX locais."""
from pathlib import Path
import shutil, subprocess, sys

root=Path(__file__).resolve().parent
for executable in ('pandoc','xelatex'):
    if shutil.which(executable) is None:
        raise SystemExit(f'Dependência ausente: {executable}. Instale-a antes de gerar o PDF.')
output=root/'COMPASS_DOCUMENTO_PRINCIPAL.pdf'
cmd=['pandoc',str(root/'COMPASS_DOCUMENTO_PRINCIPAL.md'),'-o',str(output),
     '--pdf-engine=xelatex','--standalone','--table-of-contents','--toc-depth=1',
     '--no-highlight','--lua-filter',str(root/'layout.lua'),'--include-in-header',str(root/'estilo.tex'),
     '-V','documentclass=article','-V','classoption=twoside','-V','papersize=a4',
     '-V','fontsize=11pt','-V','linestretch=1.12',
     '-V','geometry:inner=27mm,outer=22mm,top=24mm,bottom=23mm',
     '-V','mainfont=Liberation Serif','-V','sansfont=Liberation Sans',
     '-V','monofont=DejaVu Sans Mono','-V','monofontoptions=Scale=0.78',
     '-M','toc-title=Sumário']
proc=subprocess.run(cmd,cwd=root,text=True,capture_output=True)
(root/'geracao.log').write_text(proc.stdout+proc.stderr,encoding='utf-8')
if proc.returncode:
    print(proc.stderr,file=sys.stderr)
    raise SystemExit(proc.returncode)
print(f'PDF gerado: {output}')
