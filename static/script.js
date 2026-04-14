const editor = document.getElementById('editor');
const compileBtn = document.getElementById('compile-btn');
const pdfViewer = document.getElementById('pdf-viewer');
const terminal = document.getElementById('terminal-content');
const loader = document.getElementById('loader');
const compilerSelect = document.getElementById('compiler');

async function compile() {
    const code = editor.value;
    const compiler = compilerSelect.value;
    
    // Clear terminal
    terminal.innerHTML = 'Compiling...';
    compileBtn.disabled = true;
    loader.classList.add('active');
    
    try {
        const response = await fetch('/compile', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ code, compiler }),
        });
        
        if (response.ok) {
            const blob = await response.blob();
            const url = URL.createObjectURL(blob);
            pdfViewer.src = url;
            terminal.innerHTML = '<span style="color: #4ade80;">Success! PDF generated.</span>';
        } else {
            const errorData = await response.json();
            terminal.innerHTML = `<span class="log-error">Error: ${errorData.error}</span>\n\n${errorData.log || ''}`;
        }
    } catch (error) {
        terminal.innerHTML = `<span class="log-error">Fetch Error: ${error.message}</span>`;
    } finally {
        compileBtn.disabled = false;
        loader.classList.remove('active');
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

\\\\title{Antigravity TexCompiler Demo}
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

editor.value = defaultLatex.replace(/\\\\/g, '\\');
