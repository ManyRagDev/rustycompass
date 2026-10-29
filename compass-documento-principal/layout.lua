-- Ajustes editoriais: larguras de tabelas e pequenos blocos indivisíveis.
function Table(t)
  local n = #t.colspecs
  local first = ''
  local last = ''
  if t.head and t.head.rows and #t.head.rows > 0 then
    local c=t.head.rows[1].cells
    if #c>0 then first=pandoc.utils.stringify(c[1].contents);last=pandoc.utils.stringify(c[#c].contents) end
  end
  local w
  if n==6 then w={0.50,0.10,0.10,0.10,0.10,0.10}
  elseif n==5 then w={0.40,0.15,0.15,0.15,0.15}
  elseif n==4 then w={0.43,0.19,0.19,0.19}
  elseif n==3 then
    if last=='Seções' then w={0.14,0.72,0.14}
    elseif first=='ID' then w={0.20,0.46,0.34}
    elseif first:find('Tema') then w={0.45,0.15,0.40}
    else w={0.30,0.36,0.34} end
  elseif n==2 then w={0.37,0.63}
  end
  if w then for i=1,n do t.colspecs[i][2]=w[i] end end
  return t
end
function CodeBlock(b)
  return {pandoc.RawBlock('latex','\\begin{samepage}'),b,pandoc.RawBlock('latex','\\end{samepage}')}
end
function Header(h)
  if h.level==1 then
    local title=pandoc.utils.stringify(h)
    if title:match('^Apêndice F') then
      return {pandoc.RawBlock('latex','\\clearpage'),h,pandoc.RawBlock('latex','\\begingroup\\small\\setlength{\\parskip}{3pt}')}
    elseif title:match('^Apêndice G') then
      return {pandoc.RawBlock('latex','\\endgroup\\clearpage'),h}
    elseif title:match('^Apêndice') then
      return {pandoc.RawBlock('latex','\\clearpage'),h}
    end
  end
end
