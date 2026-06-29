import { initEditor, setCode, getCode, focusEditor, onChange, insertAtCursor } from './editor.js';
import { Terminal } from './terminal.js';
import { compileCode } from './compiler.js';
import { Explorer } from './explorer.js';
import { UI } from './ui.js';
import { Settings } from './settings.js';
import { DEFAULT_LATEX } from './defaults.js';
import { detectCompiler, needsUnicodeEngine } from './detector.js';

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

async function handleSave(explorer, currentFilePath, getCode, terminal) {
  if (!explorer.hasFolder) {
    terminal.warning('No workspace folder selected.');
    return;
  }
  if (!currentFilePath) {
    terminal.warning('No file is open.');
    return;
  }
  const code = getCode();
  try {
    await explorer.saveFile(currentFilePath, code);
    terminal.info(`Saved: ${currentFilePath}`);
  } catch {
    terminal.warning('Could not save file');
  }
}

function initApp() {
  const editor = initEditor('editor');
  setCode(DEFAULT_LATEX, -1);
  focusEditor();

  const terminal = new Terminal('terminal-content', 'term-copy', 'term-clear');
  const ui = new UI();
  const settings = new Settings();
  const explorer = new Explorer('explorer-content');
  let manualOverride = false;
  let currentFilePath = null;

  // ---- Save helper ----

  const saveCurrentFile = () => handleSave(explorer, currentFilePath, getCode, terminal);

  // ---- Settings / Folder ----

  settings.onFolderChange(async (handle) => {
    await explorer.openFolder(handle);
    terminal.info(`Opened folder: ${handle.name}`);
  });

  settings.init().then(() => {
    if (!settings.folderName) {
      terminal.info('Open Settings to choose a workspace folder.');
    }
  });

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
    terminal.info('Files updated');
  });

  // ---- Menu toggle ----

  const menuToggle = document.getElementById('menu-toggle');
  const explorerPanel = document.getElementById('explorer-panel');
  menuToggle.addEventListener('click', () => {
    explorerPanel.classList.toggle('open');
  });

  // ---- Save button ----

  document.getElementById('save-btn').addEventListener('click', saveCurrentFile);

  // ---- Image insert ----

  document.getElementById('image-btn').addEventListener('click', (e) => {
    if (!explorer.hasFolder) {
      terminal.warning('No workspace folder selected.');
      return;
    }
    const menu = document.createElement('div');
    menu.className = 'exp-context-menu';
    const rect = e.currentTarget.getBoundingClientRect();
    menu.style.left = rect.left + 'px';
    menu.style.top = (rect.bottom + 4) + 'px';

    const fromFileBtn = document.createElement('button');
    fromFileBtn.textContent = 'From file...';
    fromFileBtn.addEventListener('click', () => {
      menu.remove();
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = 'image/*';
      input.onchange = async () => {
        const file = input.files[0];
        if (!file) return;
        try {
          await explorer.saveImageFromFile(file, file.name);
          const tex = file.name.toLowerCase().endsWith('.pdf') ? `\\includegraphics{${file.name}}` : `\\includegraphics{${file.name}}`;
          insertAtCursor(tex);
          terminal.info(`Inserted image: ${file.name}`);
        } catch (err) {
          terminal.warning('Could not add image: ' + err.message);
        }
      };
      input.click();
    });
    menu.appendChild(fromFileBtn);

    const fromUrlBtn = document.createElement('button');
    fromUrlBtn.textContent = 'From URL...';
    fromUrlBtn.addEventListener('click', () => {
      menu.remove();
      const url = prompt('Image URL:');
      if (!url) return;
      const name = prompt('Save as filename (e.g., image.png):');
      if (!name) return;
      (async () => {
        try {
          await explorer.saveImageFromUrl(url, name);
          const tex = `\\includegraphics{${name}}`;
          insertAtCursor(tex);
          terminal.info(`Inserted image: ${name}`);
        } catch (err) {
          terminal.warning('Could not add image: ' + err.message);
        }
      })();
    });
    menu.appendChild(fromUrlBtn);

    document.body.appendChild(menu);
    const close = (ev) => {
      if (!menu.contains(ev.target)) {
        menu.remove();
        document.removeEventListener('click', close);
      }
    };
    setTimeout(() => document.addEventListener('click', close), 0);
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
  terminal.info('Ready. Ctrl+Enter to compile, Ctrl+S to save.');

  // ---- Compile handler (auto-saves before compile) ----

  async function handleCompile() {
    const code = getCode();
    let compiler = ui.getCompiler();

    if (needsUnicodeEngine(code)) {
      const unicodeCompilers = ['lualatex', 'xelatex'];
      if (!unicodeCompilers.includes(compiler)) {
        const fallback = ui.hasCompiler('lualatex') ? 'lualatex' : 'xelatex';
        terminal.warning(`"${compiler}" cannot compile code using fontspec. Switching to "${fallback}".`);
        compiler = fallback;
        ui.setCompiler(compiler);
      }
    }

    terminal.clear();
    terminal.info('Starting compilation...');
    ui.setCompiling(true);

    if (explorer.hasFolder && currentFilePath) {
      try {
        await explorer.saveFile(currentFilePath, code);
        terminal.info(`Saved: ${currentFilePath}`);
      } catch {
        terminal.warning('Could not save file before compile');
      }
    }

    const result = await compileCode(code, compiler);

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

  // ---- Keyboard shortcuts ----

  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleCompile();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      saveCurrentFile();
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
