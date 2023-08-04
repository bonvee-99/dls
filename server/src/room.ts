import { v4 as uuidv4 } from 'uuid'
import Client from './client'
import WebSocket from 'ws'
import nconf from 'nconf'

export default class Room {
  id: string
  owner: Client
  clients: Set<Client>
  readonly capacity: number

  constructor(owner: Client) {
    this.id = uuidv4()
    this.owner = owner
    this.clients = new Set()
    this.clients.add(owner)
    this.capacity = nconf.get('ROOM_CAPACITY')
  }

  add_client(client: Client): boolean {
    if (this.has_client(client)) {
      client.ws.send(JSON.stringify({ Message: { text: 'You are already in this room' } }))
      return false
    }
    if (this.clients.size >= this.capacity) {
      client.ws.send(JSON.stringify({ Message: { text: 'Room is full' } }))
      return false
    }
    this.clients.add(client)
    // maybe send to all clients ?
    this.broadcast_message(client, `User with id: ${client.id} joined your room`, false)
    client.ws.send(JSON.stringify({ JoinRoom: { room_id: this.id, text: `Successfully joined room ${this.id}` } }))
    return true
  }
  
  remove_client(client: Client) {
    if (this.has_client(client)) {
      // TODO: handle when owner leaves the room
      this.broadcast_message(client, `User with id: ${client.id} left your room`, false)
      this.clients.delete(client)
      client.ws.send(JSON.stringify({ Message: { text: `Successfully left room: ${this.id}` } }))
    } else {
      // you are not in this room
    }
  }

  has_client(client: Client) {
    return this.clients.has(client)
  }

  broadcast_message(sender: Client, message: string, secret: boolean) {
    if (!this.has_client(sender)) {
      sender.ws.send(JSON.stringify({ Message: { text: 'Unable to send message. You are not in this room' } }))
    }
    this.clients.forEach((client: Client) => {
      if (client !== sender && client.ws.readyState === WebSocket.OPEN) {
        // we do this secret flag to prevent the user from pretending to leave the room. This way we know when a user sends a message or the server is telling us something
        if (secret) {
          client.ws.send(JSON.stringify({ Message: { text: `${sender.id}: ${message}` } }))
        } else {
          client.ws.send(JSON.stringify({ Message: { text: message } }))
        }
      }
    })
  }
}
