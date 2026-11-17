#!/usr/bin/env node
'use strict';

/**
 * poof launcher for npm/npx.
 *
 * Resolves the native executable shipped by the matching optional platform
 * package and spawns it. This file performs no network access, no downloads,
 * no extraction, and no shell interpolation: the native binary is already
 * installed by npm alongside this package.
 */

const { spawn } = require('node:child_process');

/** The only supported (platform, arch) pairs. No Windows, no other Linux CPUs. */
const SUPPORTED_TARGETS = Object.freeze([
  Object.freeze({ platform: 'linux', arch: 'x64', package: '@pirafrank/poof-linux-x64' }),
  Object.freeze({ platform: 'linux', arch: 'arm64', package: '@pirafrank/poof-linux-arm64' }),
  Object.freeze({ platform: 'darwin', arch: 'x64', package: '@pirafrank/poof-darwin-x64' }),
  Object.freeze({ platform: 'darwin', arch: 'arm64', package: '@pirafrank/poof-darwin-arm64' }),
]);

/** Signals forwarded to the child so Ctrl-C and termination behave natively. */
const FORWARDED_SIGNALS = Object.freeze(['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGQUIT']);

const INSTALL_HINT = 'npm install --global @pirafrank/poof';

/**
 * Select the platform package for a (platform, arch) pair.
 * @param {string} platform process.platform value.
 * @param {string} arch process.arch value.
 * @returns {{platform: string, arch: string, package: string}|null} target or null.
 */
function selectTarget(platform, arch) {
  return SUPPORTED_TARGETS.find((t) => t.platform === platform && t.arch === arch) || null;
}

/**
 * Render the supported target list for diagnostics.
 * @returns {string} multi-line list of supported platforms.
 */
function supportedTargetsText() {
  return SUPPORTED_TARGETS.map((t) => `  - ${t.platform}/${t.arch}  (${t.package})`).join('\n');
}

/**
 * Build the actionable error for an unsupported operating system or CPU.
 * @param {string} platform detected process.platform.
 * @param {string} arch detected process.arch.
 * @returns {string} diagnostic text.
 */
function unsupportedMessage(platform, arch) {
  return [
    'poof: unsupported platform.',
    `Detected: platform=${platform} arch=${arch}`,
    'Supported targets:',
    supportedTargetsText(),
    '',
    'poof publishes native binaries for Linux (x64, arm64) and macOS (x64, arm64) only.',
    'Windows is not supported. On other systems, build from source or use a native',
    `release archive: https://github.com/pirafrank/poof/releases`,
    '',
  ].join('\n');
}

/**
 * Build the actionable error for a missing optional native package.
 * @param {{platform: string, arch: string, package: string}} target expected package.
 * @param {Error} error resolution failure.
 * @returns {string} diagnostic text.
 */
function missingPackageMessage(target, error) {
  return [
    `poof: native package ${target.package} is not installed.`,
    `Detected: platform=${target.platform} arch=${target.arch}`,
    'Supported targets:',
    supportedTargetsText(),
    '',
    'Likely causes:',
    '  - optional dependencies were skipped (npm --omit=optional, --no-optional,',
    '    or a package manager configured to ignore optionalDependencies)',
    '  - an incomplete or interrupted install, or a pruned node_modules tree',
    '  - a package-manager layout that does not place sibling scoped packages',
    '    where Node can resolve them (for example some pnpm/Yarn PnP setups)',
    '',
    'Fix: reinstall with optional dependencies enabled, for example',
    `  ${INSTALL_HINT}`,
    'or, for a local project:',
    '  npm install --include=optional @pirafrank/poof',
    '',
    `Resolution error: ${error && error.message ? error.message : String(error)}`,
    '',
  ].join('\n');
}

/**
 * Build the actionable error when the resolved binary cannot be executed.
 * @param {string} binaryPath resolved native executable path.
 * @param {Error} error spawn failure.
 * @returns {string} diagnostic text.
 */
function spawnFailureMessage(binaryPath, error) {
  const code = error && error.code ? error.code : 'unknown';
  return [
    `poof: failed to execute ${binaryPath} (${code}).`,
    'The native package is installed but its executable could not be started.',
    'Check that the file exists, is a regular file, and has the executable bit set',
    '(expected mode 0755), and that your antivirus or mount options (noexec) are',
    'not blocking it. Reinstalling usually repairs a damaged installation:',
    `  ${INSTALL_HINT}`,
    '',
  ].join('\n');
}

/**
 * Resolve the packaged native executable relative to this launcher's package.
 * @param {{package: string}} target platform package descriptor.
 * @param {string} fromDir directory to start Node resolution from.
 * @returns {string} absolute path to the native executable.
 */
function resolveBinaryPath(target, fromDir) {
  return require.resolve(`${target.package}/bin/poof`, { paths: [fromDir] });
}

/**
 * Spawn the native binary, forwarding arguments, stdio, and signals.
 * @param {string} binaryPath absolute path to the native executable.
 * @param {string[]} args arguments to forward unchanged.
 * @returns {Promise<{code: number|null, signal: string|null}>} child outcome.
 */
function runBinary(binaryPath, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(binaryPath, args, { stdio: 'inherit', shell: false });
    const forward = (signal) => {
      try {
        child.kill(signal);
      } catch {
        // The child may already have exited; nothing to forward.
      }
    };
    for (const signal of FORWARDED_SIGNALS) {
      process.on(signal, forward);
    }
    const cleanup = () => {
      for (const signal of FORWARDED_SIGNALS) {
        process.removeListener(signal, forward);
      }
    };
    child.once('error', (error) => {
      cleanup();
      reject(error);
    });
    child.once('close', (code, signal) => {
      cleanup();
      resolve({ code, signal });
    });
  });
}

/**
 * Launcher entry point.
 * @param {string[]} [argv] full argv (defaults to process.argv).
 * @param {string} [platform] detected platform (defaults to process.platform).
 * @param {string} [arch] detected architecture (defaults to process.arch).
 * @param {string} [fromDir] resolution root (defaults to this file's directory).
 * @returns {Promise<number>} process exit code.
 */
async function main(argv, platform, arch, fromDir) {
  const fullArgv = argv || process.argv;
  const detectedPlatform = platform || process.platform;
  const detectedArch = arch || process.arch;
  const resolutionRoot = fromDir || __dirname;

  const target = selectTarget(detectedPlatform, detectedArch);
  if (!target) {
    process.stderr.write(unsupportedMessage(detectedPlatform, detectedArch));
    return 1;
  }

  let binaryPath;
  try {
    binaryPath = resolveBinaryPath(target, resolutionRoot);
  } catch (error) {
    process.stderr.write(missingPackageMessage(target, error));
    return 1;
  }

  let outcome;
  try {
    outcome = await runBinary(binaryPath, fullArgv.slice(2));
  } catch (error) {
    process.stderr.write(spawnFailureMessage(binaryPath, error));
    return 1;
  }

  if (outcome.signal) {
    // Die by the same signal so shells and supervisors observe native behavior.
    process.kill(process.pid, outcome.signal);
    return 1;
  }
  return outcome.code === null ? 1 : outcome.code;
}

if (require.main === module) {
  main()
    .then((code) => {
      process.exitCode = code;
    })
    .catch((error) => {
      process.stderr.write(`poof: unexpected launcher failure: ${error && error.stack ? error.stack : error}\n`);
      process.exitCode = 1;
    });
}

module.exports = {
  SUPPORTED_TARGETS,
  FORWARDED_SIGNALS,
  selectTarget,
  supportedTargetsText,
  unsupportedMessage,
  missingPackageMessage,
  spawnFailureMessage,
  resolveBinaryPath,
  runBinary,
  main,
};
