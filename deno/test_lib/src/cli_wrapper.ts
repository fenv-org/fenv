import { FenvContext } from '@fenv/lib/context.ts';
import * as cli from 'cli';

export function testMain(
  context?: Partial<FenvContext>,
): ReturnType<typeof cli.main> {
  const {
    os = cli.detectOS(Deno.build.os),
    defaultShell = 'zsh',
  } = context ?? {};
  return cli.main({ context: { os, defaultShell } });
}
