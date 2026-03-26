import type {
  DeviceSessionInput,
  DeviceWorksListOutput,
} from '@server/shared/schema'

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${window.location.origin}${path}`, {
    ...init,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  })

  if (!response.ok) {
    throw new Error(await response.text())
  }

  if (response.status === 204) {
    return undefined as T
  }

  return await response.json() as T
}

export const api = {
  async createSession(input: DeviceSessionInput): Promise<void> {
    await apiFetch('/api/device/session', {
      method: 'POST',
      body: JSON.stringify(input),
    })
  },
  async listWorks(deviceId: string): Promise<DeviceWorksListOutput> {
    return await apiFetch(`/api/device/${encodeURIComponent(deviceId)}/works`, {
      method: 'GET',
      headers: {},
    })
  },
  async launchWork(deviceId: string, workId: string): Promise<void> {
    await apiFetch(`/api/device/${encodeURIComponent(deviceId)}/works/${encodeURIComponent(workId)}/launch`, {
      method: 'POST',
    })
  },
}
