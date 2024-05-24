import { assertEquals } from '@std/assert';
import $ from '@david/dax';
import { main } from '../../cli.ts';

Deno.test('temp', async () => {
  await main(['init', '-d']);
});
