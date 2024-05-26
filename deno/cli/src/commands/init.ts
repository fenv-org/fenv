import {
  ArgumentValue,
  Command,
  EnumType,
  ValidationError,
} from '@cliffy/command';
import { Shell } from '@fenv/lib';

function pathModeType({ value }: ArgumentValue): string {
  if (value !== '-') {
    throw new ValidationError(
      `Invalid argument value "${value}". Must be '[-]'`,
    );
  }
  return value;
}

export const initCommand = new Command()
  .description('Help registering `fenv` to your `PATH` env. variable')
  .type('pathMode', pathModeType)
  .type('shell', new EnumType(Shell))
  .arguments('[-:pathMode]')
  .option('-d, --detect-shell', 'Detects the current running shell.')
  .option('-s, --shell <shell:shell>', 'Specify the shell to use.');
