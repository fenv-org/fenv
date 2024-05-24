import { Command, ValidationError } from '@cliffy/command';
import { VERSION } from './src/version.ts';
import { initCommand } from './src/commands/init.ts';
import { Buffer, Writer, WriterSync } from '@std/io';

export async function main(
  { args, stdout, stderr }: {
    args: string[];
    stdout: WritableStream<Uint8Array>;
    stderr: WritableStream<Uint8Array>;
  },
): Promise<void> {
  await new Command()
    .name('fenv')
    .version(`v${VERSION}`)
    .description('Simple flutter sdk version management')
    .command(
      'init',
      initCommand.action((options, pathMode) =>
        stdout.getWriter().write(
          new TextEncoder().encode(
            `init command with options: ${JSON.stringify(options)}\n`,
          ),
        )
      ),
    )
    .error((err) => {
      if (err instanceof ValidationError) {
        console.error(err.message);
      }
    })
    .parse(args);
}

if (import.meta.main) {
  await main({
    args: Deno.args,
    stdout: Deno.stdout.writable,
    stderr: Deno.stderr.writable,
  });
}
