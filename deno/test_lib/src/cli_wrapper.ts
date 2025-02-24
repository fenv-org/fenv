import { FenvContext } from '@fenv/lib/context.ts';
import { OperationSystem } from '@fenv/lib/os.ts';
import * as cli from 'cli';

export function testMain(
  context?: Partial<FenvContext>,
): ReturnType<typeof cli.main> {
  const {
    os = OperationSystem.LINUX,
    defaultShell = 'zsh',
  } = context ?? {};
  return cli.main({ context: { os, defaultShell } });
}
