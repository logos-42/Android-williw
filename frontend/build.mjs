// Williw 前端构建脚本（TypeScript → ES bundle）
// 产出：dist/app.js（minified）+ 类型检查通过
// 用法：node build.mjs          （单次构建）
//       node build.mjs --watch  （监听 src/，改动即重建）

import * as esbuild from 'esbuild';
import { readFileSync, writeFileSync, statSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SRC = join(__dirname, 'src');
const OUT_DIR = join(__dirname, 'dist');
const OUT_FILE = join(OUT_DIR, 'app.js');

const isWatch = process.argv.includes('--watch');

function ensureDist() {
  if (!existsSync(OUT_DIR)) mkdirSync(OUT_DIR, { recursive: true });
  // 把 src/index.html 拷到 dist/（如果存在），否则保留 dist/ 里手写的 index.html
  const srcHtml = join(SRC, 'index.html');
  if (existsSync(srcHtml)) {
    writeFileSync(join(OUT_DIR, 'index.html'), readFileSync(srcHtml, 'utf8'));
    console.log('[build] copied src/index.html → dist/index.html');
  }
}

function runTypecheck() {
  console.log('[build] typecheck: tsc --noEmit ...');
  try {
    execSync('npx tsc --noEmit', { stdio: 'inherit', cwd: __dirname });
    console.log('[build] typecheck: OK');
  } catch (e) {
    console.error('[build] typecheck FAILED — refusing to emit bundle');
    process.exit(1);
  }
}

function buildOnce() {
  runTypecheck();
  ensureDist();
  const t0 = Date.now();
  esbuild.buildSync({
    entryPoints: [join(SRC, 'main.ts')],
    bundle: true,
    format: 'iife',
    target: 'es2020',
    outfile: OUT_FILE,
    minify: true,
    sourcemap: false,
    logLevel: 'info',
  });
  const size = statSync(OUT_FILE).size;
  console.log(`[build] ✓ dist/app.js  (${(size / 1024).toFixed(1)} KB)  in ${Date.now() - t0}ms`);
}

function buildWatch() {
  ensureDist();
  const ctx = esbuild.context({
    entryPoints: [join(SRC, 'main.ts')],
    bundle: true,
    format: 'iife',
    target: 'es2020',
    outfile: OUT_FILE,
    minify: true,
    sourcemap: false,
    logLevel: 'info',
  });
  ctx.watch();
  console.log('[build] watching src/**/*.ts  (Ctrl+C to stop)');
  // 类型检查在 build 前已通过；watch 模式不阻塞增量
}

buildOnce();
if (isWatch) buildWatch();
