export const encoder = new TextEncoder();
export const decoder = new TextDecoder();

export async function writeText(
  stream: WritableStream<Uint8Array>,
  ...obj: unknown[]
): Promise<void> {
  const text = obj.map(serialize).join(' ');
  await writeTo(stream.getWriter(), text);
}

export async function writeTextLine(
  stream: WritableStream<Uint8Array>,
  ...obj: unknown[]
): Promise<void> {
  const text = obj.map(serialize).join(' ') + '\n';
  await writeTo(stream.getWriter(), text);
}

async function writeTo(
  writer: WritableStreamDefaultWriter<Uint8Array>,
  text: string,
): Promise<void> {
  await writer.ready;
  try {
    await writer.write(encoder.encode(text));
  } finally {
    writer.releaseLock();
  }
}

function serialize(obj: unknown): string {
  if (typeof obj === 'string') {
    return obj;
  } else if (typeof obj === 'object') {
    if (Array.isArray(obj)) {
      return '[' + obj.map(serialize).join(', ') + ']';
    } else {
      return JSON.stringify(obj);
    }
  } else {
    return `${obj}`;
  }
}
