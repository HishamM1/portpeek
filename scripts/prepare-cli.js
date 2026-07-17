import fs from 'fs';
import path from 'path';

const dir = path.join('src-tauri', 'target', 'release');
fs.mkdirSync(dir, { recursive: true });
fs.writeFileSync(path.join(dir, 'portpeek-cli.exe'), '');
console.log('Created dummy portpeek-cli.exe to break circular dependency');
