const BINARY = new Set(['png','jpg','jpeg','gif','webp','pdf','zip','ttf','otf','woff','woff2']);
const INPUT = new Set([...BINARY,'tex','cls','sty','bib','bst','bbx','cbx','lbx','def','fd','clo','cfg','enc','map','tfm','vf','eps','svg','csv','dat','txt']);
const EXCLUDED = new Set(['.git','.venv','node_modules','target','build']);

export async function collectProject(root, activePath, editorSource) {
  const prefix = root.name + '/';
  const main = activePath.startsWith(prefix) ? activePath.slice(prefix.length) : activePath;
  if (!main.endsWith('.tex')) throw new Error('Select a .tex main file before compiling.');
  const files = Object.create(null);
  let total = 0;
  async function visit(directory, prefix = '', depth = 0) {
    if (depth > 32) throw new Error('Project folders are nested too deeply.');
    for await (const entry of directory.values()) {
      if (entry.kind === 'directory') {
        if (!EXCLUDED.has(entry.name) && !entry.name.startsWith('.')) await visit(entry,prefix+entry.name+'/',depth+1);
        continue;
      }
      const extension = entry.name.split('.').pop().toLowerCase();
      if (!INPUT.has(extension)) continue;
      const path = prefix + entry.name;
      const file = await entry.getFile();
      const size = path === main ? new TextEncoder().encode(editorSource).length : file.size;
      total += size;
      if (total > 20*1024*1024) throw new Error('Project exceeds 20 MiB.');
      if (Object.keys(files).length >= 256) throw new Error('Project exceeds 256 files.');
      if (path === main) files[path] = editorSource;
      else if (BINARY.has(extension)) {
        const bytes = new Uint8Array(await file.arrayBuffer());
        let binary = '';
        for (let i=0;i<bytes.length;i+=8192) binary += String.fromCharCode(...bytes.subarray(i,i+8192));
        files[path] = btoa(binary);
      } else files[path] = await file.text();
    }
  }
  await visit(root);
  if (!Object.hasOwn(files,main)) throw new Error('Active main file not found in the workspace.');
  return {files,main_file:main};
}
