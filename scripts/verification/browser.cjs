const {spawn}=require('node:child_process');
const {mkdtempSync,writeFileSync}=require('node:fs');
const base=process.env.TEXCOMPILER_URL || 'http://127.0.0.1:8001';
const browser=spawn(process.env.CHROME_BIN || 'google-chrome',['--headless=new','--disable-gpu','--no-first-run','--no-default-browser-check','--remote-debugging-port=0','--user-data-dir='+mkdtempSync('/tmp/texcompiler-chrome-'),'about:blank'],{stdio:['ignore','ignore','pipe']});
let stderr='';
browser.stderr.on('data',d=>stderr+=d);
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
(async()=>{
 try {
  for(let i=0;i<450&&!/DevTools listening on (ws:\/\/[^\s]+)/.test(stderr);i++){if(browser.exitCode!==null)throw new Error(stderr.slice(-2000));await sleep(100);}
  const wsurl=stderr.match(/DevTools listening on (ws:\/\/[^\s]+)/)?.[1];if(!wsurl)throw new Error('Chrome did not start: '+stderr.slice(-2000));
  const endpoint=wsurl.replace(/^ws:/,'http:').replace(/\/devtools\/.*$/,'');
  const target=await (await fetch(endpoint+'/json/new?'+encodeURIComponent(base),{method:'PUT'})).json();
  const ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((res,rej)=>{ws.onopen=res;ws.onerror=rej});
  let sequence=0;const pending=new Map();const errors=[];
  ws.onmessage=event=>{const message=JSON.parse(event.data);if(message.id){const p=pending.get(message.id);pending.delete(message.id);message.error?p.reject(new Error(JSON.stringify(message.error))):p.resolve(message.result);}if(message.method==='Runtime.exceptionThrown')errors.push(message.params.exceptionDetails.text);};
  const send=(method,params={})=>new Promise((resolve,reject)=>{const id=++sequence;pending.set(id,{resolve,reject});ws.send(JSON.stringify({id,method,params}));});
  const evaluate=async expression=>(await send('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true})).result?.value;
  await send('Runtime.enable');await send('Page.enable');await send('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
  for(let i=0;i<150;i++){if(await evaluate('typeof ace!=="undefined" && document.querySelector("#editor .ace_content")!==null && Boolean(document.querySelector("#compiler")?.value)'))break;if(i===149)throw new Error('Editor did not initialize: '+JSON.stringify(errors));await sleep(100);}
  if(process.env.TEXCOMPILER_API_TOKEN) await evaluate(`sessionStorage.setItem('texcompiler-api-token',${JSON.stringify(process.env.TEXCOMPILER_API_TOKEN)})`);
  await evaluate('document.querySelector("#compile-btn").click()');
  let terminal='';
  for(let i=0;i<300;i++){terminal=await evaluate('document.querySelector("#terminal-content").innerText');if(terminal.includes('PDF generated successfully.'))break;if(terminal.includes('Compilation failed'))throw new Error(terminal);if(i===299)throw new Error('Compile did not finish: '+terminal);await sleep(100);}
  const preview=await evaluate('document.querySelector("#pdf-viewer").src');if(!preview.startsWith('blob:'))throw new Error('Missing PDF preview: '+preview);
  await sleep(2500);
  const screenshot=await send('Page.captureScreenshot',{format:'png'});writeFileSync('/tmp/texcompiler-rust-working.png',Buffer.from(screenshot.data,'base64'));
  console.log(JSON.stringify({editorInitialized:true,pdfPreview:true,terminal,errors,screenshot:'/tmp/texcompiler-rust-working.png'}));
  ws.close();
 }finally{browser.kill('SIGTERM');}
})().catch(error=>{console.error(error.message);process.exitCode=1});
