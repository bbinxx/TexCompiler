import { initEditor, setCode, getCode, focusEditor, onChange } from './editor.js';
import { Terminal } from './terminal.js';
import { compileCode } from './compiler.js';
import { Explorer } from './explorer.js';
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
  const explorer = new Explorer('explorer-content');
  let manualOverride = false;
  let currentFilePath = null;

  ui.loadCompilers();

  // ---- Explorer callbacks ----

  explorer.onOpenFile((path, name, content) => {
    currentFilePath = path;
    setCode(content, -1);
    document.getElementById('editor-filename').textContent = name;
    document.getElementById('editor-title').textContent = `Editing: ${path}`;
    terminal.info(`Opened: ${path}`);
  });

  explorer.onFilesChanged(() => {
    terminal.info('Project files updated');
  });

  // Load projects on startup
  explorer.loadProjects().then(projects => {
    if (projects.length > 0) {
      terminal.info(`Workspace loaded: ${projects.length} project(s)`);
    }
  });

  // ---- Menu toggle ----

  const menuToggle = document.getElementById('menu-toggle');
  const explorerPanel = document.getElementById('explorer-panel');
  menuToggle.addEventListener('click', () => {
    explorerPanel.classList.toggle('open');
  });

  // ---- Compiler detection ----

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

  // ---- Compile handler ----

  async function handleCompile() {
    const code = getCode();
    const compiler = ui.getCompiler();

    terminal.clear();
    terminal.info('Starting compilation...');
    ui.setCompiling(true);

    if (explorer.activeProject && currentFilePath) {
      try {
        await explorer.saveFile(currentFilePath, code);
        terminal.info(`Saved: ${currentFilePath}`);
      } catch {
        terminal.warning('Could not save file before compile');
      }
    }

    let result;
    if (explorer.activeProject) {
      try {
        const status = await explorer.compileProject(compiler);
        if (status.success && status.pdf_exists) {
          const pdfUrl = explorer.getPdfUrl();
          const pdfRes = await fetch(pdfUrl);
          const blob = await pdfRes.blob();
          result = { success: true, data: blob, warnings: status.warnings, compiler_used: status.compiler_used };
        } else {
          result = { success: false, error: 'Compilation failed', log: '', warnings: status.warnings || [] };
        }
        if (status.compiler_used && status.compiler_used !== compiler) {
          terminal.info(`Compiler auto-switched to: ${status.compiler_used}`);
        }
      } catch (e) {
        result = { success: false, error: e.message, log: '' };
      }
    } else {
      result = await compileCode(code, compiler);
    }

    ui.setCompiling(false);

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
      terminal.error(result.error || 'Compilation failed');
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
