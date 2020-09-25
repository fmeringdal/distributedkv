# Distributed key value store
This is a distributed key value store optimized for large values (like files).



### API

- GET /key
  - 307 redirect to volume server.
- {PUT} /key
  - Blocks. 200 = written, anything else = nothing happened.

### Start Master Server (default port 3000)

```
./master localhost:3001,localhost:3002 /tmp/cachedb/
```

### Start Volume Server (default port 3001)

```
./volume /tmp/volume1/
PORT=3002 ./volume /tmp/volume2/
```

### Usage

```
# put "bigswag" in key "wehave"
curl -L -X PUT -d bigswag localhost:3000/wehave

# get key "wehave" (should be "bigswag")
curl -L localhost:3000/wehave
```

