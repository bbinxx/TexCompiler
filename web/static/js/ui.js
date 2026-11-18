export class UI {
  constructor() {
    this.compileBtn = document.getElementById('compile-btn');
    this.compilerSelect = document.getElementById('compiler');
    this.pdfViewer = document.getElementById('pdf-viewer');
    this.loader = document.getElementById('loader');
    this.pageCount = document.getElementById('page-count');
    this.panes = document.querySelectorAll('.pane');
    this.mobileTabBtns = document.querySelectorAll('.mobile-tab-btn');

    if (!this.compileBtn || !this.compilerSelect || !this.pdfViewer || !this.loader) {
      throw new Error('Required UI elements not found');
    }
  }

  getCompiler() {
    return this.compilerSelect.value;
  }

  hasCompiler(name) {
    for (const opt of this.compilerSelect.options) {
      if (opt.value === name) return true;
    }
    return false;
  }

  setCompiler(name) {
    if (this.hasCompiler(name)) {
      this.compilerSelect.value = name;
    }
  }

  async loadCompilers() {
    try {
      const res = await fetch('/compilers');
      const data = await res.json();
      const compilers = data.compilers || [];
      this.compilerSelect.innerHTML = '';
      compilers.forEach((name) => {
        const opt = document.createElement('option');
        opt.value = name;
        opt.textContent = name;
        this.compilerSelect.appendChild(opt);
      });
      if (compilers.length > 0) {
        this.compilerSelect.value = compilers.includes('pdflatex') ? 'pdflatex' : compilers[0];
      }
    } catch {
      this.compilerSelect.innerHTML = '<option value="pdflatex">pdflatex</option>';
    }
  }

  setCompiling(isCompiling) {
    this.compileBtn.disabled = isCompiling;
    this.compileBtn.classList.toggle('is-loading', isCompiling);
    this.loader.classList.toggle('active', isCompiling);

    if (isCompiling) {
      this.pageCount.textContent = 'Compiling...';
    }
  }

  displayPdf(blob) {
    if (this._currentBlobUrl) {
      URL.revokeObjectURL(this._currentBlobUrl);
    }
    this._currentBlobUrl = URL.createObjectURL(blob);
    this.pdfViewer.src = this._currentBlobUrl;
    this.pageCount.textContent = 'View PDF';
  }

  clearPdf() {
    if (this.pdfViewer.src && this.pdfViewer.src.startsWith('blob:')) {
      URL.revokeObjectURL(this.pdfViewer.src);
    }
    this.pdfViewer.src = 'about:blank';
    this.pageCount.textContent = 'Ready';
  }

  setPageStatus(text) {
    this.pageCount.textContent = text;
  }

  switchTab(paneId) {
    this.panes.forEach((p) => p.classList.remove('active'));
    const targetPane = document.getElementById(paneId);
    if (targetPane) {
      targetPane.classList.add('active');
    }

    this.mobileTabBtns.forEach((btn) => {
      const match = btn.getAttribute('data-pane');
      btn.classList.toggle('active', match === paneId);
    });

    if (paneId === 'editor-pane') {
      const event = new CustomEvent('editor-shown');
      document.dispatchEvent(event);
    }
  }
}
