import external from '@fenv/external';
import { bufferToText, contextFrom, testMain } from '@fenv/test_lib';
import { assertEquals } from '@std/assert';
import { Buffer } from '@std/io';
import { afterEach, beforeEach, describe, it } from '@std/testing/bdd';
import { resolvesNext, Stub, stub } from '@std/testing/mock';
import { main } from 'cli';
import { snapshotTest } from '@cliffy/testing';
import { OperationSystem } from '@fenv/lib/os.ts';
import { FenvContext } from '@fenv/lib/context.ts';

await snapshotTest({
  name: 'init without path mode: zsh',
  meta: import.meta,
  args: ['init', '-s', 'zsh'],
  async fn() {
    const code = await testMain();
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'init without path mode: bash',
  meta: import.meta,
  args: ['init', '-s', 'bash'],
  async fn() {
    const code = await testMain();
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'init without path mode: fish',
  meta: import.meta,
  args: ['init', '-s', 'fish'],
  async fn() {
    const code = await testMain();
    assertEquals(code, 0);
  },
});

/*
describe('detectShell', () => {
  let stdout: Buffer;
  let stderr: Buffer;
  let context: FenvContext;
  let getPpidExecutablePathStub: unknown;

  function setupGetPpidExecutablePathStub(shell: string): void {
    getPpidExecutablePathStub = stub(
      external,
      'getPpidExecutablePath',
      resolvesNext([shell]),
    );
  }

  beforeEach(() => {
    stdout = new Buffer();
    stderr = new Buffer();
    context = contextFrom({ stdout, stderr, defaultShell: '/usr/bin/default' });
  });

  afterEach(() => {
    (getPpidExecutablePathStub as Stub).restore();
  });

  it('zsh', async () => {
    setupGetPpidExecutablePathStub('/usr/bin/zsh');

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 0);
    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=zsh\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('bash', async () => {
    setupGetPpidExecutablePathStub('/usr/bin/bash');

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 0);
    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=bash\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('fish', async () => {
    setupGetPpidExecutablePathStub('/opt/homebrew/bin/fish');

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 0);
    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=fish\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('default shell', async () => {
    setupGetPpidExecutablePathStub('deno');

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 0);
    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=default\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('empty shell', async () => {
    setupGetPpidExecutablePathStub('');

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 0);
    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=default\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('windows', async () => {
    setupGetPpidExecutablePathStub('');
    context.os = OperationSystem.WINDOWS;

    const code = await main({ args: ['init', '-d'], context });

    assertEquals(code, 1);
    assertEquals(bufferToText(stdout), '');
    assertEquals(
      bufferToText(stderr),
      'ERROR: Failed to detect the interactive shell\n',
    );
  });
});
*/

const initOutputZsh = `# Load fenv automatically by appending the following to
# ~/.zprofile (for login shells)
# and ~/.zshrc (for interactive shells) :

export FENV_ROOT="$HOME/.fenv"
command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
eval "$(fenv init -)"

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;

const initOutputBash = `# Load fenv automatically by appending the following to
# ~/.bash_profile if it exists, otherwise ~/.profile (for login shells)
# and ~/.bashrc (for interactive shells) :

export FENV_ROOT="$HOME/.fenv"
command -v fenv >/dev/null || export PATH="$FENV_ROOT/bin:$PATH"
eval "$(fenv init -)"

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;

const initOutputFish = `# Add fenv executable to PATH by running
# the following interactively:

set -Ux FENV_ROOT $HOME/.fenv
fish_add_path $FENV_ROOT/bin

# Load fenv automatically by appending
# the following to ~/.config/fish/conf.d/fenv.fish:

fenv init - | source

# Restart your shell for the changes to take effect:

exec $SHELL -l

`;
