import { v4 as uuidv4 } from 'uuid'
import Client from './client'
import WebSocket from 'ws'

export default class Room {
  id: string
  owner: Client
  clients: Set<Client>
  capacity = 2

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
    console.log('broadcasting message', message)
    if (this.clients) {
      this.clients.forEach((client: Client) => {
        console.log('client', client.id)
        if (client !== sender && client.ws.readyState === WebSocket.OPEN) {
          client.ws.send(`Received message: ${message} from user ${sender.id}`)
        }
      });
    } else {
      console.log(this.clients)
    }
  }
}
// function broadcastMessage(sender, room, message) {
//   const clients = rooms.get(room);
//
//   if (clients) {
//     clients.forEach((client) => {
//       if (client !== sender && client.readyState === WebSocket.OPEN) {
//         client.send(JSON.stringify({ type: 'message', content: message }));
//       }
//     });
//   }
// }
