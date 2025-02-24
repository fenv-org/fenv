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
  builder: ($: $Type) => PromiseLike<T>,
  messageOnFailure: string,
): Promise<T> {
  try {
    return await builder($);
    // deno-lint-ignore no-explicit-any
  } catch (error: any) {
    const message = `${error.message}`;
    const match = /with code: (\d+)$/.exec(message);
    if (match) {
      const code = parseInt(match[1]);
      throw new CommandException(
        `${messageOnFailure}: OS status code - ${code}`,
        code,
      );
    } else {
      throw new Error(`${messageOnFailure}: ${message}`);
    }
  }
}

export class CommandException extends Error {
  constructor(message: string, public readonly code: number) {
    super(message);
    this.name = 'CommandException';
  }
}
