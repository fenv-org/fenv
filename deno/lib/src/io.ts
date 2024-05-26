export const encoder = new TextEncoder();
export const decoder = new TextDecoder();

export async function writeText(
  stream: WritableStream<Uint8Array>,
  ...obj: unknown[]
): Promise<void> {
  const text = obj.map(serialize).join(' ');
  await stream.getWriter().write(encoder.encode(text));
}

export async function writeTextLine(
  stream: WritableStream<Uint8Array>,
  ...obj: unknown[]
): Promise<void> {
  const text = obj.map(serialize).join(' ') + '\n';
  await stream.getWriter().write(encoder.encode(text));
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
