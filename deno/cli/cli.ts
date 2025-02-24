import { Command, ValidationError } from '@cliffy/command';
import { FenvContext } from '@fenv/lib/context.ts';
import { OperationSystem } from '@fenv/lib/os.ts';
import { CommandException } from '@fenv/lib/shell.ts';
import meta from '../meta.json' with { type: 'json' };
import * as init from './src/commands/init.ts';

export async function main({
  context,
}: {
  context: FenvContext;
}): Promise<number> {
  const command = new Command()
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
    .meta('v8', Deno.version.v8);
  try {
    const flags = await command.parse();
    if (flags.cmd.getRawArgs().length === 0) {
      command.showHelp();
      return 1;
    }
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
    console.error(`ERROR: ${error.message}`);
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
  const context: FenvContext = {
    os: detectOS(Deno.build.os),
    defaultShell: Deno.build.os !== 'windows' ? Deno.env.get('SHELL')! : '',
  };
  const statusCode = await main({ context });
  Deno.exit(statusCode);
}
