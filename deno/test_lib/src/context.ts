import { toWritableStream, Writer } from '@std/io';
import { FenvContext } from '@fenv/lib';

export function contextFrom(options: {
  stdout?: Writer;
  stderr?: Writer;
}): FenvContext {
  const { stdout = Deno.stdout, stderr = Deno.stderr } = options;
  return new FenvContext(
    toWritableStream(stdout, { autoClose: false }),
    toWritableStream(stderr, { autoClose: false }),
  );
}
