// deno-lint-ignore-file no-explicit-any
import { Command } from '@cliffy/command';

type ActionHandlerParameters<T> = T extends Command<
  any,
  any,
  any,
  any,
  any,
  any,
  any,
  any
> ? Parameters<Parameters<T['action']>['0']>
  : never;

export type OptionsOf<T> = ActionHandlerParameters<T>['0'];

type ExceptHead<T extends any[]> = T extends [any, ...infer Rest] ? Rest
  : never;

export type ArgumentsOf<T> = ExceptHead<ActionHandlerParameters<T>>;
