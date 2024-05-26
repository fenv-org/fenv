export class FenvContext {
  constructor(
    public stdout: WritableStream<Uint8Array>,
    public stderr: WritableStream<Uint8Array>,
  ) {}
}
