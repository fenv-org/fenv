import { toWritableStream, Writer } from '@std/io';
import { FenvContext, OperationSystem } from '@fenv/lib';

export function contextFrom(options: {
  stdout?: Writer;
  stderr?: Writer;
  os?: OperationSystem;
}): FenvContext {
  const {
    stdout = Deno.stdout,
    stderr = Deno.stderr,
    os = OperationSystem.LINUX,
  } = options;
  return new FenvContext(
    toWritableStream(stdout, { autoClose: false }),
    toWritableStream(stderr, { autoClose: false }),
    os,
    os !== OperationSystem.WINDOWS ? '/bin/bash' : '',
  );
}
