import { FenvContext } from '@fenv/lib';
import { bufferToText, contextFrom } from '@fenv/test_lib';
import { assertEquals } from '@std/assert';
import { Buffer } from '@std/io';
import { afterEach, beforeEach, describe, it } from '@std/testing/bdd';
import { resolvesNext, Stub, stub } from '@std/testing/mock';
import { main } from 'cli';
import { _internals } from '../../../lib/service/src/init_service.ts';

describe('init without path mode', () => {
  let stdout: Buffer;
  let stderr: Buffer;
  let context: FenvContext;

  beforeEach(() => {
    stdout = new Buffer();
    stderr = new Buffer();
    context = contextFrom({ stdout, stderr });
  });

  it('zsh', async () => {
    await main({
      args: ['init', '-s', 'zsh'],
      context,
    });

    assertEquals(bufferToText(stdout), initOutputZsh);
    assertEquals(bufferToText(stderr), '');
  });

  it('bash', async () => {
    await main({
      args: ['init', '-s', 'bash'],
      context,
    });

    assertEquals(bufferToText(stdout), initOutputBash);
    assertEquals(bufferToText(stderr), '');
  });

  it('fish', async () => {
    await main({
      args: ['init', '-s', 'fish'],
      context,
    });

    assertEquals(bufferToText(stdout), initOutputFish);
    assertEquals(bufferToText(stderr), '');
  });
});

describe('detectShell', () => {
  let stdout: Buffer;
  let stderr: Buffer;
  let context: FenvContext;
  let getPpidExecutablePathStub: unknown;

  function setupGetPpidExecutablePathStub(shell: string): void {
    getPpidExecutablePathStub = stub(
      _internals,
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

    await main({
      args: ['init', '-d'],
      context,
    });

    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=zsh\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('bash', async () => {
    setupGetPpidExecutablePathStub('/usr/bin/bash');

    await main({
      args: ['init', '-d'],
      context,
    });

    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=bash\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('fish', async () => {
    setupGetPpidExecutablePathStub('/opt/homebrew/bin/fish');

    await main({
      args: ['init', '-d'],
      context,
    });

    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=fish\n');
    assertEquals(bufferToText(stderr), '');
  });

  it('default shell', async () => {
    setupGetPpidExecutablePathStub('deno');

    await main({
      args: ['init', '-d'],
      context,
    });

    assertEquals(bufferToText(stdout), 'FENV_SHELL_DETECT=default\n');
    assertEquals(bufferToText(stderr), '');
  });
});

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
