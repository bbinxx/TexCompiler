const DB_NAME = 'texcompiler-store';
const DB_VERSION = 1;
const STORE_NAME = 'handles';
const HANDLE_KEY = 'workspace-dir';

function openDB() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE_NAME);
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

async function saveHandle(handle) {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    tx.objectStore(STORE_NAME).put(handle, HANDLE_KEY);
    tx.oncomplete = () => { db.close(); resolve(); };
    tx.onerror = () => { db.close(); reject(tx.error); };
  });
}

async function loadHandle() {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly');
    const req = tx.objectStore(STORE_NAME).get(HANDLE_KEY);
    req.onsuccess = () => { db.close(); resolve(req.result); };
    req.onerror = () => { db.close(); reject(req.error); };
  });
}

export class Settings {
    constructor() {
        this.modal = document.getElementById('settings-modal');
        this.folderBtn = document.getElementById('settings-pick-folder');
        this.folderLabel = document.getElementById('settings-folder-label');
        this.closeBtn = document.getElementById('settings-close');
        this.status = document.getElementById('settings-status');

        this._dirHandle = null;
        this._folderName = null;
        this._onFolderChange = null;
        this._bindEvents();
    }

    get dirHandle() {
        return this._dirHandle;
    }

    get folderName() {
        return this._folderName;
    }

    onFolderChange(fn) {
        this._onFolderChange = fn;
    }

    _bindEvents() {
        document.getElementById('settings-btn').addEventListener('click', () => this.open());
        this.closeBtn.addEventListener('click', () => this.close());
        this.modal.addEventListener('click', (e) => {
            if (e.target === this.modal) this.close();
        });
        this.folderBtn.addEventListener('click', () => this._pickFolder());
    }

    async init() {
        try {
            const handle = await loadHandle();
            if (handle) {
                this._dirHandle = handle;
                this._folderName = handle.name;
                this.folderLabel.textContent = handle.name;
                if (this._onFolderChange) this._onFolderChange(handle);
            }
        } catch {
            // IndexedDB not available or empty
        }
    }

    async _pickFolder() {
        try {
            const handle = await window.showDirectoryPicker({ mode: 'readwrite' });
            this._dirHandle = handle;
            this._folderName = handle.name;
            this.folderLabel.textContent = handle.name;
            await saveHandle(handle);
            this._showStatus(`Opened: ${handle.name}`, 'ok');
            if (this._onFolderChange) this._onFolderChange(handle);
            setTimeout(() => this.close(), 600);
        } catch (e) {
            if (e.name !== 'AbortError' && e.name !== 'SecurityError') {
                this._showStatus('Could not access folder', 'err');
            }
        }
    }

    async open() {
        this.status.textContent = '';
        this.status.className = 'settings-status';
        this.modal.classList.add('active');
        this.modal.style.display = 'flex';
    }

    close() {
        this.modal.classList.remove('active');
        this.modal.style.display = 'none';
    }

    _showStatus(msg, type) {
        this.status.textContent = msg;
        this.status.className = 'settings-status ' + type;
    }
}
