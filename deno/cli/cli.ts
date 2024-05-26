import { Command, ValidationError } from '@cliffy/command';
import { VERSION } from './src/version.ts';
import * as init from './src/commands/init.ts';
import { FenvContext } from '@fenv/lib';

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
    .error((err) => {
      if (err instanceof ValidationError) {
        console.error(err.message);
      }
    })
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
