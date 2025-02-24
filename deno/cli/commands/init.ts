import {
  ArgumentValue,
  Command,
  EnumType,
  ValidationError,
} from '@cliffy/command';
import { type FenvContext } from '@fenv/lib/context.ts';
import {
  detectShell,
  showInitInstructions,
} from '@fenv/lib/service/init_service.ts';
import { Shell } from '@fenv/lib/shell.ts';
import { ArgumentsOf, type OptionsOf } from './types.ts';

function pathModeType({ value }: ArgumentValue): string {
  if (value !== '-') {
    throw new ValidationError(
      `Invalid argument value "${value}". Must be '[-]'`,
    );
  }
  return value;
}

export const command = new Command()
  .description('Help registering `fenv` to your `PATH` env. variable')
  .type('pathMode', pathModeType)
  .type('shell', new EnumType(Shell))
  .arguments('[-:pathMode]')
  .option('-d, --detect-shell', 'Detects the current running shell.')
  .option('-s, --shell <shell:shell>', 'Specify the shell to use.');

export async function handler(
  context: FenvContext,
  options: OptionsOf<typeof command>,
  pathMode: ArgumentsOf<typeof command>['0'],
): Promise<void> {
  if (options.detectShell) {
    const shell = await detectShell(context, Deno.ppid);
    if (!shell) {
      throw new Error('Failed to detect the interactive shell');
    }
    console.log(`FENV_SHELL_DETECT=${shell}`);
    return;
  }

  if (pathMode === '-') {
    console.log('Adding fenv to PATH');
    return;
  }

  const shell = options.shell ?? Shell.BASH;
  showInitInstructions(shell);
}
