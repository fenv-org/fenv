import external from '@fenv/external';
import { FenvContext } from '@fenv/lib/context.ts';
import { OperationSystem } from '@fenv/lib/os.ts';
import {
  initOutputBash,
  initOutputFish,
  initOutputZsh,
} from '@fenv/lib/service/outputs.js';
import { executeCommand, Shell } from '@fenv/lib/shell.ts';

export function showInitInstructions(shell: Shell): void {
  switch (shell) {
    case Shell.BASH:
      console.log(initOutputBash);
      break;
    case Shell.ZSH:
      console.log(initOutputZsh);
      break;
    case Shell.FISH:
      console.log(initOutputFish);
      break;
  }
}

export async function detectShell(
  context: FenvContext,
  ppid: number,
): Promise<string | undefined> {
  if (context.os === OperationSystem.WINDOWS) {
    return;
  }

  const detectShell = await executeCommand(
    ($) => external.getPpidExecutablePath($, ppid),
    'Failed to detect shell',
  );

  return extractShellName(extractShellExecutablePath(detectShell), 1);

  function extractShellExecutablePath(command: string): string {
    const regex = /^\s*\-*(\S+)(?:\s.*)?\s*$/;
    const match = command.match(regex);
    return match?.[1] ?? context.defaultShell;
  }

  function extractShellName(
    executablePath: string,
    remainingRetry: number,
  ): string | undefined {
    const regex = /^(?:.*\/)([^/-]+)(?:-.*)?$/;
    const match = executablePath.match(regex);
    return match?.[1] ??
      (remainingRetry > 0
        ? extractShellName(context.defaultShell, remainingRetry - 1)
        : undefined);
  }
}
