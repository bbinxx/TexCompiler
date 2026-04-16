const LUALATEX_PKG = /\\usepackage(\[.*?\])?\{(luacode|luacolor|luamplib|luatexja|luabidi|luaplot)\}/;
const XELATEX_PKG = /\\usepackage(\[.*?\])?\{(xeCJK|xepersian)\}/;
const UNICODE_PKG = /\\usepackage(\[.*?\])?\{fontspec\}/;

export function detectCompiler(code) {
  if (LUALATEX_PKG.test(code)) return 'lualatex';
  if (XELATEX_PKG.test(code)) return 'xelatex';
  if (UNICODE_PKG.test(code)) return 'xelatex';
  return null;
}
