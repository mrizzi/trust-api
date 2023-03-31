# trust-api

API server for trusted content (TC).

## Running locally

Running:

``` 4d
cargo run -- run -p 8080
```

Usage:

``` 4d
curl --json '["pkg:maven/org.quarkus/quarkus@1.2"]' http://localhost:8080/api/package
```

## Testing with guac

Start guac:

``` 4d
podman run -p 8080:8080 -ti ghcr.io/dejanb/local-organic-guac /opt/guac/guacone gql-server --gql-debug
```

Ingest test data:

```
podman run --net=host -v $PWD/data/files:/data:z -ti ghcr.io/dejanb/local-organic-guac /opt/guac/guacone files /data
```

Run the API server:

```
cargo run -- run -p 8081
```

You can also run the API server using a container:

```
podman run -p 8080:8080 -ti ghcr.io/xkcd-2347/trust-api:latest run -p 8081 
```
