import { useEffect, useRef, useCallback } from 'react'
import type { ConnectionStatus } from '../types'

type ServerMsg =
  | { type: 'ready' }
  | { type: 'auth_error'; message: string }
  | { type: 'sync'; payload: unknown }
  | { type: 'update'; payload: unknown }
  | { type: 'error'; message: string }

type ClientMsg =
  | { type: 'auth'; token: string }
  | { type: 'publish'; file: unknown }
  | { type: 'request_sync' }

export interface WsHandlers {
  onSync: (payload: unknown) => void
  onUpdate: (payload: unknown) => void
}

const WS_RECONNECT_DELAY_MS = 3_000

export function useWebSocket(
  setStatus: (s: ConnectionStatus) => void,
  handlers: WsHandlers,
) {
  const wsRef = useRef<WebSocket | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const handlersRef = useRef(handlers)
  const authFailedRef = useRef(false)

  // Keep handlers ref up-to-date without re-triggering connect
  useEffect(() => { handlersRef.current = handlers }, [handlers])

  const token = new URLSearchParams(window.location.search).get('token') ?? ''

  const send = useCallback((msg: ClientMsg) => {
    const ws = wsRef.current
    if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg))
  }, [])

  const connect = useCallback(() => {
    if (!token) {
      setStatus('disconnected')
      return
    }

    authFailedRef.current = false
    setStatus('connecting')

    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws'
    const ws = new WebSocket(`${protocol}://${window.location.host}/ws`)
    wsRef.current = ws

    ws.onopen = () => ws.send(JSON.stringify({ type: 'auth', token } satisfies ClientMsg))

    ws.onmessage = ({ data }: MessageEvent<string>) => {
      let msg: ServerMsg
      try { msg = JSON.parse(data) as ServerMsg }
      catch { console.warn('[ws] unparseable message', data); return }

      switch (msg.type) {
        case 'ready':
          setStatus('connected')
          ws.send(JSON.stringify({ type: 'request_sync' } satisfies ClientMsg))
          break
        case 'auth_error':
          console.error('[ws] auth failed:', msg.message)
          authFailedRef.current = true
          setStatus('disconnected')
          ws.close()
          break
        case 'sync':    handlersRef.current.onSync(msg.payload);   break
        case 'update':  handlersRef.current.onUpdate(msg.payload); break
        case 'error':   console.warn('[ws] server error:', msg.message); break
      }
    }

    ws.onclose = () => {
      setStatus('disconnected')
      wsRef.current = null
      if (!authFailedRef.current)
        timerRef.current = setTimeout(connect, WS_RECONNECT_DELAY_MS)
    }

    ws.onerror = () => setStatus('disconnected')
  }, [token, setStatus]) // handlers intentionally excluded — accessed via ref

  useEffect(() => {
    connect()
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
      wsRef.current?.close()
    }
  }, [connect])

  return { send }
}
