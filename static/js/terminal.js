export class Terminal {
  constructor(containerId, copyBtnId, clearBtnId) {
    this.container = document.getElementById(containerId);
    if (!this.container) {
      throw new Error(`Terminal container #${containerId} not found`);
    }

    this.copyBtn = document.getElementById(copyBtnId);
    this.clearBtn = document.getElementById(clearBtnId);

    if (this.copyBtn) {
      this.copyBtn.addEventListener('click', () => this.copy());
    }
    if (this.clearBtn) {
      this.clearBtn.addEventListener('click', () => this.clear());
    }
  }

  _line(lvl, text) {
    const el = document.createElement('div');
    el.className = `terminal-line ${lvl}`;

    const lbl = document.createElement('span');
    lbl.className = 'lvl';
    lbl.textContent = lvl.toUpperCase();

    const msg = document.createElement('span');
    msg.className = 'msg';
    msg.textContent = text;

    el.appendChild(lbl);
    el.appendChild(msg);
    this.container.appendChild(el);
  }

  info(text) { this._line('info', text); }
  success(text) { this._line('ok', text); }
  error(text) { this._line('fail', text); }
  warning(text) { this._line('warn', text); }

  log(text) {
    if (!text) return;
    const el = document.createElement('div');
    el.className = 'terminal-log';
    el.textContent = text;
    this.container.appendChild(el);
  }

  divider() {
    const el = document.createElement('div');
    el.className = 'terminal-divider';
    this.container.appendChild(el);
  }

  clear() {
    this.container.innerHTML = '';
  }

  scrollToBottom() {
    this.container.scrollTop = this.container.scrollHeight;
  }

  async copy() {
    const items = this.container.querySelectorAll(
      '.terminal-line, .terminal-log, .terminal-divider'
    );
    const parts = [];

    for (const el of items) {
      if (el.classList.contains('terminal-line')) {
        const lvl = el.querySelector('.lvl')?.textContent || '';
        const msg = el.querySelector('.msg')?.textContent || '';
        parts.push(`  ${lvl.padEnd(7)}${msg}`);
      } else if (el.classList.contains('terminal-log')) {
        parts.push(el.textContent);
      } else if (el.classList.contains('terminal-divider')) {
        parts.push('─'.repeat(50));
      }
    }

    try {
      await navigator.clipboard.writeText(parts.join('\n'));
    } catch {
      const ta = document.createElement('textarea');
      ta.value = parts.join('\n');
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
    }

    if (this.copyBtn) {
      const orig = this.copyBtn.innerHTML;
      this.copyBtn.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>';
      setTimeout(() => { this.copyBtn.innerHTML = orig; }, 1200);
    }
  }
}
