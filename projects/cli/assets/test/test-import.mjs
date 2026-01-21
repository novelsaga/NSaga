// 测试 import() 如何处理 CJS
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

async function test() {
  const cjsPath = resolve(__dirname, 'test-config.cjs');
  const mjsPath = resolve(__dirname, 'test-config.mjs');

  console.log('=== Testing CJS import ===');
  const cjsModule = await import(cjsPath);
  console.log('CJS module keys:', Object.keys(cjsModule));
  console.log('CJS module.default:', cjsModule.default);
  console.log('CJS module direct:', cjsModule);

  console.log('\n=== Testing MJS import ===');
  const mjsModule = await import(mjsPath);
  console.log('MJS module keys:', Object.keys(mjsModule));
  console.log('MJS module.default:', mjsModule.default);
}

test();
