export class Explorer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.projectName = null;
    this._activeFile = null;
    this._onOpenFile = null;
    this._onFilesChanged = null;
  }

  onOpenFile(fn) {
    this._onOpenFile = fn;
  }

  onFilesChanged(fn) {
    this._onFilesChanged = fn;
  }

  async loadProjects() {
    try {
      const res = await fetch('/workspace/projects');
      const data = await res.json();
      this._renderProjectList(data.projects);
      return data.projects;
    } catch {
      this._renderError('Could not load workspace');
      return [];
    }
  }

  async openProject(name) {
    this.projectName = name;
    try {
      const res = await fetch(`/workspace/projects/${encodeURIComponent(name)}/files`);
      const data = await res.json();
      this._renderFileTree(data.files, name);
    } catch {
      this._renderError('Could not load project files');
    }
  }

  async createProject(name) {
    const res = await fetch('/workspace/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });
    if (!res.ok) throw new Error((await res.json()).detail || 'Create failed');
    return res.json();
  }

  async deleteProject(name) {
    const res = await fetch(`/workspace/projects/${encodeURIComponent(name)}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Delete failed');
  }

  async createFile(path) {
    const res = await fetch(
      `/workspace/projects/${encodeURIComponent(this.projectName)}/files/${encodeURIComponent(path)}`,
      { method: 'POST' }
    );
    if (!res.ok) throw new Error('Create file failed');
  }

  async deleteItem(path) {
    const res = await fetch(
      `/workspace/projects/${encodeURIComponent(this.projectName)}/files/${encodeURIComponent(path)}`,
      { method: 'DELETE' }
    );
    if (!res.ok) throw new Error('Delete failed');
  }

  async renameItem(path, newPath) {
    const res = await fetch(
      `/workspace/projects/${encodeURIComponent(this.projectName)}/rename`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path, new_path: newPath }),
      }
    );
    if (!res.ok) throw new Error('Rename failed');
  }

  async openFile(path) {
    this._activeFile = path;
    const res = await fetch(
      `/workspace/projects/${encodeURIComponent(this.projectName)}/files/${encodeURIComponent(path)}`
    );
    if (!res.ok) throw new Error('Read failed');
    const content = await res.text();
    const name = path.split('/').pop() || path;
    if (this._onOpenFile) this._onOpenFile(path, name, content);
    this._highlightActive(path);
  }

  async saveFile(path, content) {
    const res = await fetch(
      `/workspace/projects/${encodeURIComponent(this.projectName)}/files/${encodeURIComponent(path)}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path, content }),
      }
    );
    if (!res.ok) throw new Error('Save failed');
  }

  async compileProject(compiler) {
    const res = await fetch(`/workspace/projects/${encodeURIComponent(this.projectName)}/compile?compiler=${compiler}`);
    if (!res.ok) throw new Error('Compile failed');
    return res.json();
  }

  getPdfUrl() {
    return `/workspace/projects/${encodeURIComponent(this.projectName)}/pdf`;
  }

  get activeFile() {
    return this._activeFile;
  }

  get activeProject() {
    return this.projectName;
  }

  // ---- Rendering ----

  _renderProjectList(projects) {
    let html = '<div class="explorer-header"><span>Projects</span></div>';
    html += '<div class="explorer-actions"><button class="exp-btn" id="new-project-btn" title="New project">+ New</button></div>';
    html += '<div class="explorer-body">';
    if (projects.length === 0) {
      html += '<div class="exp-empty">No projects yet. Click "+ New" to create one.</div>';
    } else {
      html += '<ul class="exp-tree">';
      for (const p of projects) {
        html += `<li class="exp-project" data-project="${p.name}">
          <span class="exp-icon">&#128193;</span>
          <span class="exp-label">${this._esc(p.name)}</span>
          <span class="exp-meta">${p.files} .tex</span>
        </li>`;
      }
      html += '</ul>';
    }
    html += '</div>';
    this.container.innerHTML = html;
    this._bindProjectEvents();
  }

  _renderFileTree(files, project) {
    let html = `<div class="explorer-header">
      <button class="exp-back" id="exp-back-btn" title="Back to projects">&larr;</button>
      <span>${this._esc(project)}</span>
    </div>`;
    html += '<div class="explorer-actions">';
    html += '<button class="exp-btn" id="exp-new-file" title="New file">+ File</button>';
    html += '<button class="exp-btn" id="exp-new-folder" title="New folder">+ Folder</button>';
    html += '</div>';
    html += '<div class="explorer-body"><ul class="exp-tree">';
    for (const f of files) {
      const icon = f.type === 'dir' ? '&#128193;' : this._fileIcon(f.name);
      html += `<li class="exp-item" data-path="${this._esc(f.path)}" data-type="${f.type}">
        <span class="exp-icon">${icon}</span>
        <span class="exp-label">${this._esc(f.name)}</span>
        <span class="exp-meta">${f.type === 'file' ? this._fileSize(f.size) : ''}</span>
      </li>`;
    }
    html += '</ul></div>';
    this.container.innerHTML = html;
    this._bindFileEvents();
  }

  _renderError(msg) {
    this.container.innerHTML = `<div class="explorer-header"><span>Error</span></div>
      <div class="explorer-body"><div class="exp-empty">${this._esc(msg)}</div></div>`;
  }

  _bindProjectEvents() {
    document.querySelectorAll('.exp-project').forEach(el => {
      el.addEventListener('click', () => {
        const name = el.dataset.project;
        this.openProject(name);
      });
      el.addEventListener('dblclick', () => {
        const name = el.dataset.project;
        this.openProject(name);
      });
    });
    const newBtn = document.getElementById('new-project-btn');
    if (newBtn) {
      newBtn.addEventListener('click', () => this._promptCreateProject());
    }
  }

  _bindFileEvents() {
    document.querySelectorAll('.exp-item').forEach(el => {
      el.addEventListener('click', () => {
        const path = el.dataset.path;
        const type = el.dataset.type;
        if (type === 'file') {
          this.openFile(path);
        }
      });
      el.addEventListener('dblclick', () => {
        const path = el.dataset.path;
        const type = el.dataset.type;
        if (type === 'file') {
          this.openFile(path);
        }
      });
      el.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        this._showContextMenu(e.clientX, e.clientY, el.dataset.path, el.dataset.type);
      });
    });
    const backBtn = document.getElementById('exp-back-btn');
    if (backBtn) backBtn.addEventListener('click', () => this.loadProjects());
    const newFileBtn = document.getElementById('exp-new-file');
    if (newFileBtn) newFileBtn.addEventListener('click', () => this._promptCreateFile());
    const newFolderBtn = document.getElementById('exp-new-folder');
    if (newFolderBtn) newFolderBtn.addEventListener('click', () => this._promptCreateFolder());
  }

  _highlightActive(path) {
    document.querySelectorAll('.exp-item').forEach(el => {
      el.classList.toggle('exp-active', el.dataset.path === path);
    });
  }

  _showContextMenu(x, y, path, type) {
    const existing = document.querySelector('.exp-context-menu');
    if (existing) existing.remove();

    const menu = document.createElement('div');
    menu.className = 'exp-context-menu';
    menu.style.left = x + 'px';
    menu.style.top = y + 'px';

    const items = [
      { label: 'Rename', action: () => this._promptRename(path) },
      type === 'file' ? { label: 'Delete', action: () => this._promptDelete(path, false) } : null,
      type === 'dir' ? { label: 'Delete folder', action: () => this._promptDelete(path, true) } : null,
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

  async _promptCreateProject() {
    const name = prompt('Project name:');
    if (!name || !name.trim()) return;
    try {
      await this.createProject(name.trim());
      await this.loadProjects();
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptCreateFile() {
    const name = prompt('File name (e.g., section.tex):');
    if (!name || !name.trim()) return;
    try {
      await this.createFile(name.trim());
      await this.openProject(this.projectName);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptCreateFolder() {
    const name = prompt('Folder name:');
    if (!name || !name.trim()) return;
    try {
      const res = await fetch(
        `/workspace/projects/${encodeURIComponent(this.projectName)}/files/${encodeURIComponent(name.trim())}`,
        { method: 'POST' }
      );
      if (!res.ok) throw new Error('Create failed');
      await this.openProject(this.projectName);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptDelete(path, isDir) {
    const msg = isDir ? `Delete folder "${path}" and all contents?` : `Delete "${path}"?`;
    if (!confirm(msg)) return;
    try {
      await this.deleteItem(path);
      await this.openProject(this.projectName);
      if (this._onFilesChanged) this._onFilesChanged();
    } catch (e) {
      alert(e.message);
    }
  }

  async _promptRename(path) {
    const newName = prompt('New name:', path.split('/').pop());
    if (!newName || !newName.trim()) return;
    const parts = path.split('/');
    parts[parts.length - 1] = newName.trim();
    const newPath = parts.join('/');
    try {
      await this.renameItem(path, newPath);
      await this.openProject(this.projectName);
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
      tex: '&#120514;',
      pdf: '&#128196;',
      png: '&#128247;',
      jpg: '&#128247;',
      jpeg: '&#128247;',
      cls: '&#9881;',
      sty: '&#9881;',
      bib: '&#128214;',
    };
    return icons[ext] || '&#128196;';
  }

  _fileSize(bytes) {
    if (bytes === 0) return '';
    if (bytes < 1024) return bytes + 'B';
    return (bytes / 1024).toFixed(1) + 'KB';
  }
}
