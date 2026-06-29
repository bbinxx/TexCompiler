const LUALATEX_PKG = /\\usepackage(\[.*?\])?\{(luacode|luacolor|luamplib|luatexja|luabidi|luaplot)\}/;
const XELATEX_PKG = /\\usepackage(\[.*?\])?\{(xeCJK|xepersian)\}/;
const FONTSPEC_PKG = /\\usepackage(\[.*?\])?\{fontspec\}/;
const FONTSPEC_CMD = /\\(setmainfont|setsansfont|setmonofont|newfontfamily)\{/;

export function detectCompiler(code) {
  if (LUALATEX_PKG.test(code)) return 'lualatex';
  if (XELATEX_PKG.test(code)) return 'xelatex';
  if (FONTSPEC_PKG.test(code) || FONTSPEC_CMD.test(code)) return 'lualatex';
  return null;
}

export function needsUnicodeEngine(code) {
  return FONTSPEC_PKG.test(code) || FONTSPEC_CMD.test(code);
}
