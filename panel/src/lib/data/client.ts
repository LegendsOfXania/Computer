import { MockServer, type MockServerOptions } from "./mock/server";
import type { ClientMessage, ServerMessage } from "$lib/protocol/messages";

export interface Client {
  send(message: ClientMessage): void;
  subscribe(handler: (message: ServerMessage) => void): () => void;
  disconnect(): void;
}

export function createClient(options?: MockServerOptions): Client {
  return new MockServer(options).connect();
}
