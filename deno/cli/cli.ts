import { Command, ValidationError } from '@cliffy/command';
import { VERSION } from './src/version.ts';
import { initCommand } from './src/commands/init.ts';

export async function main(args: string[]): Promise<void> {
  const command = await new Command()
    .name('fenv')
    .version(`v${VERSION}`)
    .description('Simple flutter sdk version management')
    .command('init', initCommand.action((options, pathMode) => console.log(options, pathMode)))
    .error((err) => {
      if (err instanceof ValidationError) {
        console.error(err.message);
      }
    })
    .parse(args);
  console.error('args', command.args);
  console.error('literal', command.literal);
  console.error('options', command.options);
}

if (import.meta.main) main(Deno.args);
