const aceEditor = ace.edit("editor");
aceEditor.setTheme("ace/theme/dracula");
aceEditor.session.setMode("ace/mode/latex");
aceEditor.setShowPrintMargin(false);
aceEditor.setOptions({
    fontSize: "14px",
    enableBasicAutocompletion: true,
    fontFamily: "Fira Code, monospace"
});

const compileBtn = document.getElementById('compile-btn');
const pdfViewer = document.getElementById('pdf-viewer');
const terminal = document.getElementById('terminal-content');
const loader = document.getElementById('loader');
const compilerSelect = document.getElementById('compiler');
const pageCount = document.getElementById('page-count');

async function compile() {
    const code = aceEditor.getValue();
    const compiler = compilerSelect.value;
    
    // Clear terminal
    terminal.innerHTML = '<span class="log-info">🚀 Starting compilation...</span><br>';
    compileBtn.disabled = true;
    loader.classList.add('active');
    pageCount.textContent = 'Compiling...';
    
    let isOk = false;
    try {
        const response = await fetch('/compile', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ code, compiler }),
        });
        
        if (response.ok) {
            isOk = true;
            const blob = await response.blob();
            const url = URL.createObjectURL(blob);
            pdfViewer.src = url;
            terminal.innerHTML += '<span class="log-success">✅ Success! PDF generated successfully.</span><br>';
            
            // Extract warnings if any
            // Note: Since it's a blob response, we don't get the JSON warnings.
            // In a real app, you might want to send warnings in a header.
            pageCount.textContent = 'View PDF';
        } else {
            const errorData = await response.json();
            terminal.innerHTML += `<span class="log-error">❌ Error: ${errorData.error}</span><br><br>`;
            if (errorData.log) {
                terminal.innerHTML += `<div style="opacity: 0.8; font-size: 0.9em;">${errorData.log}</div>`;
            }
            pageCount.textContent = 'Failed';
        }
    } catch (error) {
        terminal.innerHTML += `<span class="log-error">❌ System Error: ${error.message}</span>`;
        pageCount.textContent = 'Error';
    } finally {
        compileBtn.disabled = false;
        loader.classList.remove('active');
        terminal.scrollTop = terminal.scrollHeight;
        
        // Auto-switch to preview on mobile after compilation
        if (window.innerWidth <= 900 && isOk) {
            switchTab('preview-pane');
        }
    }
}

function switchTab(paneId) {
    // Switch panes
    document.querySelectorAll('.pane').forEach(p => p.classList.remove('active'));
    document.getElementById(paneId).classList.add('active');
    
    // Switch buttons
    document.querySelectorAll('.mobile-tab-btn').forEach(b => {
        b.classList.toggle('active', b.getAttribute('onclick').includes(paneId));
    });

    // Resize Ace Editor if it became visible
    if (paneId === 'editor-pane') {
        aceEditor.resize();
    }
}

compileBtn.addEventListener('click', compile);

// Initial focus
editor.focus();

// Add some default LaTeX code
const defaultLatex = `\\\\documentclass{article}
\\\\usepackage[utf8]{inputenc}
\\\\usepackage[margin=1in]{geometry}
\\\\usepackage{hyperref}

\\\\title{TexCompiler Demo}
\\\\author{AI Assistant}
\\\\date{\\\\today}

\\\\begin{document}

\\\\maketitle

\\\\section{Introduction}
This is a live demonstration of the \\\\textbf{TexCompiler API}. 
You can edit this LaTeX code on the left and see the results on the right.

\\\\section{Features}
\\\\begin{itemize}
    \\\\item FastAPI Backend
    \\\\item pdflatex / xelatex support
    \\\\item Automatic multiple passes via latexmk
    \\\\item RESTful API for integration
\\\\end{itemize}

\\\\section{Mathematics}
Euler's identity is a beautiful formula:
\\\\[ e^{i\\\\pi} + 1 = 0 \\\\]

\\\\end{document}`;

aceEditor.setValue(defaultLatex.replace(/\\\\/g, '\\'), -1);
aceEditor.focus();
