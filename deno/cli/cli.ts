import { Command, ValidationError } from '@cliffy/command';
import { FenvContext } from '@fenv/lib/context.ts';
import { OperationSystem } from '@fenv/lib/os.ts';
import { CommandException } from '@fenv/lib/shell.ts';
import meta from '../meta.json' with { type: 'json' };
import * as init from './commands/init.ts';
import { join, resolve } from '@std/path';

type GlobalEnv = {
  fenvRoot?: string;
};

export async function main({
  context,
}: {
  context: {
    os: OperationSystem;
    defaultShell: string;
  };
}): Promise<number> {
  const command = new Command()
    .name('fenv')
    .version(meta.version)
    .description('Simple flutter sdk version management')
    .meta('deno', Deno.version.deno)
    .meta('v8', Deno.version.v8)
    .globalEnv(
      'FENV_ROOT=<path:string>',
      'The root directory of the fenv installation. e.g. $HOME/.fenv',
    )
    .command('init', init.buildSubCommand(buildFenvContext))
    .meta('deno', Deno.version.deno)
    .meta('v8', Deno.version.v8)
    .error(reportError);

  if (Deno.args.length === 0) {
    command.showHelp();
    return 1;
  }

  try {
    await command.parse();
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

  function buildFenvContext(options: Record<string, unknown>): FenvContext {
    let fenvRoot: string;
    if ('fenvRoot' in options && typeof options.fenvRoot === 'string') {
      fenvRoot = options.fenvRoot;
    } else {
      fenvRoot = resolve(Deno.env.get('HOME')!, '.fenv');
    }
    return { ...context, fenvRoot };
  }
}

export function detectOS(osName: string): OperationSystem {
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
  const context = {
    os: detectOS(Deno.build.os),
    defaultShell: Deno.build.os !== 'windows' ? Deno.env.get('SHELL')! : '',
  };
  const statusCode = await main({ context });
  Deno.exit(statusCode);
}
