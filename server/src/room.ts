import { v4 as uuidv4 } from 'uuid'
import Client from './client'
import WebSocket from 'ws'
import nconf from 'nconf'

export default class Room {
  id: string
  owner: Client
  clients: Set<Client>
  public_keys: Map<string, string>
  readonly capacity: number

  constructor(owner: Client, public_key: string) {
    this.id = uuidv4()
    this.owner = owner
    this.clients = new Set()
    this.clients.add(owner)
    this.capacity = nconf.get('ROOM_CAPACITY')
    this.public_keys = new Map()
    this.public_keys.set(owner.id, public_key)
  }

  add_client(client: Client, public_key: string): boolean {
    if (this.has_client(client)) {
      client.ws.send(JSON.stringify({ Message: { text: 'You are already in this room' } }))
      return false
    }
    if (this.clients.size >= this.capacity) {
      client.ws.send(JSON.stringify({ Message: { text: 'Room is full' } }))
      return false
    }

    const keys = Array.from(this.public_keys).map(([user_id, public_key]) => ({
      user_id, public_key
    }))
    
    this.clients.add(client)
    this.broadcast_message(client, JSON.stringify({ PublicKey: { text: `User with id: ${client.id} joined your room`, public_key, user_id: client.id } }))
    // TODO: now we need to send client array of public keys to save
    client.ws.send(JSON.stringify({ JoinRoom: { room_id: this.id, text: `Successfully joined room ${this.id}`, public_keys: keys } }))
    return true
  }
  
  remove_client(client: Client) {
    if (this.has_client(client)) {
      // TODO: handle when owner leaves the room
      this.broadcast_message(client, JSON.stringify({ Message: { text: `User with id: ${client.id} left your room` } }))
      this.clients.delete(client)
      client.ws.send(JSON.stringify({ Message: { text: `Successfully left room: ${this.id}` } }))
    } else {
      // you are not in this room
    }
  }

  has_client(client: Client) {
    return this.clients.has(client)
  }

  broadcast_message(sender: Client, message: string) {
    if (!this.has_client(sender)) {
      sender.ws.send(JSON.stringify({ Message: { text: 'Unable to send message. You are not in this room' } }))
    }
    this.clients.forEach((client: Client) => {
      if (client !== sender && client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(message)
      }
    })
  }
}
