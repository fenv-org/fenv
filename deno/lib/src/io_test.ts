import { writeText, writeTextLine } from './io.ts';
import { assertEquals } from '@std/assert';
import { beforeEach, describe, it } from '@std/testing/bdd';
import { Buffer, toWritableStream } from '@std/io';
import { bufferToText } from '@fenv/test_lib';

describe('writeText', () => {
  let buffer: Buffer;
  let stream: WritableStream<Uint8Array>;

  beforeEach(() => {
    buffer = new Buffer();
    stream = toWritableStream(buffer);
  });

  it('ensure writeText works with strings', async () => {
    await writeText(stream, 'Hello', 'world', 1, 2, 3, 4, ['a', 'b', true, false], {
      foo: 'bar',
    });
    assertEquals(
      bufferToText(buffer),
      'Hello world 1 2 3 4 [a, b, true, false] {"foo":"bar"}',
    );
  });

  it('ensure writeTextLine works with objects', async () => {
    await writeTextLine(stream, 'Hello', 'world', 1, 2, 3, 4, ['a', 'b', true, false], {
      foo: 'bar',
    });
    assertEquals(
      bufferToText(buffer),
      'Hello world 1 2 3 4 [a, b, true, false] {"foo":"bar"}\n',
    );
  });
});
