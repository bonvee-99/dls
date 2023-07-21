import { v4 as uuidv4 } from 'uuid'
import Client from './client'
import WebSocket from 'ws'

const CAPACITY = 2

export default class Room {
  id: string
  owner: Client
  clients: Set<Client>
  capacity = CAPACITY

  constructor(owner: Client) {
    this.id = uuidv4()
    this.owner = owner
    this.clients = new Set()
    this.clients.add(owner)
  }

  add_client(client: Client) {
    // TODO: make sure capacity isnt overflowed
    this.clients.add(client)
    client.ws.send(`Successfully joined room ${this.id}`)
    this.owner.ws.send(`User with id: ${client.id} joined your room`)
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
