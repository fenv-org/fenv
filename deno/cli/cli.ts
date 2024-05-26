import { Command } from '@cliffy/command';
import { FenvContext } from '@fenv/lib';
import * as init from './src/commands/init.ts';
import { VERSION } from './src/version.ts';

export async function main(
  { args, context }: {
    args: string[];
    context: FenvContext;
  },
): Promise<void> {
  await new Command()
    .name('fenv')
    .version(`v${VERSION}`)
    .description('Simple flutter sdk version management')
    .command(
      'init',
      init.command.action((options, args) =>
        init.handler(context, options, args)
      ),
    )
    .parse(args);
}

if (import.meta.main) {
  const context = new FenvContext(
    Deno.stdout.writable,
    Deno.stderr.writable,
  );
  await main({
    args: Deno.args,
    context,
  });
}
