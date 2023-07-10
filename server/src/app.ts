import express from 'express';
import http from 'http';
import WebSocket from 'ws';
import { AddressInfo } from 'net';

const app = express();
// initialize a simple http server
const server = http.createServer(app);
// initialize the WebSocket server instance
const wss = new WebSocket.Server({ server });

wss.on('connection', function connection(ws) {
  ws.on('error', console.error);

  ws.on('message', function message(data) {
    console.log('received: %s', data);
  });

  ws.send('something');
});

// start our server
server.listen(process.env.PORT || 8999, () => {
    console.log(`Server started on port ${(server.address() as AddressInfo).port} :)`);
});
