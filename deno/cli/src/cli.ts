import { Command } from '@cliffy/command';
import { VERSION } from './version.ts';

async function main(args: string[]): Promise<void> {
  const command = await new Command()
    .name('fenv')
    .version(`v${VERSION}`)
    .parse(args);
  console.error(command);
}

if (import.meta.main) main(Deno.args);
