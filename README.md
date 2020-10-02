# Distributed key value store
This is a distributed key value store optimized for large values (like files).


### API

- GET /key
  - 302 redirect to volume server.
- PUT /key
  - Blocks. 201 = written, anything else = probably not written.
- DELETE /key
  - Blocks. 204 = deleted, anything else = probably not deleted.

### Start Volume Servers (default port 3001)

```
# this is just under the hood
PORT=3001 ./volume /tmp/volume1/ &;
PORT=3002 ./volume /tmp/volume2/ &;
PORT=3003 ./volume /tmp/volume3/ &;
```

### Start Master Server (default port 3000)

```
./master localhost:3001,localhost:3002,localhost:3003 /tmp/indexdb/
```


### Usage

```
# put "bigswag" in key "wehave"
curl -v -L -X PUT -d bigswag localhost:3000/wehave

# get key "wehave" (should be "bigswag")
curl -v -L localhost:3000/wehave

# delete key "wehave"
curl -v -L -X DELETE localhost:3000/wehave

# put file in key "file.txt"
curl -v -L -X PUT -T /path/to/local/file.txt localhost:3000/file.txt

# get file in key "file.txt"
curl -v -L -o /path/to/local/file.txt localhost:3000/file.txt
```

### ./mkv Usage

```
Usage: ./mkv <server, rebuild, rebalance>

  -db string
        Path to leveldb
  -fallback string
        Fallback server for missing keys
  -port int
        Port for the server to listen on (default 3000)
  -protect
        Force UNLINK before DELETE
  -replicas int
        Amount of replicas to make of the data (default 3)
  -subvolumes int
        Amount of subvolumes, disks per machine (default 10)
  -volumes string
        Volumes to use for storage, comma separated
```

### Performance

```
# Fetching non-existent key
wrk -t2 -c100 -d10s http://localhost:3000/key
Requests/sec: 236960.61
Transfer/sec: 18.53MB


# go run thrasher.go
starting thrasher
10000 write/read/delete in 1.930297339s
thats 5180.52/sec
```

