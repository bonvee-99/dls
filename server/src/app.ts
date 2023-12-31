import express from 'express'
import http from 'http'
import WebSocket from 'ws'
import { AddressInfo } from 'net'
import Room from './room'
import Client from './client'
import nconf from 'nconf'
import path from 'path'

const app = express();
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

// TODO: we need to use a mutex here
const rooms = new Map<string,Room>()

nconf
  .file('appsettings', { file: path.join(__dirname, 'appsettings.json') })

wss.on('connection', function connection(ws: WebSocket) {
  const client = new Client(ws)
  console.log('New client connected: ', client.id)

  ws.on('close', () => {
    if (client.room) {
      client.room.remove_client(client)
    }
    console.log(`Client ${client.id} disconnected`)
  })

  ws.on('message', (data: string) => {
    handle_message(data, client)
  })

  ws.send(JSON.stringify(
    { Message: { text: `Your id is: ${client.id}` } }
  ))
})

server.listen(process.env.PORT || 3000, () => {
    console.log(nconf.get())
    console.log(`Server started on port ${(server.address() as AddressInfo).port}`)
})

function handle_message(data: string, client: Client) {
  try {
    const { message_type, room_id, secret_message, public_key } = JSON.parse(data)

    console.log(JSON.parse(data))

    // TODO: validate that message_type and room_id are valid ?

    if (message_type === 'create') {
      const room = new Room(client, public_key)
      rooms.set(room.id, room)
      const message = {
        CreateRoom: {
          text: `Created room with the id: ${room.id}`,
          room_id: room.id
        }
      }
      client.ws.send(JSON.stringify(message))
    } else if (message_type === 'join') {
      join_room(client, room_id, public_key)
    } else if (message_type === 'secret') {
      send_message(client, room_id, secret_message)
    // TODO: leave room
    } else {
      // ???
    }
  } catch (error) {
    console.error(error)
    client.ws.send(JSON.stringify({ Message: { text: 'Received invalid JSON' } }))
  }
}

function join_room(client: Client, room_id: string, public_key: string) {
  const room = rooms.get(room_id)
  if (!room) {
    client.ws.send(JSON.stringify({ Message: { text: `No room with the id: ${room_id} found` } }))
    return
  }

  client.join_room(room, public_key)
}

function send_message(client: Client, room_id: string, secret_message: string) {
  // check if client belongs to room even
  const room = rooms.get(room_id)

  if (!room) {
    client.ws.send(JSON.stringify({ Message: { text: `No room with the id: ${room_id} found` } }))
    return
  }

  // if (!room.has_client(client)) {
  //   // client.ws.send(`You are not in this room`)
  //   return
  // }
  room.broadcast_message(client, JSON.stringify({ SecretMessage: { user_id: client.id, text: secret_message } }))
}

