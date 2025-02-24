import { snapshotTest } from '@cliffy/testing';
import external from '@fenv/external';
import { OperationSystem } from '@fenv/lib/os.ts';
import { testMain } from '@fenv/test_lib';
import { assertEquals } from '@std/assert';
import { resolvesNext, stub } from '@std/testing/mock';

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

await snapshotTest({
  name: 'detectShell: zsh',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['/usr/bin/zsh']),
    );
    const code = await testMain({
      defaultShell: '/usr/bin/default',
    });
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'detectShell: bash',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['/usr/bin/bash']),
    );
    const code = await testMain({
      defaultShell: '/usr/bin/default',
    });
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'detectShell: fish',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['/opt/homebrew/bin/fish']),
    );
    const code = await testMain({
      defaultShell: '/usr/bin/default',
    });
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'detectShell: default shell',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['deno']),
    );
    const code = await testMain({
      defaultShell: '/usr/bin/default',
    });
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'detectShell: empty shell',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['']),
    );
    const code = await testMain({
      defaultShell: '/usr/bin/default',
    });
    assertEquals(code, 0);
  },
});

await snapshotTest({
  name: 'detectShell: unsupported os',
  meta: import.meta,
  args: ['init', '-d'],
  async fn() {
    stub(
      external,
      'getPpidExecutablePath',
      resolvesNext(['']),
    );
    const code = await testMain({
      os: OperationSystem.WINDOWS,
    });
    assertEquals(code, 1);
  },
});
