import express from 'express'
import http from 'http'
import WebSocket from 'ws'
import { AddressInfo } from 'net'
import Room from './room'
import Client from './client'

const app = express();
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

// TODO: we need to use a mutex here probably
const rooms = new Map<string,Room>()

wss.on('connection', function connection(ws: WebSocket) {
  const client = new Client('dave', ws)
  console.log('New client connected: ', client.id)

  ws.on('close', () => {
    // TODO: leave all rooms its connected to (maybe add 'room' field on client?)
    console.log(`Client ${client.id} disconnected`)
  })

  ws.on('message', (data: string) => {
    handle_message(data, client)
  })

  ws.send(`Your id is: ${client.id}`)
})

server.listen(process.env.PORT || 3000, () => {
    console.log(`Server started on port ${(server.address() as AddressInfo).port}`)
})

function handle_message(data: string, client: Client) {
  try {
    const { message_type, room_id, secret_message } = JSON.parse(data)

    // TODO: validate that message_type and room_id are valid ?

    if (message_type === 'create') {
      // create room with uuid
      const room = new Room(client)
      rooms.set(room.id, room)
      client.ws.send(`Created room with the id: ${room.id}`)
    } else if (message_type === 'join') {
      join_room(client, room_id)
    } else if (message_type === 'secret') {
      send_message(client, room_id, secret_message)
    } else {
      // ???
    }
  } catch (error) {
    console.error(error)
    client.ws.send('Received invalid JSON')
  }
}

// TODO: do we want to add these methods to client class ?
function join_room(client: Client, room_id: string) {
  const room = rooms.get(room_id)
  if (!room) {
    client.ws.send(`No room with the id: ${room_id} found`)
    return
  }

  if (room.has_client(client)) {
    client.ws.send(`You are already in this room`)
    return
  }

  room.add_client(client)
}

function send_message(client: Client, room_id: string, secret_message: string) {
  // check if client belongs to room even
  const room = rooms.get(room_id)

  if (!room) {
    client.ws.send(`No room with the id: ${room_id} found`)
    return
  }

  if (!room.has_client(client)) {
    client.ws.send(`You are already in this room`)
    return
  }

  room.broadcast_message(client, secret_message)
}

