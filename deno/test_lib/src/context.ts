import { FenvContext } from '@fenv/lib/context.ts';
import { OperationSystem } from '@fenv/lib/os.ts';
import { Writer } from '@std/io';

export function contextFrom(options: {
  stdout?: Writer;
  stderr?: Writer;
  os?: OperationSystem;
  defaultShell?: string;
}): FenvContext {
  const {
    os = OperationSystem.LINUX,
    defaultShell = '/bin/sh',
  } = options;
  return {
    // toWritableStream(stdout, { autoClose: false }),
    // toWritableStream(stderr, { autoClose: false }),
    os,
    defaultShell,
  };
}
