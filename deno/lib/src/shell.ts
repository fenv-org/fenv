import { $, $Type } from '@david/dax';

/**
 * Supported shells.
 */
export enum Shell {
  BASH = 'bash',
  ZSH = 'zsh',
  FISH = 'fish',
}

export async function executeCommand<T>(
  builder: ($: $Type) => Promise<T>,
  message: string,
): Promise<T> {
  try {
    return await builder($);
  } catch (error) {
    const errorMessage = `${error.message}`;
    const match = /with code: (\d+)$/.exec(errorMessage);
    if (match) {
      const code = parseInt(match[1]);
      throw new CommandException(`${message}: OS status code - ${code}`, code);
    } else {
      throw error;
    }
  }
}

export class CommandException extends Error {
  constructor(message: string, public readonly code: number) {
    super(message);
    this.name = 'CommandException';
  }
}
