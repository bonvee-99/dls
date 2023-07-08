import express, { Application, Request, Response, NextFunction} from 'express';
import http from 'http'
import { Server } from 'socket.io'
import cors from 'cors'

const app: Application = express()
const server = http.createServer(app)
const io = new Server(server, {
  allowUpgrades: false, 
  transports: ["polling"], 
  cors: { origin: "*" },
})

app.get('/', (req, res) => {
  res.send('<h1>Hello world</h1>')
})

io.on('connection', (socket) => { 
    socket.on('message', msg => {  
        io.emit('message', msg)
    })
    socket.on('clear', function clear() {  
        io.emit('clear')
    })
})

server.listen(3000, () => {
  console.log('listening on *:3000')
})
