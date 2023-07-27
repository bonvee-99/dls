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

  add_client(client: Client) {
    if (this.clients.size < this.capacity - 1) {
      this.clients.add(client)
      client.ws.send(`Successfully joined room ${this.id}`)
      this.owner.ws.send(`User with id: ${client.id} joined your room`)
    } else {
      client.ws.send(`Room ${this.id} is full`)
    }
  }

  has_client(client: Client) {
    return this.clients.has(client)
  }

  broadcast_message(sender: Client, message: string) {
    this.clients.forEach((client: Client) => {
      console.log('client', client.id)
      if (client !== sender && client.ws.readyState === WebSocket.OPEN) {
        client.ws.send(`Received message: ${message} from user ${sender.id}`)
      }
    })
  }
}
