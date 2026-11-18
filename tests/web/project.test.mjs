import { test } from 'node:test';
import assert from 'node:assert/strict';
import { collectProject } from '../../web/static/js/project.js';

function file(name, content, size) {
  const bytes = typeof content === 'string' ? new TextEncoder().encode(content) : content;
  return {kind:'file', name, async getFile() {return {size:size ?? bytes.length, async text(){return new TextDecoder().decode(bytes)}, async arrayBuffer(){return bytes.buffer}}}};
}
function directory(name, children) {
  return {kind:'directory', name, async *values(){yield* children}};
}
test('collects project-relative sources and binary assets with unsaved editor source', async () => {
  const root = directory('resume', [file('main.tex','saved'), directory('fonts',[file('regular.otf',new Uint8Array([0,255,17]))]), file('body.tex','body'), directory('.git',[file('secret','private')])]);
  const result = await collectProject(root,'resume/main.tex','unsaved');
  assert.equal(result.main_file,'main.tex');
  assert.deepEqual({...result.files}, {'main.tex':'unsaved','fonts/regular.otf':'AP8R','body.tex':'body'});
});
test('rejects oversized projects before reading bytes', async () => {
  await assert.rejects(collectProject(directory('r',[file('main.tex','x'),file('image.png','x',21*1024*1024)]),'r/main.tex','x'),/20 MiB/);
});
test('rejects non-TeX active files and missing active files', async () => {
  const root = directory('r',[file('main.tex','source')]);
  await assert.rejects(collectProject(root,'r/image.png','source'),/\.tex/);
  await assert.rejects(collectProject(root,'r/absent.tex','source'),/not found/);
});
