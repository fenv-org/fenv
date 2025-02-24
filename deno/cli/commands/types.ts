// deno-lint-ignore-file no-explicit-any
import { Command } from '@cliffy/command';

export type OptionsOf<T> = T extends Command<
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any
> ? Parameters<Parameters<T['action']>['0']>['0']
  : never;
