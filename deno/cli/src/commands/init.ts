import {
  ArgumentValue,
  Command,
  EnumType,
  ValidationError,
} from '@cliffy/command';
import { FenvContext, io, Shell } from '@fenv/lib';
import { init } from '@fenv/lib/service';

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
  options: {
    shell?: Shell;
    detectShell?: boolean;
  },
  pathMode?: string,
): Promise<void> {
  if (options.detectShell) {
    await io.writeTextLine(context.stdout, 'Detected shell:', Shell.BASH);
    return;
  }

  if (pathMode === '-') {
    await io.writeTextLine(context.stdout, 'Adding fenv to PATH');
    return;
  }

  const shell = options.shell ?? Shell.BASH;
  await init.showInitInstructions(context, shell);
}
