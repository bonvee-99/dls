import WebSocket from 'ws'
import { v4 as uuidv4 } from 'uuid'
import Room from './room'

export default class Client {
  id: string
  ws: WebSocket
  room: Room | null

  constructor(ws: WebSocket) {
    this.id = uuidv4()
    this.ws = ws
    this.room = null
  }

  join_room(room: Room, public_key: string) {
    if (this.room) {
      this.ws.send(JSON.stringify({ Message: { text: `Leaving room ${this.room}` } }))
      this.room.remove_client(this)
    }
    const joined = room.add_client(this, public_key)
    if (joined) {
      this.room = room
    }
  }
}
