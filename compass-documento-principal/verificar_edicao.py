"""Conferências editoriais e aritméticas; não executa o Compass."""
from pathlib import Path
from fractions import Fraction
import hashlib, json, re
import sympy as sp

ROOT = Path(__file__).resolve().parent
MD = ROOT / 'COMPASS_DOCUMENTO_PRINCIPAL.md'
text = MD.read_text(encoding='utf-8')
weights = [15,12,12,10,12,7,12,5,5,10]
notes = {'Rust':[5,5,5,5,4,5,5,4,4,3], 'C++':[5,5,5,3,5,4,3,5,3,2],
         'C#':[4,4,4,5,4,4,4,4,5,5], 'Java':[3,3,4,5,4,4,4,4,5,4]}
scores = {k:sum(w*n for w,n in zip(weights,v))/5 for k,v in notes.items()}
assert sum(weights)==100
assert scores=={'Rust':91.6,'C++':81.8,'C#':85.0,'Java':77.6}
W=sp.Matrix([[0,1,sp.Rational(1,10)],[1,0,0],[sp.Rational(1,10),0,0]])
L=sp.diag(*[sum(W.row(i)) for i in range(3)])-W
h=(L+sp.eye(3)).inv()*sp.Matrix([1,0,0])
assert h == sp.Matrix([sp.Rational(22,35),sp.Rational(11,35),sp.Rational(2,35)])
B=[0,0,0,2,1,1,2,0,0,0,0,2]
C0=[2,0,3,0,0,2,0,0,3,3,3,0]
C1=[3,0,3,0,0,2,0,0,3,3,3,0]
avg={k:float(Fraction(sum(v),36)) for k,v in [('B',B),('C0',C0),('C1',C1)]}
assert sum(x>y for x,y in zip(C1,B))==6
assert sum(x<y for x,y in zip(C1,B))==4
assert sum(x==0 for x in C1)==6
sources=['S-CONV','S-HIST','S-P23','S-WP','S-CAD','S-R00','E-RUST','E-MCP','E-PARSER','E-FAER','E-CG','E-EXPM','E-AOT']
for source in sources:
    assert f'**[{source}]' in text, source
for n in range(1,21):
    assert re.search(rf'^# {n}\. ',text,re.M), f'Capítulo {n}'
for label in ['DRIFT-001','RETR-001','HEAT-001','COND-001','AGENT-001','ADR-001']:
    assert label in text
assert not any(x in text for x in ['TODO','TBD','turn778','filecite','<WritingBlock'])
result={
 'scope':'Verificação documental e cálculos didáticos; sem execução do motor ou das campanhas.',
 'word_count':len(text.split()), 'heading_count':len(re.findall(r'^# ',text,re.M)),
 'score_weights_total':sum(weights),'language_scores':scores,
 'weighted_graph_exact_solution':[str(x) for x in h],
 'historical_coverage_recomputed_from_published_table':avg,
 'c1_vs_b':{'wins':6,'losses':4,'ties':2,'zero_coverage_tasks':6},
 'bootstrap_probability_no_T01':float(Fraction(11,12)**12),
 'embeddings_example':{'bytes':100000*384*4,'MB':153.6,'MiB':100000*384*4/2**20},
 'all_source_labels_defined':True,'chapters_1_to_20_present':True,
 'product_executed':False,'scientific_campaign_reproduced':False,
}
(ROOT/'VERIFICACAO_DA_EDICAO.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))
