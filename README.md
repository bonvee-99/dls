### Server setup locally
```bash
cd server && npm install
npm run dev
```

### Websocket setup locally
```bash
cd cli
cargo run
```
# alternatively you can use wscat for testing
```bash
npm i -g wscat
wscat -c ws://localhost:3000
```
