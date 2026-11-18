let editorInstance = null;

export function initEditor(containerId) {
  const el = document.getElementById(containerId);
  if (!el) {
    throw new Error(`Editor container #${containerId} not found`);
  }

  const editor = ace.edit(containerId);
  editor.setTheme('ace/theme/dracula');
  editor.session.setMode('ace/mode/latex');
  editor.setShowPrintMargin(false);
  editor.setOptions({
    fontSize: '13px',
    fontFamily: 'Fira Code, SF Mono, Cascadia Code, monospace',
    enableBasicAutocompletion: false,
    enableLiveAutocompletion: false,
    tabSize: 2,
  });

  editorInstance = editor;
  return editor;
}

export function getCode() {
  return editorInstance ? editorInstance.getValue() : '';
}

export function setCode(code, cursorPos = -1) {
  if (!editorInstance) return;
  editorInstance.setValue(code, cursorPos);
}

export function onChange(fn) {
  if (!editorInstance) return;
  editorInstance.session.on('change', fn);
}

export function resizeEditor() {
  if (editorInstance) {
    editorInstance.resize();
  }
}

export function focusEditor() {
  if (editorInstance) {
    editorInstance.focus();
  }
}

export function insertAtCursor(text) {
  if (!editorInstance) return;
  const session = editorInstance.session;
  const pos = editorInstance.getCursorPosition();
  session.insert(pos, text);
  editorInstance.focus();
}
