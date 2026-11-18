const {execFileSync} = require('node:child_process');
const {readFileSync} = require('node:fs');
const assert = require('node:assert/strict');
const base = `http://127.0.0.1:${process.env.TEXCOMPILER_PORT || 8001}`;
const token = process.env.TEXCOMPILER_API_TOKEN;
const compose = (...args) => execFileSync('docker',['compose','-p','texcompiler-ci',...args],{encoding:'utf8'}).trim();
const sleep = ms => new Promise(r=>setTimeout(r,ms));
const source = name => readFileSync(`fixtures/${name}`,'utf8');
(async()=>{
  const hostNamespaceLimit = readFileSync('/proc/sys/user/max_user_namespaces','utf8');
  const cid = compose('ps','-q','texcompiler');
  assert(cid,'container is not running');
  const inspect = JSON.parse(execFileSync('docker',['inspect',cid],{encoding:'utf8'}))[0];
  assert.equal(inspect.Config.User,'10001:10001');
  assert.equal(inspect.HostConfig.ReadonlyRootfs,true);
  assert.equal(inspect.HostConfig.PidsLimit,128);
  assert(inspect.HostConfig.Memory > 0 && inspect.HostConfig.Memory <= 3*1024**3);
  assert.deepEqual(inspect.HostConfig.CapDrop,['ALL']);
  assert.equal(inspect.HostConfig.NanoCpus,2_000_000_000);
  assert(inspect.HostConfig.SecurityOpt.some(value=>value.startsWith('no-new-privileges')));
  assert(inspect.HostConfig.SecurityOpt.includes('systempaths=unconfined'));
  assert(inspect.HostConfig.Tmpfs['/tmp'].includes('size='));
  execFileSync('docker',['exec',cid,'/bin/sh','-ec','! command -v python; ! command -v python3; ! command -v pip; test ! -d /usr/lib/python3; test ! -d /usr/lib/python3.12; ! touch /app/should-be-read-only']);
  for(let i=0;i<180;i++) {
    try { const response = await fetch(base+'/ready'); if(response.ok)break; } catch {}
    if(i===179) throw new Error('native container never became ready');
    await sleep(1000);
  }
  const health = await (await fetch(base+'/health')).json();
  assert.equal(health.compilation_ready,true);
  assert.deepEqual(health.compilers,['lualatex','pdflatex','xelatex']);
  const unauthorized = await fetch(base+'/compile',{method:'POST',headers:{'Content-Type':'application/json'},body:'{}'});
  assert.equal(unauthorized.status,401);
  let count = 0;
  async function compile(payload, expected) {
    const response = await fetch(base+'/compile',{method:'POST',headers:{'Content-Type':'application/json',Authorization:`Bearer ${token}`},body:JSON.stringify(payload)});
    if(!response.ok) throw new Error(`compile failed ${response.status}: ${await response.text()}`);
    assert.equal(response.headers.get('content-type'),'application/pdf');
    const bytes = Buffer.from(await response.arrayBuffer());
    assert.equal(bytes.subarray(0,5).toString(),'%PDF-');
    const text = execFileSync('docker',['exec','-i',cid,'pdftotext','-','-'],{input:bytes}).toString();
    assert(text.includes(expected),text);
    count++;
    return bytes;
  }
  for(const compiler of ['pdflatex','xelatex','lualatex']) await compile({code:source('basic/main.tex'),compiler},'TexCompiler compatibility baseline.');
  await compile({main_file:'main.tex',files:{'main.tex':source('multi-file/main.tex'),'body.tex':source('multi-file/body.tex')}},'Multi-file compatibility baseline.');
  await compile({main_file:'main.tex',files:{'main.tex':source('resume/main.tex'),'resume.cls':source('resume/resume.cls'),'details.sty':source('resume/details.sty')}},'Alex Example');
  await compile({code:source('cover-letter/main.tex')},'Hiring Team');
  for(const backend of ['bibtex','biber']) await compile({main_file:'main.tex',files:{'main.tex':source(`bibliography/${backend}.tex`),'references.bib':source('bibliography/references.bib')}},'Example Research');
  for(const [name,extension] of [['lmroman10-regular.otf','otf'],['Vera.ttf','ttf']]) {
    const path = execFileSync('docker',['exec',cid,'kpsewhich',name],{encoding:'utf8'}).trim();
    const font = execFileSync('docker',['exec',cid,'cat',path],{maxBuffer:4*1024*1024}).toString('base64');
    for(const compiler of ['xelatex','lualatex']) await compile({compiler,main_file:'main.tex',files:{'main.tex':`\\documentclass{article}\\usepackage{fontspec}\\setmainfont[Path=fonts/]{Regular.${extension}}\\begin{document}Uploaded font café\\end{document}`,[`fonts/Regular.${extension}`]:font}},'Uploaded font');
  }
  // Restart the immutable service and prove the same inputs work after recovery.
  compose('restart','texcompiler');
  for(let i=0;i<90;i++) {
    try {if((await fetch(base+'/ready')).ok)break;} catch{}
    if(i===89)throw new Error('service did not recover after restart');
    await sleep(1000);
  }
  await compile({code:source('basic/main.tex')},'TexCompiler compatibility baseline.');
  console.log(JSON.stringify({pythonFree:true,restrictedRuntime:true,compileChecks:count,restartRecovery:true}));
  assert.equal(readFileSync('/proc/sys/user/max_user_namespaces','utf8'),hostNamespaceLimit);
})().catch(error=>{console.error(error.message);process.exitCode=1});
