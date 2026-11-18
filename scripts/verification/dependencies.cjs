const {readFileSync}=require('node:fs');
const metadata=JSON.parse(readFileSync(process.argv[2],'utf8'));
// Exact reviewed SPDX expressions. A new expression requires explicit review,
// rather than treating an unknown license as permissive by substring matching.
const reviewed=new Set([
 '0BSD OR MIT OR Apache-2.0','MIT OR Apache-2.0','Unlicense OR MIT','BSD-3-Clause',
 'Apache-2.0 OR MIT','MIT','BSD-3-Clause/MIT','CC0-1.0 OR MIT-0 OR Apache-2.0',
 'MIT/Apache-2.0','(Apache-2.0 OR MIT) AND BSD-3-Clause',
 'Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT','MIT AND BSD-3-Clause',
 'MIT OR Zlib OR Apache-2.0','MIT OR Apache-2.0 OR LGPL-2.1-or-later',
 'Apache-2.0 OR BSL-1.0','Apache-2.0','Zlib OR Apache-2.0 OR MIT',
 '(MIT OR Apache-2.0) AND Unicode-3.0','Zlib',
]);
const dependencies=metadata.packages.filter(pkg=>!metadata.workspace_members.includes(pkg.id));
for(const pkg of dependencies)if(!reviewed.has(pkg.license))throw new Error(`License review required: ${pkg.name} ${pkg.version}: ${pkg.license}`);
console.log(JSON.stringify({policy:'Select permissive alternatives where offered; retain required MIT/BSD/Apache/Unicode notices and review every new expression. This does not assign a license to the application.',dependencies:dependencies.map(({name,version,license})=>({name,version,license})).sort((a,b)=>a.name.localeCompare(b.name))},null,2));
