import { OperationSystem } from '../os.ts';

export class FenvContext {
  constructor(
    /**
     * The standard output stream.
     *
     * @see Deno.stdout
     */
    public stdout: WritableStream<Uint8Array>,
    /**
     * The standard error stream.
     *
     * @see Deno.stderr
     */
    public stderr: WritableStream<Uint8Array>,
    /**
     * The operation system of the running host.
     *
     * @see Deno.build.os
     */
    public os: OperationSystem,
    /**
     * The shell executable that `$SHELL` environment variable points to.
     *
     * If the running host is Windows, the default shell is an empty string.
     */
    public defaultShell: string,
  ) {}
}
