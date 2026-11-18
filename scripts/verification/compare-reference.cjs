// Migration-only harness: both endpoints must be local trusted-fixture servers.
const {readFileSync,writeFileSync,mkdtempSync,rmSync} = require('node:fs');
const {execFileSync} = require('node:child_process');
const {createHash} = require('node:crypto');
const assert = require('node:assert/strict');
const root = mkdtempSync('/tmp/texcompiler-comparison-');
const source = name => readFileSync(`fixtures/${name}`,'utf8');
const endpoints = {python:process.env.TEXCOMPILER_REFERENCE_URL||'http://127.0.0.1:8120',rust:process.env.TEXCOMPILER_URL||'http://127.0.0.1:8119'};
for(const endpoint of Object.values(endpoints)) assert(['127.0.0.1','localhost','[::1]'].includes(new URL(endpoint).hostname),'reference comparison is loopback-only');
const records=[];
const cases = ['pdflatex','xelatex','lualatex'].map(compiler=>({name:`basic-${compiler}`,payload:{code:source('basic/main.tex'),compiler}}));
cases.push({name:'multi-file',payload:{main_file:'main.tex',files:{'main.tex':source('multi-file/main.tex'),'body.tex':source('multi-file/body.tex')}}});
cases.push({name:'resume-class-package-tikz-table',payload:{main_file:'main.tex',files:{'main.tex':source('resume/main.tex'),'resume.cls':Buffer.from(source('resume/resume.cls')).toString('base64'),'details.sty':Buffer.from(source('resume/details.sty')).toString('base64')}}});
cases.push({name:'cover-letter',payload:{code:source('cover-letter/main.tex')}});
cases.push({name:'jobname',payload:{main_file:'resume.tex',files:{'resume.tex':'\\documentclass{article}\\begin{document}Job: \\jobname\\end{document}'}}});
(async()=>{
  try {
    for(const entry of cases) {
      const outputs={};
      for(const [implementation,url] of Object.entries(endpoints)) {
        const start=performance.now();
        const headers={'Content-Type':'application/json'};
        if(implementation==='rust' && process.env.TEXCOMPILER_API_TOKEN)headers.Authorization=`Bearer ${process.env.TEXCOMPILER_API_TOKEN}`;
        const response=await fetch(url+'/compile',{method:'POST',headers,body:JSON.stringify(entry.payload)});
        if(!response.ok)throw new Error(`${entry.name} ${implementation}: ${await response.text()}`);
        const pdf=Buffer.from(await response.arrayBuffer());
        const milliseconds=performance.now()-start;
        const path=`${root}/${implementation}.pdf`;writeFileSync(path,pdf);
        const info=execFileSync('pdfinfo',[path],{encoding:'utf8'});
        const pages=Number(info.match(/^Pages:\s+(\d+)/m)?.[1]);
        const text=execFileSync('pdftotext',[path,'-'],{encoding:'utf8'}).replace(/\s+/g,' ').trim();
        const fonts=execFileSync('pdffonts',[path],{encoding:'utf8'}).trim().split('\n').slice(2).map(line=>line.trim().split(/\s+/)[0].replace(/^[A-Z]{6}\+/, '')).sort();
        const images=[];
        for(let page=1;page<=pages;page++)images.push(createHash('sha256').update(execFileSync('pdftoppm',['-r','72','-f',String(page),'-l',String(page),'-singlefile','-gray',path],{maxBuffer:16*1024*1024})).digest('hex'));
        outputs[implementation]={pages,text,fonts,images,milliseconds,pdfBytes:pdf.length};
      }
      for(const key of ['pages','text','fonts','images'])assert.deepEqual(outputs.rust[key],outputs.python[key],`${entry.name}: ${key}`);
      records.push({fixture:entry.name,checks:{pages:true,text:true,fontNames:true,pixelRendering72dpi:true},timing: {pythonMs:outputs.python.milliseconds,rustMs:outputs.rust.milliseconds},pages:outputs.rust.pages,fonts:outputs.rust.fonts});
    }
    const report={environment:execFileSync('uname',['-srm'],{encoding:'utf8'}).trim(),texVersion:execFileSync('pdflatex',['--version'],{encoding:'utf8'}).split('\n')[0],transport:'HTTP loopback for both implementations',notes:'Sequential single samples; Rust includes isolation and three TeX passes, Python one pass. These are compatibility observations, not a statistically controlled speedup claim.',records};
    writeFileSync(process.env.TEXCOMPILER_COMPARISON_OUTPUT||'/tmp/texcompiler-native-comparison.json',JSON.stringify(report,null,2)+'\n');
    console.log(JSON.stringify({fixtures:records.length,allEquivalent:true}));
  } finally {rmSync(root,{recursive:true,force:true});}
})().catch(error=>{console.error(error.message);process.exitCode=1});
