/**
 * The HTTP client for the service.
 *
 * Every call returns parsed JSON. A failure carries the status and, when the
 * service supplied one, the message from the error body, so that a caller
 * can show the reason rather than a generic message.
 */

/** The shape of a failed call. */
export class ApiError extends Error {
  readonly status: number;
  readonly parameter: string | null;

  constructor(message: string, status: number, parameter: string | null = null) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.parameter = parameter;
  }
}

/** Fetches a JSON document from a GET route. */
export async function getJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { headers: { accept: 'application/json' } });
  if (!response.ok) {
    throw new ApiError(`request failed with status ${response.status}`, response.status);
  }
  return (await response.json()) as T;
}

/** Posts a JSON body to a route and returns the parsed response. */
export async function postJson<T>(path: string, body: unknown): Promise<T> {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'content-type': 'application/json', accept: 'application/json' },
    body: JSON.stringify(body),
  });
  const text = await response.text();
  let parsed: unknown = null;
  try {
    parsed = text.length > 0 ? JSON.parse(text) : null;
  } catch {
    throw new ApiError('the service returned a response that is not JSON', response.status);
  }
  if (!response.ok && parsed === null) {
    throw new ApiError(`request failed with status ${response.status}`, response.status);
  }
  return parsed as T;
}

/** Builds a query string from a record, skipping empty values. */
export function queryString(parameters: Record<string, string | number | undefined>): string {
  const parts: string[] = [];
  for (const [key, value] of Object.entries(parameters)) {
    if (value === undefined || value === '') continue;
    parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
  }
  return parts.length > 0 ? `?${parts.join('&')}` : '';
}
