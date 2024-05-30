import { toWritableStream, Writer } from '@std/io';
import { FenvContext, OperationSystem } from '@fenv/lib';

export function contextFrom(options: {
  stdout?: Writer;
  stderr?: Writer;
  os?: OperationSystem;
  defaultShell?: string;
}): FenvContext {
  const {
    stdout = Deno.stdout,
    stderr = Deno.stderr,
    os = OperationSystem.LINUX,
    defaultShell = OperationSystem.WINDOWS ? '/bin/bash' : '',
  } = options;
  return new FenvContext(
    toWritableStream(stdout, { autoClose: false }),
    toWritableStream(stderr, { autoClose: false }),
    os,
    defaultShell,
  );
}
