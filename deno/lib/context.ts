import { OperationSystem } from './os.ts';

export type FenvContext = {
  /**
   * The operation system of the running host.
   *
   * @see Deno.build.os
   */
  os: OperationSystem;

  /**
   * The shell executable that `$SHELL` environment variable points to.
   *
   * If the running host is Windows, the default shell is an empty string.
   */
  defaultShell: string;

  /**
   * The root directory of the fenv installation.
   */
  fenvRoot?: string;
};
