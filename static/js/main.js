import { initEditor, setCode, getCode, focusEditor, onChange } from './editor.js';
import { Terminal } from './terminal.js';
import { compileCode } from './compiler.js';
import { UI } from './ui.js';
import { DEFAULT_LATEX } from './defaults.js';
import { detectCompiler } from './detector.js';

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

function initApp() {
  const editor = initEditor('editor');
  setCode(DEFAULT_LATEX, -1);
  focusEditor();

  const terminal = new Terminal('terminal-content', 'term-copy', 'term-clear');
  const ui = new UI();
  let manualOverride = false;

  ui.loadCompilers();

  function runDetect() {
    const code = getCode();
    const detected = detectCompiler(code);
    if (detected && !manualOverride) {
      const current = ui.getCompiler();
      if (detected !== current && ui.hasCompiler(detected)) {
        ui.setCompiler(detected);
        terminal.info(`Auto-detected compiler: ${detected}`);
      }
    }
  }

  const debouncedDetect = debounce(runDetect, 600);

  onChange(() => {
    manualOverride = false;
    debouncedDetect();
  });

  ui.compilerSelect.addEventListener('change', () => {
    manualOverride = true;
  });

  runDetect();
  terminal.info('Ready. Press Ctrl+Enter to compile.');

  async function handleCompile() {
    const code = getCode();
    const compiler = ui.getCompiler();

    terminal.clear();
    terminal.info('Starting compilation...');
    ui.setCompiling(true);

    const result = await compileCode(code, compiler);

    ui.setCompiling(false);
    terminal.scrollToBottom();

    if (result.success) {
      terminal.success('PDF generated successfully.');
      if (result.warnings && result.warnings.length > 0) {
        result.warnings.forEach((w) => terminal.warning(w));
      }
      ui.displayPdf(result.data);

      if (window.innerWidth <= 900) {
        ui.switchTab('preview-pane');
      }
    } else {
      terminal.error(result.error);
      if (result.log) {
        terminal.divider();
        terminal.log(result.log);
      }
      ui.setPageStatus('Failed');
    }
  }

  ui.compileBtn.addEventListener('click', handleCompile);

  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleCompile();
    }
  });

  document.addEventListener('editor-shown', () => {
    window.dispatchEvent(new Event('resize'));
  });

  ui.mobileTabBtns.forEach((btn) => {
    btn.addEventListener('click', () => {
      const paneId = btn.getAttribute('data-pane');
      if (paneId) {
        ui.switchTab(paneId);
      }
    });
  });
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
