'use strict';

/**
 * Launcher tests: target selection, diagnostics, resolution layouts, and real
 * subprocess fidelity for arguments, exit codes, signals, and stdio.
 *
 * Run with: node --test npm/tests/
 */

const { test, describe } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

const launcher = require('../bin/poof.js');

const LAUNCHER_SOURCE = path.join(__dirname, '..', 'bin', 'poof.js');
const HOST_TARGET = launcher.selectTarget(process.platform, process.arch);

const FAKE_BINARY = `#!/usr/bin/env node
'use strict';
const args = process.argv.slice(2);
const mode = args[0];
if (mode === '--exit') {
  process.exit(Number(args[1]));
} else if (mode === '--echo') {
  process.stdout.write(JSON.stringify(args));
} else if (mode === '--stderr') {
  process.stderr.write('stderr-payload');
  process.exit(3);
} else if (mode === '--self-signal') {
  process.kill(process.pid, args[1]);
} else {
  process.stdout.write('fake-poof:' + args.join('|'));
}
`;

/** Create a temp dir removed automatically when the test finishes. */
function tempDir(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'poof-launcher-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

/**
 * Build an installed-package layout: launcher plus (optionally) native package.
 * @returns {{root: string, launcherPath: string, binaryPath: string|null}}
 */
function makeLayout(t, { withNative = true, binaryMode = 0o755, binaryContent = FAKE_BINARY } = {}) {
  const root = tempDir(t);
  const baseBin = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin');
  fs.mkdirSync(baseBin, { recursive: true });
  const launcherPath = path.join(baseBin, 'poof.js');
  fs.copyFileSync(LAUNCHER_SOURCE, launcherPath);
  fs.chmodSync(launcherPath, 0o755);
  let binaryPath = null;
  if (withNative) {
    const nativeBin = path.join(root, 'node_modules', HOST_TARGET.package, 'bin');
    fs.mkdirSync(nativeBin, { recursive: true });
    binaryPath = path.join(nativeBin, 'poof');
    fs.writeFileSync(binaryPath, binaryContent);
    fs.chmodSync(binaryPath, binaryMode);
  }
  return { root, launcherPath, binaryPath };
}

/** Run the installed launcher as a real subprocess. */
function runLauncher(launcherPath, args) {
  return spawnSync(process.execPath, [launcherPath, ...args], {
    encoding: 'utf8',
    timeout: 30000,
  });
}

/** Capture process.stderr.write across awaits, restoring it afterwards. */
async function captureStderr(fn) {
  const original = process.stderr.write;
  let text = '';
  process.stderr.write = (chunk) => {
    text += String(chunk);
    return true;
  };
  try {
    const result = await fn();
    return { result, text };
  } finally {
    process.stderr.write = original;
  }
}

describe('target selection', () => {
  test('maps the four supported platform/arch pairs', () => {
    const expected = [
      ['linux', 'x64', '@pirafrank/poof-linux-x64'],
      ['linux', 'arm64', '@pirafrank/poof-linux-arm64'],
      ['darwin', 'x64', '@pirafrank/poof-darwin-x64'],
      ['darwin', 'arm64', '@pirafrank/poof-darwin-arm64'],
    ];
    for (const [platform, arch, pkg] of expected) {
      const target = launcher.selectTarget(platform, arch);
      assert.ok(target, `${platform}/${arch} must be supported`);
      assert.equal(target.package, pkg);
    }
  });

  test('rejects unsupported platforms and architectures', () => {
    const unsupported = [
      ['win32', 'x64'],
      ['win32', 'arm64'],
      ['linux', 'ia32'],
      ['linux', 'arm'],
      ['linux', 's390x'],
      ['linux', 'ppc64'],
      ['darwin', 'ppc64'],
      ['darwin', 'ia32'],
      ['freebsd', 'x64'],
      ['aix', 'ppc64'],
      ['sunos', 'x64'],
      ['linux', 'x64-musl'],
      ['', ''],
    ];
    for (const [platform, arch] of unsupported) {
      assert.equal(launcher.selectTarget(platform, arch), null, `${platform}/${arch}`);
    }
  });

  test('exposes exactly four frozen targets and no Windows entry', () => {
    assert.equal(launcher.SUPPORTED_TARGETS.length, 4);
    assert.ok(Object.isFrozen(launcher.SUPPORTED_TARGETS));
    for (const target of launcher.SUPPORTED_TARGETS) {
      assert.ok(Object.isFrozen(target));
      assert.notEqual(target.platform, 'win32');
    }
  });
});

describe('diagnostics', () => {
  test('unsupported message names the host and every supported target', () => {
    const text = launcher.unsupportedMessage('win32', 'x64');
    assert.match(text, /unsupported platform/);
    assert.match(text, /platform=win32 arch=x64/);
    assert.match(text, /Windows is not supported/);
    for (const target of launcher.SUPPORTED_TARGETS) {
      assert.ok(text.includes(target.package), `must list ${target.package}`);
    }
    assert.match(text, /https:\/\/github\.com\/pirafrank\/poof\/releases/);
  });

  test('missing package message is actionable and names the cause', () => {
    const target = launcher.SUPPORTED_TARGETS[0];
    const text = launcher.missingPackageMessage(target, new Error('Cannot find module'));
    assert.ok(text.includes(target.package));
    assert.match(text, /is not installed/);
    assert.match(text, /--omit=optional/);
    assert.match(text, /--no-optional/);
    assert.match(text, /--include=optional/);
    assert.match(text, /npm install --global @pirafrank\/poof/);
    assert.match(text, /pnpm\/Yarn PnP/);
    assert.match(text, /Cannot find module/);
  });

  test('missing package message tolerates a non-Error value', () => {
    const text = launcher.missingPackageMessage(launcher.SUPPORTED_TARGETS[0], 'plain failure');
    assert.match(text, /plain failure/);
  });

  test('spawn failure message explains permissions and reinstall', () => {
    const text = launcher.spawnFailureMessage('/tmp/poof', { code: 'EACCES' });
    assert.match(text, /failed to execute \/tmp\/poof \(EACCES\)/);
    assert.match(text, /0755/);
    assert.match(text, /noexec/);
    assert.match(text, /npm install --global @pirafrank\/poof/);
  });

  test('spawn failure message handles a missing code', () => {
    assert.match(launcher.spawnFailureMessage('/tmp/poof', {}), /\(unknown\)/);
  });

  test('supported targets text lists all four combinations', () => {
    const text = launcher.supportedTargetsText();
    assert.equal(text.split('\n').length, 4);
    assert.match(text, /linux\/x64/);
    assert.match(text, /darwin\/arm64/);
  });
});

describe('resolution', () => {
  test('resolves a sibling scoped platform package', { skip: !HOST_TARGET }, (t) => {
    const { root, binaryPath } = makeLayout(t);
    const fromDir = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin');
    assert.equal(launcher.resolveBinaryPath(HOST_TARGET, fromDir), binaryPath);
  });

  test('resolves from a nested consumer directory', { skip: !HOST_TARGET }, (t) => {
    const { root, binaryPath } = makeLayout(t);
    const nested = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin', 'deep');
    fs.mkdirSync(nested, { recursive: true });
    assert.equal(launcher.resolveBinaryPath(HOST_TARGET, nested), binaryPath);
  });

  test('throws when the platform package is absent', { skip: !HOST_TARGET }, (t) => {
    const { root } = makeLayout(t, { withNative: false });
    const fromDir = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin');
    assert.throws(() => launcher.resolveBinaryPath(HOST_TARGET, fromDir), /Cannot find module/);
  });
});

describe('main() error paths', () => {
  test('unsupported platform returns 1 and writes a diagnostic', async () => {
    const { result, text } = await captureStderr(() =>
      launcher.main(['node', 'poof.js', '--version'], 'win32', 'x64')
    );
    assert.equal(result, 1);
    assert.match(text, /unsupported platform/);
    assert.match(text, /platform=win32 arch=x64/);
  });

  test('missing native package returns 1 and writes a diagnostic', { skip: !HOST_TARGET }, async (t) => {
    const { root } = makeLayout(t, { withNative: false });
    const fromDir = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin');
    const { result, text } = await captureStderr(() =>
      launcher.main(['node', 'poof.js'], process.platform, process.arch, fromDir)
    );
    assert.equal(result, 1);
    assert.match(text, /is not installed/);
    assert.match(text, /--include=optional/);
  });

  test('non-executable binary returns 1 with a spawn diagnostic', { skip: !HOST_TARGET }, async (t) => {
    const { root } = makeLayout(t, { binaryMode: 0o644 });
    const fromDir = path.join(root, 'node_modules', '@pirafrank', 'poof', 'bin');
    const { result, text } = await captureStderr(() =>
      launcher.main(['node', 'poof.js'], process.platform, process.arch, fromDir)
    );
    assert.equal(result, 1);
    assert.match(text, /failed to execute/);
    assert.match(text, /0755/);
  });
});

describe('subprocess fidelity', () => {
  test('forwards arguments byte-for-byte, including shell metacharacters', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    const args = ['--echo', 'a b', ';echo INJECTED', '$(whoami)', '--flag=x', '', 'üñï'];
    const result = runLauncher(launcherPath, args);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), args);
    assert.ok(!result.stdout.includes('INJECTED\n'));
  });

  test('propagates exit code 0', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    assert.equal(runLauncher(launcherPath, ['--exit', '0']).status, 0);
  });

  test('propagates exit code 1', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    assert.equal(runLauncher(launcherPath, ['--exit', '1']).status, 1);
  });

  test('propagates a non-standard exit code 42', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    assert.equal(runLauncher(launcherPath, ['--exit', '42']).status, 42);
  });

  test('inherits stdout and stderr and keeps the child exit code', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    const result = runLauncher(launcherPath, ['--stderr']);
    assert.equal(result.status, 3);
    assert.equal(result.stderr, 'stderr-payload');
  });

  test('dies by the same signal as the child', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    const result = runLauncher(launcherPath, ['--self-signal', 'SIGTERM']);
    assert.equal(result.signal, 'SIGTERM');
    assert.equal(result.status, null);
  });

  test('missing native package exits 1 as a real subprocess', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t, { withNative: false });
    const result = runLauncher(launcherPath, ['--version']);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /is not installed/);
    assert.equal(result.stdout, '');
  });

  test('no arguments still runs the binary', { skip: !HOST_TARGET }, (t) => {
    const { launcherPath } = makeLayout(t);
    const result = runLauncher(launcherPath, []);
    assert.equal(result.status, 0);
    assert.equal(result.stdout, 'fake-poof:');
  });
});

describe('no network, no shell, no downloads', () => {
  const source = fs.readFileSync(LAUNCHER_SOURCE, 'utf8');

  test('requires only child_process spawn', () => {
    const requires = [...source.matchAll(/require\((['"])([^'"]+)\1\)/g)].map((m) => m[2]);
    assert.deepEqual(requires, ['node:child_process']);
    assert.match(source, /const \{ spawn \} = require\('node:child_process'\)/);
  });

  test('contains no network primitives', () => {
    for (const pattern of [
      /require\(['"](node:)?(http|https|net|dns|dgram|tls)['"]\)/,
      /\bfetch\s*\(/,
      /\bXMLHttpRequest\b/,
      /\baxios\b/,
      /\bcurl\b/,
      /\bwget\b/,
    ]) {
      assert.ok(!pattern.test(source), `launcher must not contain ${pattern}`);
    }
  });

  test('never uses a shell or exec', () => {
    assert.match(source, /shell:\s*false/);
    for (const pattern of [/shell:\s*true/, /\bexecSync\b/, /\bexecFileSync\b/, /\bexec\s*\(/]) {
      assert.ok(!pattern.test(source), `launcher must not contain ${pattern}`);
    }
  });

  test('performs no extraction and never spawns an installer', () => {
    for (const pattern of [/\btar\b/, /\bunzip\b/, /\bextract\s*\(/, /spawn\(\s*['"](npm|npx)['"]/]) {
      assert.ok(!pattern.test(source), `launcher must not contain ${pattern}`);
    }
    // The remediation text may mention npm install; only spawning it is forbidden.
    assert.match(source, /npm install --global @pirafrank\/poof/);
  });

  test('forwards the documented signal set', () => {
    assert.deepEqual([...launcher.FORWARDED_SIGNALS], ['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGQUIT']);
  });
});
