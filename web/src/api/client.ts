export type Health = { status: 'ok'; version: string; database: 'up' | 'down' }

export async function getHealth(): Promise<Health> {
  const res = await fetch('/api/v1/health')
  if (!res.ok) throw new Error(`Health check failed: ${res.status}`)
  return res.json() as Promise<Health>
}
