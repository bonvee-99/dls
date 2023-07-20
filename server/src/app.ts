import express from 'express'
import http from 'http'
import WebSocket from 'ws'
import { AddressInfo } from 'net'
import { v4 as uuidv4 } from 'uuid'

const app = express();
const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

interface Custom_Web_Socket extends WebSocket {
  id: string,
  rooms: []
}

interface WS_Message {
  type: string
}

const rooms = new Map()

wss.on('connection', function connection(ws: Custom_Web_Socket) {
  ws.id = uuidv4()
  console.log('New client connected: ', ws.id)

  ws.on('close', () => {
    // leave all rooms its connected to
    console.log(`Client ${ws.id} disconnected`)
  })

  ws.on('message', (data: string) => {
    handle_message(data, ws)
  })

  ws.send(`Your id is: ${ws.id}`)

  wss.clients.forEach((client) => {
    console.log('Client.ID: ', (client as Custom_Web_Socket).id)
  });
})

// start our server
server.listen(process.env.PORT || 3000, () => {
    console.log(`Server started on port ${(server.address() as AddressInfo).port}`)
})

function handle_message(data: string, ws: Custom_Web_Socket) {
  try {
    // client can (1) create room, (2) join room, (3) send data
    console.log(`Client ${ws.id} has sent us: ${data}`)
    // const parsedMessage: WS_Message = JSON.parse(data)
    ws.send('Received valid JSON')
    // send to room
  } catch (error) {
    console.error(error)
    ws.send('Received invalid JSON')
  }
}

// TODO: allow user to create room, and let others join it using the given url, leave room upon WS disconnect

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
