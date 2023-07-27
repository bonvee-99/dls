import WebSocket from 'ws'
import { v4 as uuidv4 } from 'uuid'

export default class Client {
  id: string
  name: string
  ws: WebSocket

  constructor(name: string, ws: WebSocket) {
    this.id = uuidv4()
    this.name = name
    this.ws = ws
  }
}
