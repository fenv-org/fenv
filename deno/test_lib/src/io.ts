import { Buffer } from '@std/io';

export function bufferToText(buffer: Buffer): string {
  return new TextDecoder().decode(buffer.bytes());
}
