# Axum Webserver example for using MPSC for sync long compute tasks like Machine Learning Prediction

This example uses `axum` to create a webserver, which can be used to run long compute tasks like Machine Learning Prediction. On Start up it will create a `mpsc` channel passing the `tx` into the route and threads for incoming requests. The `rx` loads the  compute heavy context and listens to the channel. The Workers are sending the messages to the channel which blocks the `rx` thread. The `rx` thread is waiting for the messages and processes them.

### Benchmark hey 

curl
```bash
curl -X POST \
  'http://127.0.0.1:3000/predict' \
  -H 'content-type: application/json' \
  -d '{
  "inputs": 3
}'
```

hey script

```bash
hey -n 10 -c 10 \
  -m POST \
  -H 'content-type: application/json' \
  -d '{
  "inputs": 3
}' \
'http://127.0.0.1:3000/predict' 
```