export interface SseMessage {
  event: string;
  data: string;
  id?: string;
  retry?: number;
}

export async function readServerSentEvents<T>(
  response: Response,
  onEvent: (event: T, raw: SseMessage) => void,
): Promise<void> {
  if (!response.ok) {
    throw new Error(`SSE request failed with ${response.status}`);
  }
  if (!response.body) {
    throw new Error("SSE response body is empty");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    buffer += decoder.decode(value, { stream: true });
    const chunks = buffer.split(/\r?\n\r?\n/);
    buffer = chunks.pop() ?? "";
    for (const chunk of chunks) {
      const message = parseSseMessage(chunk);
      if (!message.data) {
        continue;
      }
      onEvent(JSON.parse(message.data) as T, message);
    }
  }

  if (buffer.trim()) {
    const message = parseSseMessage(buffer);
    if (message.data) {
      onEvent(JSON.parse(message.data) as T, message);
    }
  }
}

function parseSseMessage(chunk: string): SseMessage {
  const message: SseMessage = { event: "message", data: "" };
  const dataLines: string[] = [];

  for (const line of chunk.split(/\r?\n/)) {
    if (!line || line.startsWith(":")) {
      continue;
    }
    const separatorIndex = line.indexOf(":");
    const field = separatorIndex >= 0 ? line.slice(0, separatorIndex) : line;
    const rawValue = separatorIndex >= 0 ? line.slice(separatorIndex + 1) : "";
    const value = rawValue.startsWith(" ") ? rawValue.slice(1) : rawValue;

    if (field === "event") {
      message.event = value;
    } else if (field === "data") {
      dataLines.push(value);
    } else if (field === "id") {
      message.id = value;
    } else if (field === "retry") {
      message.retry = Number(value);
    }
  }

  message.data = dataLines.join("\n");
  return message;
}
