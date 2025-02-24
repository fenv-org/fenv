import { Command, ValidationError } from '@cliffy/command';
import { CommandException, FenvContext, OperationSystem } from '@fenv/lib';
import { writeTextLine } from '../lib/src/io/io.ts';
import * as init from './src/commands/init.ts';
import meta from '../meta.json' with { type: 'json' };

export async function main(
  { args, context }: {
    args: string[];
    context: FenvContext;
  },
): Promise<number> {
  try {
    await new Command()
      .name('fenv')
      .version(meta.version)
      .description('Simple flutter sdk version management')
      .globalEnv(
        'FENV_ROOT=<path:string>',
        'The root directory of the fenv installation. e.g. $HOME/.fenv',
      )
      .command(
        'init',
        init.command.action((options, args) =>
          init.handler(context, options, args)
        ),
      )
      .error(reportError)
      .meta('deno', Deno.version.deno)
      .meta('v8', Deno.version.v8)
      .parse(args);
    return 0;
  } catch (error) {
    if (error instanceof CommandException) {
      return error.code;
    } else {
      return 1;
    }
  }

  function reportError(error: Error): void {
    if (error instanceof ValidationError) {
      return;
    }
    writeTextLine(context.stderr, `ERROR: ${error.message}`);
  }
}

function detectOS(osName: string): OperationSystem {
  switch (osName) {
    case 'windows':
      return OperationSystem.WINDOWS;
    case 'darwin':
      return OperationSystem.MACOS;
    default:
      return OperationSystem.LINUX;
  }
}

if (import.meta.main) {
  const context = new FenvContext(
    Deno.stdout.writable,
    Deno.stderr.writable,
    detectOS(Deno.build.os),
    Deno.build.os !== 'windows' ? Deno.env.get('SHELL')! : '',
  );
  const statusCode = await main({
    args: Deno.args,
    context,
  });
  Deno.exit(statusCode);
}
