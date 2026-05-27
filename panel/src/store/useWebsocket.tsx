import { useEffect, useRef, useCallback } from 'react'
import type { ConnectionStatus } from '../types'

type SMsg =
  | { type: 'ready' }
  | { type: 'sync'; payload: unknown }
  | { type: 'error'; message: string }

type CMsg =
  | { type: 'request_sync' }
  | { type: 'publish'; file: unknown }

export interface WsHandlers {
  onSync: (payload: unknown) => void
}

const WS_RECONNECT_DELAY_MS = 3_000

export function useWebSocket(
  setStatus: (s: ConnectionStatus) => void,
  handlers: WsHandlers,
) {
  const wsRef = useRef<WebSocket | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const handlersRef = useRef(handlers)
  useEffect(() => { handlersRef.current = handlers }, [handlers])

  const token = new URLSearchParams(window.location.search).get('token') ?? ''

  const send = useCallback((msg: CMsg) => {
    const ws = wsRef.current
    if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg))
  }, [])

  const connect = useCallback(() => {
    if (!token) {
      setStatus('disconnected')
      return
    }

    setStatus('connecting')

    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws'
    const ws = new WebSocket(`${protocol}://${window.location.host}/ws?token=${token}`)
    wsRef.current = ws

    ws.onopen = () => {
    }

    ws.onmessage = ({ data }: MessageEvent<string>) => {
      let msg: SMsg
      try { msg = JSON.parse(data) as SMsg }
      catch { console.warn('[ws] message invalide', data); return }

      switch (msg.type) {
        case 'ready':
          setStatus('connected')
          send({ type: 'request_sync' })
          break
        case 'sync':
          handlersRef.current.onSync(msg.payload)
          break
        case 'error':
          console.warn('[ws] erreur serveur:', msg.message)
          break
      }
    }

    ws.onclose = () => {
      setStatus('disconnected')
      wsRef.current = null
      timerRef.current = setTimeout(connect, WS_RECONNECT_DELAY_MS)
    }

    ws.onerror = () => {
      setStatus('disconnected')
      if (timerRef.current) clearTimeout(timerRef.current)
    }
  }, [token, setStatus, send])

  useEffect(() => {
    connect()
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current)
      wsRef.current?.close()
    }
  }, [connect])

  return { send }
}
