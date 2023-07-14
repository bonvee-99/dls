import express from 'express'
import http from 'http'
import WebSocket from 'ws'
import { AddressInfo } from 'net'
import { v4 as uuidv4 } from 'uuid'

const app = express();
// initialize a simple http server
const server = http.createServer(app);
// initialize the WebSocket server instance
const wss = new WebSocket.Server({ server });

interface Custom_Web_Socket extends WebSocket {
  id: string,
  rooms: []
}

interface WS_Message {
  type: string
}

wss.on('connection', function connection(ws: Custom_Web_Socket) {
  ws.id = uuidv4()
  console.log('New client connected: ', ws.id)

  ws.on('close', () => {
    console.log(`Client ${ws.id} disconnected`)
  })

  ws.on('message', (data: string) => {
    try {
      console.log(`Client ${ws.id} has sent us: ${data}`)
      // const parsedMessage: WS_Message = JSON.parse(data)
      ws.send('Received valid JSON')
    } catch (error) {
      console.error(error)
      ws.send('Received invalid JSON')
    }
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
