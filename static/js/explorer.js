export class Explorer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this._activeFile = null;
    this._onOpenFile = null;
    this._onFilesChanged = null;
    this._dirStack = [];
    this._entries = [];
  }

  onOpenFile(fn) {
    this._onOpenFile = fn;
  }

  onFilesChanged(fn) {
    this._onFilesChanged = fn;
  }

  get activeFile() {
    return this._activeFile;
  }

  get folderName() {
    return this._dirStack.length > 0 ? this._dirStack[0].name : null;
  }

  get hasFolder() {
    return this._dirStack.length > 0;
  }

  get currentPath() {
    return this._dirStack.map(h => h.name).join('/');
  }

  get rootHandle() {
    return this._dirStack.length > 0 ? this._dirStack[0] : null;
  }

  async openFolder(handle) {
    this._dirStack = [handle];
    this._activeFile = null;
    await this._loadCurrentDir();
    if (this._onFilesChanged) this._onFilesChanged();
  }

  async navigateInto(name) {
    const current = this._dirStack[this._dirStack.length - 1];
    const sub = await current.getDirectoryHandle(name);
    this._dirStack.push(sub);
    await this._loadCurrentDir();
  }

  async navigateUp() {
    if (this._dirStack.length <= 1) return;
    this._dirStack.pop();
    await this._loadCurrentDir();
  }

  async _loadCurrentDir() {
    const handle = this._dirStack[this._dirStack.length - 1];
    const dirs = [];
    const files = [];
    for await (const entry of handle.values()) {
      if (entry.kind === 'directory') {
        dirs.push({ name: entry.name, kind: 'directory' });
      } else {
        files.push({ name: entry.name, kind: 'file' });
      }
    }
    dirs.sort((a, b) => a.name.localeCompare(b.name));
    files.sort((a, b) => a.name.localeCompare(b.name));
    this._entries = [...dirs, ...files];
    this._renderLocalFiles();
  }

  async openFile(name) {
    const handle = this._dirStack[this._dirStack.length - 1];
    const fileHandle = await handle.getFileHandle(name);
    const file = await fileHandle.getFile();
    const content = await file.text();
    const path = this.currentPath + '/' + name;
    this._activeFile = path;
    if (this._onOpenFile) this._onOpenFile(path, name, content);
    this._highlightActive(path);
  }

  async saveFile(path, content) {
    const parts = path.split('/');
    const fileName = parts.pop();
    if (parts.length > 0 && parts[0] === this._dirStack[0].name) {
      parts.shift();
    }
    let handle = this.rootHandle;
    for (const p of parts) {
      handle = await handle.getDirectoryHandle(p, { create: true });
    }
    const fileHandle = await handle.getFileHandle(fileName, { create: true });
    const writable = await fileHandle.createWritable();
    await writable.write(content);
    await writable.close();
  }

  async createFile(name) {
    const handle = this._dirStack[this._dirStack.length - 1];
    await handle.getFileHandle(name, { create: true });
    await this._loadCurrentDir();
  }

  async createFolder(name) {
    const handle = this._dirStack[this._dirStack.length - 1];
    await handle.getDirectoryHandle(name, { create: true });
    await this._loadCurrentDir();
  }

  async deleteItem(name) {
    const handle = this._dirStack[this._dirStack.length - 1];
    await handle.removeEntry(name, { recursive: true });
    await this._loadCurrentDir();
  }

  async saveImageFromFile(file, fileName) {
    const handle = this._dirStack[this._dirStack.length - 1];
    const fileHandle = await handle.getFileHandle(fileName, { create: true });
    const writable = await fileHandle.createWritable();
    await writable.write(file);
    await writable.close();
    await this._loadCurrentDir();
  }

  async saveImageFromUrl(url, fileName) {
    const res = await fetch(url, { mode: 'cors' });
    if (!res.ok) throw new Error('Failed to fetch image');
    const blob = await res.blob();
    const handle = this._dirStack[this._dirStack.length - 1];
    const fileHandle = await handle.getFileHandle(fileName, { create: true });
    const writable = await fileHandle.createWritable();
    await writable.write(blob);
    await writable.close();
    await this._loadCurrentDir();
  }

  async duplicateItem(name) {
    const handle = this._dirStack[this._dirStack.length - 1];
    const entry = await handle.getFileHandle(name).catch(() => handle.getDirectoryHandle(name));
    if (entry.kind === 'directory') return;
    const file = await (await handle.getFileHandle(name)).getFile();
    const ext = name.includes('.') ? name.split('.').pop() : '';
    const base = ext ? name.slice(0, -(ext.length + 1)) : name;
    let copyName = `${base} copy.${ext}`;
    let n = 2;
    while (true) {
      try { await handle.getFileHandle(copyName); copyName = `${base} copy ${n}.${ext}`; n++; }
      catch { break; }
    }
    const fh = await handle.getFileHandle(copyName, { create: true });
    const w = await fh.createWritable();
    await w.write(await file.text());
    await w.close();
    await this._loadCurrentDir();
  }

  async renameItem(oldName, newName) {
    const handle = this._dirStack[this._dirStack.length - 1];
    const entry = await handle.getFileHandle(oldName).catch(() => handle.getDirectoryHandle(oldName));
    const isDir = entry.kind === 'directory';

    if (isDir) {
      const oldDir = await handle.getDirectoryHandle(oldName);
      const newDir = await handle.getDirectoryHandle(newName, { create: true });
      for await (const child of oldDir.values()) {
        if (child.kind === 'file') {
          const fh = await oldDir.getFileHandle(child.name);
          const file = await fh.getFile();
          const nh = await newDir.getFileHandle(child.name, { create: true });
          const w = await nh.createWritable();
          await w.write(await file.text());
          await w.close();
        }
      }
      await handle.removeEntry(oldName, { recursive: true });
    } else {
      const oldFile = await handle.getFileHandle(oldName);
      const file = await oldFile.getFile();
      const newFile = await handle.getFileHandle(newName, { create: true });
      const w = await newFile.createWritable();
      await w.write(await file.text());
      await w.close();
      await handle.removeEntry(oldName);
    }
    await this._loadCurrentDir();
  }

  _renderEmpty() {
    this.container.innerHTML = `<div class="explorer-header"><span>Files</span></div>
      <div class="explorer-body"><div class="exp-empty">No workspace folder selected. Open Settings to choose one.</div></div>`;
  }

  _renderLocalFiles() {
    const folderSvg = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';
    const current = this._dirStack[this._dirStack.length - 1];
    const atRoot = this._dirStack.length === 1;
    const displayPath = this.currentPath;

    const backHtml = atRoot ? '' : '<button class="exp-back" id="exp-local-up" title="Go up">&larr;</button>';
    let html = `<div class="explorer-header">
      ${backHtml}
      <span class="exp-label" title="${this._esc(displayPath)}">${this._esc(atRoot ? current.name : displayPath)}</span>
    </div>`;
    html += '<div class="explorer-actions">';
    html += '<button class="exp-btn" id="exp-new-file" title="New file">+ File</button>';
    html += '<button class="exp-btn" id="exp-new-folder" title="New folder">+ Folder</button>';
    html += '</div>';
    html += '<div class="explorer-body"><ul class="exp-tree">';
    for (const e of this._entries) {
      const icon = e.kind === 'directory' ? folderSvg : this._fileIcon(e.name);
      html += `<li class="exp-item" data-name="${this._esc(e.name)}" data-kind="${e.kind}">
        <span class="exp-icon">${icon}</span>
        <span class="exp-label">${this._esc(e.name)}</span>
        <button class="exp-item-btn" title="More actions">&hellip;</button>
      </li>`;
    }
    html += '</ul></div>';
    this.container.innerHTML = html;
    this._bindEvents();
  }

  _bindEvents() {
    document.querySelectorAll('.exp-item').forEach(el => {
      const name = el.dataset.name;
      const kind = el.dataset.kind;
      el.addEventListener('click', (e) => {
        if (e.target.closest('.exp-item-btn')) return;
        if (kind === 'directory') {
          this.navigateInto(name);
        } else {
          this.openFile(name);
        }
      });
      el.addEventListener('dblclick', (e) => {
        if (e.target.closest('.exp-item-btn')) return;
        if (kind === 'directory') {
          this.navigateInto(name);
        } else {
          this.openFile(name);
        }
      });
      el.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        this._showContextMenu(e.clientX, e.clientY, name, kind);
      });
      const btn = el.querySelector('.exp-item-btn');
      if (btn) {
        btn.addEventListener('click', (e) => {
          e.stopPropagation();
          const rect = btn.getBoundingClientRect();
          this._showContextMenu(rect.left, rect.bottom, name, kind);
        });
      }
    });
    const upBtn = document.getElementById('exp-local-up');
    if (upBtn) upBtn.addEventListener('click', () => this.navigateUp());
    const newFileBtn = document.getElementById('exp-new-file');
    if (newFileBtn) newFileBtn.addEventListener('click', () => this._promptNewFile());
    const newFolderBtn = document.getElementById('exp-new-folder');
    if (newFolderBtn) newFolderBtn.addEventListener('click', () => this._promptNewFolder());
  }

  _highlightActive(path) {
    const fileName = path.split('/').pop();
    document.querySelectorAll('.exp-item').forEach(el => {
      el.classList.toggle('exp-active', el.dataset.name === fileName);
    });
  }

  _showContextMenu(x, y, name, kind) {
    const existing = document.querySelector('.exp-context-menu');
    if (existing) existing.remove();

    const menu = document.createElement('div');
    menu.className = 'exp-context-menu';
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';

    const items = [
      { label: 'Rename', action: () => this._promptRename(name, kind) },
      kind === 'file' ? { label: 'Duplicate', action: () => this._promptDuplicate(name) } : null,
      { label: 'Delete', action: () => this._promptDelete(name, kind) },
    ].filter(Boolean);

    for (const item of items) {
      const btn = document.createElement('button');
      btn.textContent = item.label;
      btn.addEventListener('click', () => {
        menu.remove();
        item.action();
      });
      menu.appendChild(btn);
    }
    document.body.appendChild(menu);

    const close = (e) => {
      if (!menu.contains(e.target)) {
        menu.remove();
        document.removeEventListener('click', close);
      }
    };
    setTimeout(() => document.addEventListener('click', close), 0);
  }

  async _promptNewFile() {
    const name = prompt('File name (e.g., section):');
    if (!name || !name.trim()) return;
    let fileName = name.trim();
    if (!fileName.includes('.')) fileName += '.tex';
    try {
      await this.createFile(fileName);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptNewFolder() {
    const name = prompt('Folder name:');
    if (!name || !name.trim()) return;
    try {
      await this.createFolder(name.trim());
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptDuplicate(name) {
    try {
      await this.duplicateItem(name);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptDelete(name, kind) {
    const msg = kind === 'directory' ? `Delete folder "${name}" and all contents?` : `Delete "${name}"?`;
    if (!confirm(msg)) return;
    try {
      await this.deleteItem(name);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptRename(name, kind) {
    const newName = prompt('New name:', name);
    if (!newName || !newName.trim() || newName.trim() === name) return;
    try {
      await this.renameItem(name, newName.trim());
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  _esc(s) {
    const div = document.createElement('div');
    div.textContent = s;
    return div.innerHTML;
  }

  _fileIcon(name) {
    const ext = name.split('.').pop().toLowerCase();
    const icons = {
      tex: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="13" x2="15" y2="13"/><line x1="12" y1="13" x2="12" y2="18"/></svg>',
      pdf: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><path d="M9 15l3 3 6-6"/></svg>',
      png: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>',
      jpg: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>',
      jpeg: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>',
      cls: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>',
      sty: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>',
      bib: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M4 4.5A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5A2.5 2.5 0 0 1 4 19.5v-15z"/><line x1="8" y1="7" x2="16" y2="7"/><line x1="8" y1="11" x2="14" y2="11"/></svg>',
    };
    return icons[ext] || '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>';
  }
}
